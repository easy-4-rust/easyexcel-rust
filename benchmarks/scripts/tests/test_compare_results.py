"""Contract tests for the Java/Rust benchmark comparator."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
MODULE_PATH = ROOT / "benchmarks/scripts/compare_results.py"
SPEC_PATH = ROOT / "benchmarks/spec/benchmark-suite-v1.json"
MODULE_SPEC = importlib.util.spec_from_file_location("compare_results", MODULE_PATH)
assert MODULE_SPEC is not None and MODULE_SPEC.loader is not None
COMPARE = importlib.util.module_from_spec(MODULE_SPEC)
MODULE_SPEC.loader.exec_module(COMPARE)


class CompareResultsContractTest(unittest.TestCase):
    """验证正式矩阵不会因缺样本或错误并发统计而被误判通过。"""

    @classmethod
    def setUpClass(cls) -> None:
        cls.spec = json.loads(SPEC_PATH.read_text(encoding="utf-8"))

    def test_expected_profile_shapes_are_exact(self) -> None:
        expectations = {
            "pr": (28, 28),
            "nightly": (56, 392),
            "release": (104, 2_912),
        }
        for profile, (group_count, sample_count) in expectations.items():
            with self.subTest(profile=profile):
                groups = COMPARE.expected_matrix_groups(self.spec, profile)
                self.assertEqual(group_count, len(groups))
                self.assertEqual(sample_count, sum(groups.values()))

    def test_missing_group_is_rejected(self) -> None:
        failures: list[str] = []
        COMPARE.validate_matrix_completeness({}, self.spec, "pr", failures)
        self.assertTrue(any(value.startswith("missing benchmark group:") for value in failures))

    def test_concurrency_cv_uses_trial_aggregate(self) -> None:
        samples = [
            {"trial": 0, "rows": 100, "wall_time_ns": 1_000_000_000},
            {"trial": 0, "rows": 100, "wall_time_ns": 2_000_000_000},
            {"trial": 1, "rows": 100, "wall_time_ns": 2_000_000_000},
            {"trial": 1, "rows": 100, "wall_time_ns": 1_000_000_000},
        ]
        summary = COMPARE.summarize_concurrent_throughput(samples)
        self.assertIsNotNone(summary)
        self.assertEqual(100.0, summary["median"])
        self.assertEqual(0.0, summary["coefficient_of_variation"])

    def test_bootstrap_ratio_is_deterministic(self) -> None:
        first = COMPARE.bootstrap_median_ratio(
            [100.0] * 7, [100.0] * 7, seed="same-scenario", iterations=100
        )
        second = COMPARE.bootstrap_median_ratio(
            [100.0] * 7, [100.0] * 7, seed="same-scenario", iterations=100
        )
        self.assertEqual(first, second)
        self.assertEqual(1.0, first["median_ratio"])
        self.assertEqual(1.0, first["confidence_lower_bound"])

    def test_bootstrap_ratio_exposes_slow_runtime(self) -> None:
        ratio = COMPARE.bootstrap_median_ratio(
            [80.0] * 7, [100.0] * 7, seed="slow-rust", iterations=100
        )
        self.assertAlmostEqual(0.8, ratio["median_ratio"])
        self.assertAlmostEqual(0.8, ratio["confidence_lower_bound"])

    def test_wrong_spec_and_unknown_git_sha_are_rejected(self) -> None:
        failures: list[str] = []
        result = {
            "environment": {"spec_sha256": "wrong", "git_sha": "unknown"},
            "phase": "matrix",
            "operation": "read",
            "fixture_origin": "rust",
            "input_sha256": "fixture-sha",
        }
        result["implementation"] = "rust"
        COMPARE.validate_result_provenance([result], self.spec, SPEC_PATH, failures)
        self.assertIn("spec SHA mismatch at sample 1", failures)
        self.assertIn("unknown implementation Git SHA at sample 1", failures)

    def test_result_schema_rejects_missing_and_unknown_fields(self) -> None:
        schema = json.loads(
            (ROOT / "benchmarks/spec/benchmark-result-v1.schema.json").read_text(
                encoding="utf-8"
            )
        )
        errors = COMPARE.validate_json_schema({"schema_version": 1, "extra": True}, schema)
        self.assertIn("$: missing required property implementation", errors)
        self.assertIn("$: unexpected property extra", errors)


if __name__ == "__main__":
    unittest.main()
