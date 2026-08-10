import importlib.util
import json
from pathlib import Path

import pytest


SCRIPT = Path(__file__).parents[1] / "materialize_public_api_evidence.py"
SPEC = importlib.util.spec_from_file_location("materialize_public_api_evidence", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


def java_manifest():
    return {
        "types": [
            {
                "id": "java.family.One",
                "kind": "type",
                "owner": "java.family.One",
            }
        ],
        "members": [
            {
                "id": "java.family.One#value()I",
                "kind": "method",
                "name": "value",
                "owner": "java.family.One",
            }
        ],
    }


def candidate_entries(strategy="existing_implementation"):
    return {
        "java.family.One": {
            "java_id": "java.family.One",
            "implementation_strategy": strategy,
            "rust_ids": ["crate:one"],
        },
        "java.family.One#value()I": {
            "java_id": "java.family.One#value()I",
            "implementation_strategy": strategy,
            "rust_ids": ["crate:value"],
        },
    }


def family_template():
    return {
        "id": "family.behavior.v1",
        "kind": "behavior_test",
        "java_owner_prefixes": ["java.family."],
        "expected_java_api_items": 2,
        "commands": [["cargo", "test", "family"]],
        "source_paths": ["src/lib.rs"],
    }


def test_materializes_exact_java_and_rust_ids_with_current_source_hash(tmp_path):
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir()
    source.write_text("pub fn value() -> i32 { 1 }\n", encoding="utf-8")

    record = MODULE.materialize_family(
        family_template(),
        MODULE.java_items(java_manifest()),
        candidate_entries(),
        tmp_path,
    )

    assert record["java_ids"] == ["java.family.One", "java.family.One#value()I"]
    assert record["rust_ids"] == ["crate:one", "crate:value"]
    assert record["source_files"] == [
        {
            "path": "src/lib.rs",
            "sha256": MODULE.sha256_file(source),
        }
    ]


def test_rejects_family_when_any_selected_api_still_needs_implementation(tmp_path):
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir()
    source.write_text("", encoding="utf-8")
    entries = candidate_entries()
    entries["java.family.One#value()I"] = {
        "java_id": "java.family.One#value()I",
        "implementation_strategy": "needs_implementation",
        "rust_ids": [],
    }

    with pytest.raises(ValueError, match="is not implemented"):
        MODULE.materialize_family(
            family_template(),
            MODULE.java_items(java_manifest()),
            entries,
            tmp_path,
        )


def test_rejects_selector_count_drift(tmp_path):
    source = tmp_path / "src" / "lib.rs"
    source.parent.mkdir()
    source.write_text("", encoding="utf-8")
    template = family_template()
    template["expected_java_api_items"] = 3

    with pytest.raises(ValueError, match="selector count changed"):
        MODULE.materialize_family(
            template,
            MODULE.java_items(java_manifest()),
            candidate_entries(),
            tmp_path,
        )


def test_template_tree_hash_includes_nested_catalog_content(tmp_path):
    nested = tmp_path / "nested.json"
    nested.write_text(json.dumps({"family_evidence": []}), encoding="utf-8")
    root = tmp_path / "root.json"
    root.write_text(json.dumps({"include": ["nested.json"]}), encoding="utf-8")
    _, _, _, sources = MODULE.load_template_tree(root)
    before = MODULE.template_tree_sha256(sources, tmp_path)

    nested.write_text(json.dumps({"family_evidence": [], "version": 2}), encoding="utf-8")
    _, _, _, changed_sources = MODULE.load_template_tree(root)
    after = MODULE.template_tree_sha256(changed_sources, tmp_path)

    assert before != after
