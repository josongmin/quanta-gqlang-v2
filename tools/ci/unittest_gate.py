#!/usr/bin/env python3
"""Run a Python unittest discovery rail and reject an empty suite."""

from __future__ import annotations

import argparse
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("start_dir")
    parser.add_argument("pattern")
    args = parser.parse_args()
    suite = unittest.defaultTestLoader.discover(args.start_dir, pattern=args.pattern)
    selected = suite.countTestCases()
    if selected == 0:
        print("unittest gate: no tests discovered in %s (%s)" % (
            args.start_dir, args.pattern), file=sys.stderr
        )
        return 2
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    if result.testsRun != selected:
        print("unittest gate: selected=%d executed=%d" % (
            selected, result.testsRun), file=sys.stderr
        )
        return 2
    print("unittest gate: selected=%d executed=%d" % (selected, result.testsRun))
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
