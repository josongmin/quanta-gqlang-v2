#![expect(
    clippy::expect_used,
    reason = "process-boundary tests use expect for fixture diagnostics"
)]

#[path = "../../../tools/test_support/process.rs"]
mod process;

#[cfg(unix)]
mod unix {
    use std::io;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    #[test]
    fn timed_out_child_is_killed_and_reaped() {
        let child = Command::new("sh")
            .args(["-c", "exec sleep 30"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn hanging fixture");
        let pid = child.id();
        let started = Instant::now();
        let error = super::process::wait_bounded(child, None, Duration::from_millis(100))
            .expect_err("hanging fixture must hit watchdog");
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(5));
        let probe = Command::new("/bin/kill")
            .args(["-0", &pid.to_string()])
            .output()
            .expect("probe timed-out child");
        assert!(
            !probe.status.success(),
            "timed-out child remains observable"
        );
    }

    #[test]
    fn drains_both_streams_beyond_pipe_capacity() {
        let child = Command::new("sh")
            .args([
                "-c",
                "dd if=/dev/zero bs=65536 count=4; dd if=/dev/zero bs=65536 count=4 >&2",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn large-output fixture");
        let output = super::process::wait_bounded(child, None, Duration::from_secs(10))
            .expect("concurrent drains must allow large-output child to finish");
        assert!(output.status.success());
        assert_eq!(output.stdout.len(), 4 * 65536);
        assert!(output.stderr.len() >= 4 * 65536);
    }

    #[test]
    fn delivers_exact_stdin_beyond_pipe_capacity() {
        let input: Vec<u8> = (0..=255).cycle().take(4 * 65536 + 17).collect();
        let child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn stdin echo fixture");
        let output =
            super::process::wait_bounded(child, Some(input.clone()), Duration::from_secs(10))
                .expect("writer and drain must complete without pipe deadlock");
        assert!(output.status.success());
        assert_eq!(
            output.stdout.len(),
            input.len(),
            "stdin must arrive through EOF"
        );
        assert!(output.stdout == input, "stdin must arrive byte-for-byte");
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn blocked_stdin_writer_cannot_extend_watchdog() {
        let child = Command::new("sh")
            .args(["-c", "exec sleep 30"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn fixture that never reads stdin");
        let pid = child.id();
        let started = Instant::now();
        let error = super::process::wait_bounded(
            child,
            Some(vec![0x5a; 4 * 1024 * 1024]),
            Duration::from_millis(100),
        )
        .expect_err("an unread full stdin pipe must still hit the watchdog");
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(5));
        let probe = Command::new("/bin/kill")
            .args(["-0", &pid.to_string()])
            .output()
            .expect("probe child with blocked stdin writer");
        assert!(
            !probe.status.success(),
            "unread-stdin child remains observable"
        );
    }
}
