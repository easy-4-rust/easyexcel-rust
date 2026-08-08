import importlib.util
import json
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "run_public_api_evidence.py"
SPEC = importlib.util.spec_from_file_location("run_public_api_evidence", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


def test_runs_shared_command_once_and_attests_each_evidence(tmp_path):
    command = ["python3", "-c", "print('ok')"]
    catalog_path = tmp_path / "catalog.json"
    catalog_path.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "evidence": [
                    {"id": "first", "commands": [command]},
                    {"id": "second", "commands": [command]},
                ],
            }
        ),
        encoding="utf-8",
    )

    report, passed = MODULE.run(catalog_path, tmp_path)

    assert passed
    assert [item["status"] for item in report["results"]] == ["passed", "passed"]
    assert report["results"][0]["commands"][0] == report["results"][1]["commands"][0]


def test_rejects_evidence_without_commands(tmp_path):
    catalog_path = tmp_path / "catalog.json"
    catalog_path.write_text(
        json.dumps({"schema_version": 1, "evidence": [{"id": "missing"}]}),
        encoding="utf-8",
    )

    report, passed = MODULE.run(catalog_path, tmp_path)

    assert not passed
    assert report["results"][0]["status"] == "invalid"


def test_recursively_loads_includes_relative_to_each_catalog(tmp_path):
    nested = tmp_path / "catalogs" / "nested"
    nested.mkdir(parents=True)
    (nested / "leaf.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "evidence": [
                    {
                        "id": "leaf",
                        "commands": [["python3", "-c", "print('leaf')"]],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    (tmp_path / "catalogs" / "child.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "include": ["nested/leaf.json"],
                "evidence": [
                    {
                        "id": "child",
                        "commands": [["python3", "-c", "print('child')"]],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    root = tmp_path / "root.json"
    root.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "include": ["catalogs/child.json"],
                "evidence": [
                    {
                        "id": "root",
                        "commands": [["python3", "-c", "print('root')"]],
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    report, passed = MODULE.run(root, tmp_path)

    assert passed
    assert [item["evidence_id"] for item in report["results"]] == [
        "root",
        "child",
        "leaf",
    ]
