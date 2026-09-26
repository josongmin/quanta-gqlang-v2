#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "binary ingress tests assert fixture/process plumbing and fail a bounded hang watchdog"
)]

mod support;

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(unix)]
use std::time::{Duration, Instant};

use quanta_gqlang_v2_cli::v3_plan::render_failure_v2;
use quanta_gqlang_v2_frontend_contract::{
    FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2, LocatedSpanV2,
    SourceIdV2, SpanV1,
};
use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

static FILE_ID: AtomicU64 = AtomicU64::new(0);

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(bytes: &[u8]) -> Self {
        let id = FILE_ID.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "qglang-cli-ingress-{}-{id}.gql",
            std::process::id()
        ));
        fs::write(&path, bytes).expect("write fixture");
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ignored = fs::remove_file(&self.path);
    }
}

fn run_os(args: &[OsString]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-cli"))
        .args(args)
        .output()
        .expect("spawn qglang cli")
}

#[cfg(unix)]
fn run_os_bounded(args: &[OsString]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-cli"))
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn qglang cli");
    let started = Instant::now();
    loop {
        if child.try_wait().expect("poll qglang cli").is_some() {
            return child.wait_with_output().expect("collect qglang cli output");
        }
        if started.elapsed() >= Duration::from_secs(10) {
            child.kill().expect("kill hung CLI");
            drop(child.wait_with_output().expect("reap hung CLI"));
            panic!("CLI did not reject a non-regular input within the watchdog");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn assert_usage_block<'a>(mut lines: impl Iterator<Item = &'a str>, rendered: &str) {
    assert!(
        lines
            .next()
            .is_some_and(|line| line.starts_with("cli.usage=quanta-gqlang-v2-cli <")),
        "{rendered}"
    );
    assert_eq!(lines.next(), Some("cli.read_only=true"), "{rendered}");
    assert_eq!(
        lines.next(),
        Some("cli.command_plane=forbidden"),
        "{rendered}"
    );
    assert_eq!(lines.next(), None, "{rendered}");
}

fn assert_io_error(stderr: &str, message_prefix: &str) {
    let mut lines = stderr.lines();
    assert_eq!(lines.next(), Some("cli.error.class=io"), "{stderr}");
    assert!(
        lines.next().is_some_and(|line| {
            line.strip_prefix("cli.error.message=")
                .is_some_and(|message| message.starts_with(message_prefix))
        }),
        "{stderr}"
    );
    assert_eq!(lines.next(), None, "{stderr}");
}

fn assert_usage_error(args: &[&str], message: &str) {
    let output = run_cli_argv_v1(args);
    assert_eq!(output.status.code(), Some(2), "argv {args:?}");
    assert!(cli_stdout_v1(&output).is_empty(), "argv {args:?}");
    let stderr = cli_stderr_v1(&output);
    let mut lines = stderr.lines();
    assert_eq!(lines.next(), Some("cli.error.class=usage"), "{stderr}");
    let expected_message = format!("cli.error.message={message}");
    assert_eq!(lines.next(), Some(expected_message.as_str()), "{stderr}");
    assert_usage_block(lines, &stderr);
}

#[test]
fn help_aliases_are_stdout_only_successes() {
    for token in ["help", "--help"] {
        let output = run_cli_argv_v1(&[token]);
        assert_eq!(output.status.code(), Some(0), "{token}");
        assert!(cli_stderr_v1(&output).is_empty(), "{token}");
        let stdout = cli_stdout_v1(&output);
        assert_usage_block(stdout.lines(), &stdout);
    }
}

#[test]
fn command_and_file_arity_errors_are_exact_usage_failures() {
    assert_usage_error(&[], "missing command");
    assert_usage_error(&["unknown", "x"], "unsupported command `unknown`");
    assert_usage_error(&["query"], "missing input or --file <path>");
    assert_usage_error(
        &["query", "--file"],
        "--file requires exactly one path argument",
    );
    assert_usage_error(
        &["query", "--file", "a", "b"],
        "--file requires exactly one path argument",
    );
}

#[test]
fn file_open_and_utf8_failures_use_the_documented_channel_and_exit() {
    let absent = std::env::temp_dir().join(format!(
        "qglang-cli-absent-{}-{}",
        std::process::id(),
        FILE_ID.fetch_add(1, Ordering::SeqCst)
    ));
    let output = run_os(&[
        OsString::from("query"),
        OsString::from("--file"),
        absent.into_os_string(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(cli_stdout_v1(&output).is_empty());
    let stderr = cli_stderr_v1(&output);
    assert_io_error(&stderr, "failed to open");

    #[cfg(unix)]
    {
        let output = run_os(&[
            OsString::from("query"),
            OsString::from("--file"),
            std::env::temp_dir().into_os_string(),
        ]);
        assert_eq!(output.status.code(), Some(1));
        assert!(cli_stdout_v1(&output).is_empty());
        let stderr = cli_stderr_v1(&output);
        assert_io_error(&stderr, "input path is not a regular file");
    }

    let invalid = TempFile::new(&[0xff, 0xfe]);
    let output = run_os(&[
        OsString::from("query"),
        OsString::from("--file"),
        invalid.path.clone().into_os_string(),
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(cli_stdout_v1(&output).is_empty());
    let stderr = cli_stderr_v1(&output);
    assert_eq!(
        stderr.lines().next(),
        Some("cli.error.class=usage"),
        "{stderr}"
    );
    assert_eq!(
        stderr.lines().nth(1),
        Some("cli.error.message=input file must be valid UTF-8"),
        "{stderr}"
    );
    assert_usage_block(stderr.lines().skip(2), &stderr);
}

#[cfg(unix)]
#[test]
fn non_utf8_command_and_inline_input_are_usage_failures() {
    use std::os::unix::ffi::OsStringExt;

    for (args, message) in [
        (vec![OsString::from_vec(vec![0xff])], "missing command"),
        (
            vec![OsString::from("query"), OsString::from_vec(vec![0xff])],
            "input must be valid UTF-8",
        ),
    ] {
        let output = run_os(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(cli_stdout_v1(&output).is_empty());
        let stderr = cli_stderr_v1(&output);
        assert_eq!(
            stderr.lines().next(),
            Some("cli.error.class=usage"),
            "{stderr}"
        );
        let expected_message = format!("cli.error.message={message}");
        assert_eq!(
            stderr.lines().nth(1),
            Some(expected_message.as_str()),
            "{stderr}"
        );
        assert_usage_block(stderr.lines().skip(2), &stderr);
    }
}

#[test]
fn formatter_failure_is_stderr_only_and_nonzero() {
    let output = run_cli_argv_v1(&["fmt", "let x = ("]);
    assert_eq!(output.status.code(), Some(1));
    assert!(cli_stdout_v1(&output).is_empty());
    let stderr = cli_stderr_v1(&output);
    assert!(stderr.starts_with("cli.error.class=command\nanalysis.status=error\n"));
    assert!(stderr.contains("diagnostic[0].code=surface.parse."));
}

#[test]
fn leading_empty_argv_preserves_the_source_bytes_seen_by_the_binary() {
    let split = run_cli_argv_v1(&["query", "", "totally not a qglang root"]);
    let joined = run_cli_argv_v1(&["query", " totally not a qglang root"]);
    assert_eq!(split.status.code(), Some(1));
    assert_eq!(joined.status.code(), Some(1));
    assert!(
        !split.stderr.is_empty(),
        "the parse error must be rendered; empty stderr would make the byte comparison vacuous"
    );
    assert_eq!(
        split.stdout, joined.stdout,
        "inline source must be byte-exact"
    );
    assert_eq!(
        split.stderr, joined.stderr,
        "diagnostic spans must be byte-exact"
    );
}

#[test]
fn untrusted_command_renders_as_a_single_line_value() {
    let command = "evil\ncli.read_only=false\r\u{1b}[31m\\\u{2028}\u{2029}";
    let output = run_cli_argv_v1(&[command, "x"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = cli_stderr_v1(&output);
    let mut lines = stderr.lines();
    assert_eq!(lines.next(), Some("cli.error.class=usage"));
    assert_eq!(
        lines.next(),
        Some("cli.error.message=unsupported command `evil\\ncli.read_only=false\\r\\x1b[31m\\\\\\u{2028}\\u{2029}`")
    );
    assert_usage_block(lines, &stderr);
    assert!(!stderr.contains('\r'));
    assert!(!stderr.contains('\u{1b}'));
    assert!(!stderr.contains('\u{2028}'));
    assert!(!stderr.contains('\u{2029}'));
}

#[test]
fn untrusted_file_path_renders_as_a_single_line_value() {
    let missing = std::env::temp_dir().join(format!(
        "qglang-cli-absent-{}-{}\ncli.error.class=usage\r\u{1b}\\\u{2028}\u{2029}",
        std::process::id(),
        FILE_ID.fetch_add(1, Ordering::SeqCst)
    ));
    let output = run_os(&[
        OsString::from("query"),
        OsString::from("--file"),
        missing.into_os_string(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = cli_stderr_v1(&output);
    assert_eq!(stderr.lines().count(), 2, "{stderr:?}");
    assert_eq!(stderr.lines().next(), Some("cli.error.class=io"));
    assert!(
        stderr.lines().nth(1).is_some_and(|line| {
            line.starts_with("cli.error.message=failed to open ")
                && line.contains("\\ncli.error.class=usage\\r\\x1b\\\\\\u{2028}\\u{2029}")
        }),
        "{stderr:?}"
    );
    assert!(!stderr.contains('\r'));
    assert!(!stderr.contains('\u{1b}'));
    assert!(!stderr.contains('\u{2028}'));
    assert!(!stderr.contains('\u{2029}'));
}

#[test]
fn diagnostic_message_is_escaped_without_collapsing_the_error_block() {
    let diagnostic = FrontendDiagnosticV2::new(
        FrontendStageV2::Integrity,
        "test.untrusted_message",
        FrontendSeverityV2::Error,
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(0),
            span: SpanV1::point(0),
        },
        vec![],
        "bad\nanalysis.status=complete\r\u{1b}[31m\\\u{2028}\u{2029}",
    );
    let rendered = render_failure_v2(&FrontendFailureV2::one(diagnostic));
    assert_eq!(
        rendered.lines().collect::<Vec<_>>(),
        [
            "analysis.status=error",
            "analysis.diagnostic_count=1",
            "diagnostic[0].stage=integrity",
            "diagnostic[0].code=test.untrusted_message",
            "diagnostic[0].severity=error",
            "diagnostic[0].source_id=0",
            "diagnostic[0].span=0..0",
            "diagnostic[0].message=bad\\nanalysis.status=complete\\r\\x1b[31m\\\\\\u{2028}\\u{2029}",
        ]
    );
    assert!(!rendered.contains('\r'));
    assert!(!rendered.contains('\u{1b}'));
    assert!(!rendered.contains('\u{2028}'));
    assert!(!rendered.contains('\u{2029}'));
}

#[test]
fn benign_command_message_keeps_its_exact_bytes() {
    assert_usage_error(&["é=100%plain"], "unsupported command `é=100%plain`");
}

#[test]
fn binary_diagnostic_esc_is_encoded_but_the_error_block_stays_multiline() {
    let output = run_cli_argv_v1(&["fmt", "from workspace.files\u{1b}"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = cli_stderr_v1(&output);
    assert!(stderr.starts_with("cli.error.class=command\nanalysis.status=error\n"));
    assert_eq!(
        stderr
            .lines()
            .filter(|line| line == &"analysis.status=error")
            .count(),
        1
    );
    assert!(
        stderr
            .lines()
            .any(|line| { line.starts_with("diagnostic[0].message=") && line.contains("\\x1b") }),
        "{stderr:?}"
    );
    assert!(!stderr.contains('\u{1b}'));
}

#[cfg(unix)]
#[test]
fn file_input_accepts_regular_symlink_and_rejects_nonregular_nodes() {
    use std::os::unix::fs::symlink;

    let regular = TempFile::new(b"show capabilities");
    let link = TempFile::new(b"");
    fs::remove_file(&link.path).expect("remove placeholder");
    symlink(&regular.path, &link.path).expect("create regular-file symlink");
    let output = run_os(&[
        OsString::from("query"),
        OsString::from("--file"),
        link.path.clone().into_os_string(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    assert!(cli_stdout_v1(&output).contains("analysis.status=incomplete"));

    for path in [std::env::temp_dir(), PathBuf::from("/dev/null")] {
        let output = run_os_bounded(&[
            OsString::from("query"),
            OsString::from("--file"),
            path.into_os_string(),
        ]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_io_error(&cli_stderr_v1(&output), "input path is not a regular file");
    }

    let fifo = TempFile::new(b"");
    fs::remove_file(&fifo.path).expect("remove placeholder");
    let mkfifo = Command::new("mkfifo")
        .arg(&fifo.path)
        .status()
        .expect("spawn mkfifo");
    assert!(mkfifo.success(), "mkfifo failed: {mkfifo}");
    let output = run_os_bounded(&[
        OsString::from("query"),
        OsString::from("--file"),
        fifo.path.clone().into_os_string(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_io_error(&cli_stderr_v1(&output), "input path is not a regular file");
}
