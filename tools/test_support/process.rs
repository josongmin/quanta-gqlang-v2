//! Bounded, concurrently drained child processes for shipped-binary tests.
//! Cleanup kills and reaps the direct child; it does not promise descendant cleanup.

use std::io::{self, Read, Write};
use std::process::{Child, Output};
use std::sync::mpsc;
use std::time::{Duration, Instant};

struct ReapedChild(Child);

impl Drop for ReapedChild {
    fn drop(&mut self) {
        // Also clean up if polling or a pipe operation fails before completion.
        drop(self.0.kill());
        drop(self.0.wait());
    }
}

fn drain(pipe: Option<impl Read + Send + 'static>) -> mpsc::Receiver<io::Result<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    drop(std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = match pipe {
            Some(mut pipe) => pipe.read_to_end(&mut bytes).map(|_| bytes),
            None => Ok(bytes),
        };
        drop(sender.send(result));
    }));
    receiver
}

fn pipe_error(error: mpsc::RecvTimeoutError) -> io::Error {
    let kind = match error {
        mpsc::RecvTimeoutError::Timeout => io::ErrorKind::TimedOut,
        mpsc::RecvTimeoutError::Disconnected => io::ErrorKind::BrokenPipe,
    };
    io::Error::new(kind, error)
}

pub(crate) fn wait_bounded(
    child: Child,
    input: Option<Vec<u8>>,
    timeout: Duration,
) -> io::Result<Output> {
    let mut child = ReapedChild(child);
    let started = Instant::now();
    let stdout = drain(child.0.stdout.take());
    let stderr = drain(child.0.stderr.take());
    let stdin = child.0.stdin.take();
    let (sender, written) = mpsc::channel();
    drop(std::thread::spawn(move || {
        let result = match (stdin, input) {
            (Some(mut pipe), Some(bytes)) => pipe.write_all(&bytes),
            (_, None) => Ok(()),
            (None, Some(_)) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "stdin is not piped",
            )),
        };
        drop(sender.send(result));
    }));

    let remaining = || timeout.saturating_sub(started.elapsed());
    let timed_out = || {
        io::Error::new(
            io::ErrorKind::TimedOut,
            "binary child exceeded its watchdog",
        )
    };
    let status = loop {
        if let Some(status) = child.0.try_wait()? {
            break status;
        }
        if remaining().is_zero() {
            return Err(timed_out());
        }
        std::thread::sleep(Duration::from_millis(5).min(remaining()));
    };
    // Pipe EOF is bounded too: an inherited writer cannot extend the watchdog.
    let stdout = stdout.recv_timeout(remaining()).map_err(pipe_error)??;
    let stderr = stderr.recv_timeout(remaining()).map_err(pipe_error)??;
    written.recv_timeout(remaining()).map_err(pipe_error)??;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}
