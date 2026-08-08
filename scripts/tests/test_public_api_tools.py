#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def load_module(name: str):
    path = ROOT / "scripts" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


java_api = load_module("generate_java_public_api")
rust_api = load_module("generate_rust_public_api")
parity = load_module("verify_public_api_parity")


class JavaPublicApiTest(unittest.TestCase):
    def test_parse_class_and_overloads(self):
        block = """Compiled from \"Demo.java\"
public class com.example.Demo {
  public static final int VALUE = 7;
    descriptor: I
  public com.example.Demo();
    descriptor: ()V
  public java.lang.String value(java.lang.String);
    descriptor: (Ljava/lang/String;)Ljava/lang/String;
}""".splitlines()
        type_item, members = java_api.parse_class(block)
        self.assertEqual("com.example.Demo", type_item["id"])
        self.assertEqual(
            [
                "com.example.Demo#FIELD:VALUEI",
                "com.example.Demo#<init>()V",
                "com.example.Demo#value(Ljava/lang/String;)Ljava/lang/String;",
            ],
            [item["id"] for item in members],
        )


class RustPublicApiTest(unittest.TestCase):
    def test_classifies_signatures(self):
        self.assertEqual("module", rust_api.api_kind("pub mod easyexcel"))
        self.assertEqual("struct", rust_api.api_kind("pub struct easyexcel::Demo"))
        self.assertEqual("function", rust_api.api_kind("pub fn easyexcel::run()"))


class ParityVerifierTest(unittest.TestCase):
    def setUp(self):
        self.java = {
            "types": [{"id": "com.example.Demo"}],
            "members": [{"id": "com.example.Demo#run()V"}],
        }
        self.rust = {
            "packages": [
                {
                    "snapshots": [
                        {"items": [{"id": "easyexcel:abc"}]},
                        {"items": [{"id": "easyexcel:abc"}]},
                    ]
                }
            ]
        }

    def test_unmapped_is_incomplete(self):
        mapping = parity.skeleton(self.java, "java", "rust")
        report = parity.validate(self.java, self.rust, mapping)
        self.assertEqual(2, report["error_count"])
        self.assertEqual({"unmapped": 2}, report["status"])

    def test_verified_requires_all_evidence(self):
        entry = {
            "java_id": "com.example.Demo",
            "status": "verified",
            "rust_ids": ["easyexcel:abc"],
            "compile_probes": ["probe"],
            "behavior_tests": ["behavior"],
            "java_golden": ["golden"],
        }
        second = {**entry, "java_id": "com.example.Demo#run()V"}
        java_ids = [entry["java_id"], second["java_id"]]
        source_files = [{"path": "probe.rs", "sha256": "0" * 64}]
        catalog = {
            "evidence": [
                {
                    "id": "probe",
                    "kind": "compile_probe",
                    "java_ids": java_ids,
                    "rust_ids": ["easyexcel:abc"],
                    "profiles": ["stable-default-features", "stable-all-features"],
                    "source_files": source_files,
                    "commands": [["compile"]],
                },
                {
                    "id": "behavior",
                    "kind": "behavior_test",
                    "java_ids": java_ids,
                    "rust_ids": ["easyexcel:abc"],
                    "source_files": source_files,
                    "commands": [["behavior"]],
                },
                {
                    "id": "golden",
                    "kind": "java_golden",
                    "java_ids": java_ids,
                    "rust_ids": ["easyexcel:abc"],
                    "source_files": source_files,
                    "commands": [["golden"]],
                },
            ]
        }
        results = {
            "results": [
                {
                    "evidence_id": evidence_id,
                    "status": "passed",
                    "commands": [
                        {"argv": [evidence_id], "status": "passed", "exit_code": 0}
                    ],
                }
                for evidence_id in ("probe", "behavior", "golden")
            ]
        }
        results["results"][0]["commands"][0]["argv"] = ["compile"]
        report = parity.validate(
            self.java,
            self.rust,
            {"entries": [entry, second]},
            evidence_catalog=catalog,
            evidence_results=results,
        )
        self.assertEqual([], report["errors"])

    def test_verified_rejects_unbound_or_unexecuted_evidence(self):
        entry = {
            "java_id": "com.example.Demo",
            "status": "verified",
            "rust_ids": ["easyexcel:abc"],
            "compile_probes": ["probe"],
            "behavior_tests": ["behavior"],
            "java_golden": ["golden"],
        }
        catalog = {
            "evidence": [
                {
                    "id": evidence_id,
                    "kind": kind,
                    "java_ids": [],
                    "rust_ids": [],
                    "source_files": [{"path": "missing", "sha256": "0" * 64}],
                }
                for evidence_id, kind in (
                    ("probe", "compile_probe"),
                    ("behavior", "behavior_test"),
                    ("golden", "java_golden"),
                )
            ]
        }
        report = parity.validate(
            self.java,
            self.rust,
            {"entries": [entry, {**entry, "java_id": "com.example.Demo#run()V"}]},
            evidence_catalog=catalog,
        )
        self.assertGreater(report["error_count"], 0)
        self.assertTrue(any("not bound" in error for error in report["errors"]))
        self.assertTrue(any("no execution result" in error for error in report["errors"]))


if __name__ == "__main__":
    unittest.main()
