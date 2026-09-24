#![expect(
    clippy::expect_used,
    reason = "binary ingress tests require successful process and fixture plumbing"
)]

mod support;

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

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
        assert_io_error(&stderr, "failed to read");
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
    assert_eq!(
        split.stdout, joined.stdout,
        "inline source must be byte-exact"
    );
    assert_eq!(
        split.stderr, joined.stderr,
        "diagnostic spans must be byte-exact"
    );
}
