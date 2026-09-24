"""Bound a local gate command and terminate its whole process group on timeout.

Cargo and `just` spawn child processes. Killing only the immediate process can
leave a build or test running after its gate has reported failure. This module
is POSIX-only, matching the local macOS/Linux qualification driver.
"""

from __future__ import annotations

import os
import signal
import subprocess
from pathlib import Path
from typing import IO, Mapping, Optional, Sequence, Union


class ProcessTimeoutError(RuntimeError):
    def __init__(self, command: Sequence[str], timeout_s: float) -> None:
        self.command = tuple(command)
        self.timeout_s = timeout_s
        super().__init__("%s timed out after %g s" % (" ".join(command), timeout_s))


def run_bounded(
    command: Sequence[str],
    *,
    cwd: Path,
    timeout_s: float,
    env: Optional[Mapping[str, str]] = None,
    input_data: Optional[Union[bytes, str]] = None,
    capture_output: bool = False,
    text: bool = False,
    output_file: Optional[IO[bytes]] = None,
) -> subprocess.CompletedProcess:
    if timeout_s <= 0:
        raise ValueError("timeout_s must be positive")
    if capture_output and output_file is not None:
        raise ValueError("capture_output and output_file are mutually exclusive")
    process = subprocess.Popen(
        list(command),
        cwd=str(cwd),
        env=env,
        stdin=subprocess.PIPE if input_data is not None else None,
        stdout=subprocess.PIPE if capture_output else output_file,
        stderr=subprocess.PIPE if capture_output else subprocess.STDOUT if output_file else None,
        text=text,
        start_new_session=True,
    )
    try:
        stdout, stderr = process.communicate(input=input_data, timeout=timeout_s)
    except BaseException as error:
        # A caller interruption must clean up the detached process group too.
        # Otherwise Ctrl-C can leave `just`, Cargo, and their children running.
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        # A descendant can create its own session yet inherit our output
        # pipes. A second communicate() would then wait for that descendant
        # despite the immediate process group having been killed.
        for stream in (process.stdin, process.stdout, process.stderr):
            if stream is not None:
                try:
                    stream.close()
                except (OSError, ValueError):
                    pass
        try:
            process.wait(timeout=1)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=1)
        if isinstance(error, subprocess.TimeoutExpired):
            raise ProcessTimeoutError(command, timeout_s) from error
        raise
    return subprocess.CompletedProcess(list(command), process.returncode, stdout, stderr)
