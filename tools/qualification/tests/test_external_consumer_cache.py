"""Custody checks for the stable-path external-consumer fixture cache."""

import tempfile
import unittest
from pathlib import Path

from tools.test_frontend_external_consumer import (
    FIXTURE_DIR,
    REPOSITORY_ROOT,
    InfrastructureError,
    lock_cache,
    materialize_fixture,
)


class ExternalConsumerCacheTest(unittest.TestCase):
    def test_replay_restores_source_at_the_same_path(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "cache"
            with lock_cache(root, REPOSITORY_ROOT):
                fixture = materialize_fixture(root)
                (fixture / "src" / "main.rs").write_text("fn main() { panic!() }\n")
            with lock_cache(root, REPOSITORY_ROOT):
                replay = materialize_fixture(root)
                self.assertEqual(replay, fixture)
                self.assertEqual(
                    (replay / "src" / "main.rs").read_bytes(),
                    (FIXTURE_DIR / "src" / "main.rs").read_bytes(),
                )

    def test_unowned_or_foreign_cache_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "cache"
            root.mkdir()
            (root / "foreign").write_text("do not adopt")
            with self.assertRaisesRegex(InfrastructureError, "unowned"):
                lock_cache(root, REPOSITORY_ROOT)
            (root / "foreign").unlink()
            with lock_cache(root, REPOSITORY_ROOT):
                pass
            (root / ".owner").write_text("other checkout\n")
            with self.assertRaisesRegex(InfrastructureError, "another checkout"):
                lock_cache(root, REPOSITORY_ROOT)

    def test_second_runner_cannot_interleave_privacy_probes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "cache"
            with lock_cache(root, REPOSITORY_ROOT):
                with self.assertRaisesRegex(InfrastructureError, "another external-consumer gate"):
                    lock_cache(root, REPOSITORY_ROOT)

    def test_redirected_build_cache_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "cache"
            with lock_cache(root, REPOSITORY_ROOT):
                pass
            (root / "build").symlink_to(Path(directory) / "foreign-build")
            with self.assertRaisesRegex(InfrastructureError, "build cache is redirected"):
                lock_cache(root, REPOSITORY_ROOT)


if __name__ == "__main__":
    unittest.main()
