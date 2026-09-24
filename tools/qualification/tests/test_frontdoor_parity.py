from pathlib import Path
import json
import tempfile
import unittest

from tools.test_frontdoor_parity import built_binaries, usable_binary


class FrontdoorParityArtifactTest(unittest.TestCase):
    def test_build_artifacts_preserve_cargo_selected_target_paths(self):
        cli = "/tmp/target/aarch64-apple-darwin/debug/quanta-gqlang-v2-cli"
        stdio = "/tmp/target/aarch64-apple-darwin/debug/quanta-gqlang-v2-diagnostics-stdio"
        events = [
            {
                "reason": "compiler-artifact",
                "target": {"kind": ["lib"], "name": "ignored-library"},
                "executable": None,
            },
            {
                "reason": "compiler-artifact",
                "target": {"kind": ["bin"], "name": "quanta-gqlang-v2-cli"},
                "executable": cli,
            },
            {
                "reason": "compiler-artifact",
                "target": {
                    "kind": ["bin"],
                    "name": "quanta-gqlang-v2-diagnostics-stdio",
                },
                "executable": stdio,
            },
            {"reason": "build-finished", "success": True},
        ]
        output = b"\n".join(json.dumps(event).encode() for event in events)

        self.assertEqual(
            built_binaries(output),
            {
                "quanta-gqlang-v2-cli": Path(cli),
                "quanta-gqlang-v2-diagnostics-stdio": Path(stdio),
            },
        )

    def test_duplicate_and_relative_binary_artifacts_fail_closed(self):
        artifact = {
            "reason": "compiler-artifact",
            "target": {"kind": ["bin"], "name": "quanta-gqlang-v2-cli"},
            "executable": "/tmp/build/quanta-gqlang-v2-cli",
        }
        line = json.dumps(artifact).encode()
        with self.assertRaisesRegex(ValueError, "duplicate Cargo binary artifact"):
            built_binaries(line + b"\n" + line)
        artifact["executable"] = "debug/quanta-gqlang-v2-cli"
        with self.assertRaisesRegex(ValueError, "relative Cargo binary artifact"):
            built_binaries(json.dumps(artifact).encode())

    def test_missing_and_redirected_binary_paths_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "binary"
            binary.write_bytes(b"fixture")
            redirected = root / "redirected"
            redirected.symlink_to(binary)
            self.assertTrue(usable_binary(binary))
            self.assertFalse(usable_binary(redirected))
            self.assertFalse(usable_binary(root / "absent"))


if __name__ == "__main__":
    unittest.main()
