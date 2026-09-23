from pathlib import Path
import json
import unittest

from tools.test_frontdoor_parity import built_binaries


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


if __name__ == "__main__":
    unittest.main()
