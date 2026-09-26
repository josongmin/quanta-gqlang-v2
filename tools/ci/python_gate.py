#!/usr/bin/env python3
"""Run a repository Python gate without interpreter startup hooks.

Invoke with ``python3 -I -S``. User-site dependencies are added only after
startup, so PYTHONPATH, .pth files, and sitecustomize cannot preempt a gate.
"""

from __future__ import annotations

import runpy
import os
import pwd
import site
import sys
from pathlib import Path


def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit("python gate: expected a repository script")
    root = Path(__file__).resolve().parents[2]
    script = (root / sys.argv[1]).resolve()
    if script == root or root not in script.parents or not script.is_file():
        raise SystemExit("python gate: script must be a repository file")
    for key in ("PYTHONPATH", "PYTHONHOME", "PYTHONUSERBASE"):
        os.environ.pop(key, None)
    os.environ["HOME"] = pwd.getpwuid(os.getuid()).pw_dir
    site.USER_BASE = None
    site.USER_SITE = None
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(root))
    sys.path.append(site.getusersitepackages())
    sys.argv = [str(script), *sys.argv[2:]]
    runpy.run_path(str(script), run_name="__main__")


if __name__ == "__main__":
    main()
