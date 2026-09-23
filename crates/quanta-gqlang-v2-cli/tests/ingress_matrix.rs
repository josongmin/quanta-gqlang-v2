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

fn assert_usage_error(args: &[&str], message: &str) {
    let output = run_cli_argv_v1(args);
    assert_eq!(output.status.code(), Some(2), "argv {args:?}");
    assert!(cli_stdout_v1(&output).is_empty(), "argv {args:?}");
    let stderr = cli_stderr_v1(&output);
    assert!(stderr.contains("cli.error.class=usage"), "{stderr}");
    assert!(stderr.contains(message), "{stderr}");
    assert_eq!(stderr.matches("cli.usage=").count(), 1, "{stderr}");
}

#[test]
fn help_aliases_are_stdout_only_successes() {
    for token in ["help", "--help"] {
        let output = run_cli_argv_v1(&[token]);
        assert_eq!(output.status.code(), Some(0), "{token}");
        assert!(cli_stderr_v1(&output).is_empty(), "{token}");
        let stdout = cli_stdout_v1(&output);
        assert!(stdout.starts_with("cli.usage="), "{stdout}");
        assert_eq!(stdout.matches("cli.usage=").count(), 1);
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
    assert!(stderr.contains("cli.error.class=io"), "{stderr}");
    assert!(stderr.contains("failed to open"), "{stderr}");
    assert!(!stderr.contains("cli.usage="), "{stderr}");

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
        assert!(stderr.contains("cli.error.class=io"), "{stderr}");
        assert!(stderr.contains("failed to read"), "{stderr}");
        assert!(!stderr.contains("cli.usage="), "{stderr}");
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
    assert!(
        stderr.contains("input file must be valid UTF-8"),
        "{stderr}"
    );
    assert_eq!(stderr.matches("cli.usage=").count(), 1);
}

#[cfg(unix)]
#[test]
fn non_utf8_command_and_inline_input_are_usage_failures() {
    use std::os::unix::ffi::OsStringExt;

    for args in [
        vec![OsString::from_vec(vec![0xff])],
        vec![OsString::from("query"), OsString::from_vec(vec![0xff])],
    ] {
        let output = run_os(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(cli_stdout_v1(&output).is_empty());
        let stderr = cli_stderr_v1(&output);
        assert!(stderr.contains("cli.error.class=usage"), "{stderr}");
        assert_eq!(stderr.matches("cli.usage=").count(), 1);
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
