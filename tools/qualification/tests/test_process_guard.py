"""The local gate runner must bound a stuck child and preserve normal output."""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.process_guard import ProcessTimeoutError, run_bounded


def popen_after_child_ready(pid_file: Path, ready_at=None):
    """Start the test parent before its short gate timeout begins."""
    real_popen = subprocess.Popen

    def start(*args, **kwargs):
        process = real_popen(*args, **kwargs)
        deadline = time.monotonic() + 5
        while not pid_file.exists() and process.poll() is None and time.monotonic() < deadline:
            time.sleep(0.01)
        if not pid_file.exists():
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=1)
            raise AssertionError("test child did not signal readiness")
        if ready_at is not None:
            ready_at.append(time.monotonic())
        return process

    return start


class ProcessGuardTest(unittest.TestCase):
    def test_interrupted_gate_kills_and_reaps_its_process_group(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            started = []
            real_popen = subprocess.Popen

            def interrupted_popen(*args, **kwargs):
                process = real_popen(*args, **kwargs)
                started.append(process)
                process.communicate = lambda **_kwargs: (_ for _ in ()).throw(KeyboardInterrupt)
                return process

            try:
                with patch("tools.process_guard.subprocess.Popen", side_effect=interrupted_popen):
                    with self.assertRaises(KeyboardInterrupt):
                        run_bounded(
                            [sys.executable, "-c", "import signal; signal.pause()"],
                            cwd=Path(directory),
                            timeout_s=5,
                        )
                self.assertEqual(len(started), 1)
                self.assertIsNotNone(started[0].poll(), "interrupted gate left its child running")
            finally:
                for process in started:
                    if process.poll() is None:
                        os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=1)

    def test_captured_stdin_and_exit_status(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = run_bounded(
                [sys.executable, "-c", "import sys; print(sys.stdin.read().upper())"],
                cwd=Path(directory),
                timeout_s=5,
                input_data="qglang",
                capture_output=True,
                text=True,
            )
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "QGLANG\n")

    def test_stuck_child_exits_at_the_gate_timeout(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(ProcessTimeoutError):
                run_bounded(
                    [sys.executable, "-c", "import signal; signal.pause()"],
                    cwd=Path(directory),
                    timeout_s=0.1,
                )

    def test_timeout_does_not_wait_for_escaped_pipe_holder(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            pid_file = Path(directory) / "escaped.pid"
            parent = (
                "import pathlib, signal, subprocess, sys; "
                "child = subprocess.Popen([sys.executable, '-c', "
                "'import time; time.sleep(1.5)'], start_new_session=True); "
                "pathlib.Path(%r).write_text(str(child.pid)); signal.pause()"
            ) % str(pid_file)
            try:
                ready_at = []
                with patch(
                    "tools.process_guard.subprocess.Popen",
                    side_effect=popen_after_child_ready(pid_file, ready_at),
                ):
                    with self.assertRaises(ProcessTimeoutError):
                        run_bounded(
                            [sys.executable, "-c", parent],
                            cwd=Path(directory),
                            timeout_s=0.3,
                            capture_output=True,
                        )
                self.assertTrue(pid_file.exists(), "escaped pipe holder did not start")
                self.assertLess(time.monotonic() - ready_at[0], 0.9)
            finally:
                if pid_file.exists():
                    try:
                        os.kill(int(pid_file.read_text()), signal.SIGKILL)
                    except ProcessLookupError:
                        pass

    def test_timeout_kills_child_in_the_same_process_group(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            pid_file = Path(directory) / "group-child.pid"
            child = "import os, signal; print(os.getpid(), flush=True); signal.pause()"
            parent = (
                "import pathlib, signal, subprocess, sys\n"
                "child = subprocess.Popen([sys.executable, '-c', %r], "
                "stdout=subprocess.PIPE, text=True)\n"
                "pathlib.Path(%r).write_text(child.stdout.readline())\n"
                "signal.pause()\n"
            ) % (child, str(pid_file))
            try:
                with patch(
                    "tools.process_guard.subprocess.Popen",
                    side_effect=popen_after_child_ready(pid_file),
                ):
                    with self.assertRaises(ProcessTimeoutError):
                        run_bounded(
                            [sys.executable, "-c", parent],
                            cwd=Path(directory),
                            timeout_s=0.5,
                        )
                self.assertTrue(pid_file.exists(), "same-group child did not start")
                pid = int(pid_file.read_text())
                status = subprocess.run(
                    ["ps", "-o", "stat=", "-p", str(pid)],
                    capture_output=True, text=True, check=False,
                )
                self.assertTrue(
                    status.returncode != 0 or status.stdout.strip().startswith("Z"),
                    f"the gate left a same-group child running: {status.stdout!r}",
                )
            finally:
                if pid_file.exists():
                    try:
                        os.kill(int(pid_file.read_text()), signal.SIGKILL)
                    except ProcessLookupError:
                        pass


if __name__ == "__main__":
    unittest.main()
