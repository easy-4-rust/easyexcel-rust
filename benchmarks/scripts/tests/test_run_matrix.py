"""Contract tests for benchmark process and temporary-directory isolation."""

from __future__ import annotations

import argparse
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[3]
MODULE_PATH = ROOT / "benchmarks/scripts/run_matrix.py"
MODULE_SPEC = importlib.util.spec_from_file_location("run_matrix", MODULE_PATH)
assert MODULE_SPEC is not None and MODULE_SPEC.loader is not None
RUN_MATRIX = importlib.util.module_from_spec(MODULE_SPEC)
MODULE_SPEC.loader.exec_module(RUN_MATRIX)


def arguments(output_dir: Path) -> argparse.Namespace:
    """创建只包含命令构造所需字段的参数对象。"""
    return argparse.Namespace(
        spec=ROOT / "benchmarks/spec/benchmark-suite-v1.json",
        rust_bin=Path("/tmp/rust-runner"),
        java_bin=Path("/tmp/java"),
        java_xms="512m",
        java_xmx="4g",
        java_classpath="classes",
        java_git_sha="java-sha",
        output_dir=output_dir,
    )


class RunMatrixContractTest(unittest.TestCase):
    """验证每个被测进程拥有隔离且可观测的临时目录。"""

    def test_java_command_pins_process_temp_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            temp_dir = Path(directory) / "tmp"
            command = RUN_MATRIX.runner_command(
                "java",
                arguments(Path(directory)),
                {"id": "xlsx-event-read"},
                100_000,
                1,
                Path("input.xlsx"),
                None,
                temp_dir=temp_dir,
            )
        self.assertIn(f"-Djava.io.tmpdir={temp_dir}", command)

    @mock.patch.object(RUN_MATRIX.subprocess, "run")
    def test_invoke_exports_cross_runtime_temp_environment(
        self, run: mock.Mock
    ) -> None:
        run.return_value = subprocess.CompletedProcess(["runner"], 0, "{}\n", "")
        with tempfile.TemporaryDirectory() as directory:
            temp_dir = Path(directory)
            RUN_MATRIX.invoke(["runner"], None, measured=False, temp_dir=temp_dir)
        environment = run.call_args.kwargs["env"]
        self.assertEqual(str(temp_dir), environment["TMPDIR"])
        self.assertEqual(str(temp_dir), environment["TMP"])
        self.assertEqual(str(temp_dir), environment["TEMP"])

    @mock.patch.object(RUN_MATRIX, "invoke")
    def test_fixture_origins_do_not_share_worker_directories(
        self, invoke: mock.Mock
    ) -> None:
        invoke.return_value = {
            "success": True,
            "correctness": {},
        }
        scenario = {
            "id": "xlsx-event-read",
            "format": "xlsx",
            "operation": "read",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = root / "fixture.xlsx"
            fixture.write_bytes(b"fixture")
            args = arguments(root / "results")
            RUN_MATRIX.run_worker(
                "rust", args, scenario, 10, 1, 2, 0, "java", fixture,
                False, "cold", 0,
            )
        temp_dir = invoke.call_args.kwargs["temp_dir"]
        self.assertIn("/java/rust-worker-0/tmp", str(temp_dir))

    @mock.patch.object(RUN_MATRIX, "verify_written_output")
    @mock.patch.object(RUN_MATRIX, "run_worker")
    def test_verified_sample_output_is_removed_after_cross_runtime_reread(
        self, run_worker: mock.Mock, verify_written_output: mock.Mock
    ) -> None:
        scenario = {
            "id": "xlsx-stream-write",
            "format": "xlsx",
            "operation": "write",
        }
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "output.xlsx"
            output.write_bytes(b"verified workbook")
            run_worker.return_value = {
                "_output_path": str(output),
                "correctness": {},
                "success": True,
                "errors": 0,
            }
            verify_written_output.return_value = (10, "checksum")

            results = RUN_MATRIX.run_group(
                "rust",
                arguments(Path(directory) / "results"),
                scenario,
                10,
                1,
                0,
                None,
                None,
                True,
            )

            self.assertFalse(output.exists())
            self.assertEqual(10, results[0]["correctness"]["observed_rows"])
            self.assertEqual("checksum", results[0]["correctness"]["checksum"])
            self.assertTrue(results[0]["correctness"]["rereadable"])
            self.assertNotIn("_output_path", results[0])


if __name__ == "__main__":
    unittest.main()
