from pathlib import Path
import json
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from tools.test_frontdoor_parity import (
    built_binaries,
    decode_line_value,
    parse_cli,
    parse_stdio,
    usable_binary,
)


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


class FrontdoorParityOutputTest(unittest.TestCase):
    env = {"PATH": "/usr/bin"}

    def test_cli_channel_and_schema_mutations_fail_closed(self):
        valid = b"cli.error.class=command\nanalysis.status=error\nanalysis.diagnostic_count=0\n"
        cases = (
            (b"analysis.status=error\n", valid, "exactly one output channel"),
            (b"", valid + b"analysis.status=complete\n", "duplicate CLI output key"),
            (b"", valid + b"extra=bad\n", "CLI output schema drift"),
            (b"", valid + b"not-a-field\n", "non-field CLI output row"),
        )
        for stdout, stderr, message in cases:
            with self.subTest(message=message):
                result = subprocess.CompletedProcess([], 1, stdout=stdout, stderr=stderr)
                with patch("tools.test_frontdoor_parity.run", return_value=result):
                    with self.assertRaisesRegex((RuntimeError, ValueError), message):
                        parse_cli(Path("/unused/cli"), "invalid", self.env)

    def test_cli_escaped_message_is_reversibly_compared(self):
        payload = (
            "cli.error.class=command\nanalysis.status=error\nanalysis.diagnostic_count=1\n"
            "diagnostic[0].stage=surface\n"
            "diagnostic[0].code=surface.parse.unexpected_token\n"
            "diagnostic[0].severity=error\n"
            "diagnostic[0].source_id=0\n"
            "diagnostic[0].span=0..1\n"
            "diagnostic[0].message=bad\\u{2028}\\x1b\\\\tail\n"
        ).encode()
        result = subprocess.CompletedProcess([], 1, stdout=b"", stderr=payload)
        with patch("tools.test_frontdoor_parity.run", return_value=result):
            parsed = parse_cli(Path("/unused/cli"), "invalid", self.env)
        self.assertEqual(parsed["diagnostics"][0]["message"], "bad\u2028\x1b\\tail")
        with self.assertRaisesRegex(ValueError, "unknown CLI line-value escape"):
            decode_line_value("bad\\q")

    def test_stdio_mixed_channel_and_duplicate_json_key_fail_closed(self):
        valid = b'{"id":"parity","ok":true,"method":"v3_diagnostics","analysis_status":"error","diagnostics":[]}\n'
        mixed = subprocess.CompletedProcess([], 0, stdout=valid, stderr=b"unexpected")
        with patch("tools.test_frontdoor_parity.run", return_value=mixed):
            with self.assertRaisesRegex(RuntimeError, "transport failed"):
                parse_stdio(Path("/unused/stdio"), "invalid", self.env)
        duplicate = valid.replace(b'"ok":true,', b'"ok":true,"ok":true,')
        result = subprocess.CompletedProcess([], 0, stdout=duplicate, stderr=b"")
        with patch("tools.test_frontdoor_parity.run", return_value=result):
            with self.assertRaisesRegex(ValueError, "duplicate diagnostics stdio JSON key"):
                parse_stdio(Path("/unused/stdio"), "invalid", self.env)


if __name__ == "__main__":
    unittest.main()
