import importlib.util
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "apply_public_api_evidence.py"
SPEC = importlib.util.spec_from_file_location("apply_public_api_evidence", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


def test_verifies_only_when_all_three_evidence_kinds_cover_mapped_rust_ids():
    mapping = {
        "entries": [
            {
                "java_id": "java:one",
                "status": "candidate",
                "rust_ids": ["rust:one"],
                "compile_probes": [],
                "behavior_tests": [],
                "java_golden": [],
            },
            {
                "java_id": "java:two",
                "status": "candidate",
                "rust_ids": ["rust:two"],
                "compile_probes": [],
                "behavior_tests": [],
                "java_golden": [],
            },
        ]
    }
    catalog = {
        "evidence": [
            {
                "id": kind,
                "kind": kind,
                "java_ids": ["java:one", "java:two"],
                "rust_ids": ["rust:one"],
            }
            for kind in ("compile_probe", "behavior_test", "java_golden")
        ]
    }

    result = MODULE.apply(mapping, catalog)

    assert result["entries"][0]["status"] == "verified"
    assert result["entries"][1]["status"] == "candidate"
