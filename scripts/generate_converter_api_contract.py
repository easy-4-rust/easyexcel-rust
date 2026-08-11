#!/usr/bin/env python3
"""Generate converter_api.contract.json from Java source files.

Extracts the public API contract of each Converter<T> implementation in the
`com.alibaba.excel.converters` package tree.  The output JSON mirrors the
structure used by `excel_writer_lifecycle.contract.json` and is consumed by
the public-api-evidence materialization pipeline.

Usage:
    python3 scripts/generate_converter_api_contract.py \
        --java-root /path/to/easyexcel \
        --rust-root . \
        --output tests/easyexcel-test/tests/golden/converter_api.contract.json
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


# ---------------------------------------------------------------------------
# Regex patterns for Java source parsing
# ---------------------------------------------------------------------------

# Matches: public class FooBarConverter implements Converter<SomeType>
CLASS_RE = re.compile(
    r"public\s+class\s+(?P<name>\w+)\s+"
    r"implements\s+(?P<iface>Converter|NullableObjectConverter)\s*<\s*(?P<type_param>[^>]+)\s*>"
)

# Matches: public interface NullableObjectConverter<T> extends Converter<T>
INTERFACE_RE = re.compile(
    r"public\s+interface\s+(?P<name>\w+)\s*(?:<[^>]*>)?\s+extends\s+Converter\s*<\s*(?P<type_param>[^>]+)\s*>"
)

# Matches: public Class<?> supportJavaTypeKey() { return Foo.class; }
#   or:    public Class<BigDecimal> supportJavaTypeKey() { return BigDecimal.class; }
#   or:    return byte[].class; / Byte[].class;
SUPPORT_JAVA_RE = re.compile(
    r"public\s+Class\s*<[^>]*>\s+supportJavaTypeKey\s*\(\s*\)\s*\{[^}]*?return\s+(?P<cls>[\w.\[\]]+)\.class\s*;[^}]*\}",
    re.DOTALL,
)

# Matches: public CellDataTypeEnum supportExcelTypeKey() { return CellDataTypeEnum.XXX; }
SUPPORT_EXCEL_RE = re.compile(
    r"public\s+CellDataTypeEnum\s+supportExcelTypeKey\s*\(\s*\)\s*\{[^}]*?return\s+CellDataTypeEnum\.(?P<enum>\w+)\s*;[^}]*\}",
    re.DOTALL,
)

# Matches convertToJavaData method signature
CONVERT_JAVA_RE = re.compile(
    r"public\s+(?P<ret>\S+)\s+convertToJavaData\s*\("
    r"(?P<params>[^)]*)\)",
    re.DOTALL,
)

# Matches convertToExcelData method signature
CONVERT_EXCEL_RE = re.compile(
    r"public\s+(?P<ret>\S+)\s+convertToExcelData\s*\("
    r"(?P<params>[^)]*)\)",
    re.DOTALL,
)

# Import line for class short-name resolution
IMPORT_RE = re.compile(r"import\s+([\w.]+)\s*;")


def short_class(fqcn: str) -> str:
    """Return the simple class name from a fully-qualified name."""
    return fqcn.rsplit(".", 1)[-1]


def resolve_imports(source: str) -> dict[str, str]:
    """Build a simple-name -> fqcn map from import statements."""
    mapping: dict[str, str] = {}
    for m in IMPORT_RE.finditer(source):
        fqcn = m.group(1)
        mapping[short_class(fqcn)] = fqcn
    return mapping


def parse_converter_file(path: Path) -> dict[str, Any] | None:
    """Parse a single Java file and return its converter contract, or None."""
    source = path.read_text(encoding="utf-8")
    imports = resolve_imports(source)

    # Try class match first
    m = CLASS_RE.search(source)
    if m is None:
        # Check if it's an interface (NullableObjectConverter)
        im = INTERFACE_RE.search(source)
        if im is not None:
            return {
                "class_name": im.group("name"),
                "kind": "interface",
                "extends": "Converter",
                "type_parameter": im.group("type_param").strip(),
                "file": str(path),
            }
        return None

    class_name = m.group("name")
    iface = m.group("iface")
    type_param = m.group("type_param").strip()

    record: dict[str, Any] = {
        "class_name": class_name,
        "kind": "class",
        "implements": iface,
        "type_parameter": type_param,
        "file": str(path),
    }

    # supportJavaTypeKey
    sj = SUPPORT_JAVA_RE.search(source)
    if sj:
        raw_cls = sj.group("cls")
        # Resolve to fqcn if possible
        short = short_class(raw_cls)
        if "." not in raw_cls and short in imports:
            record["support_java_type_key"] = imports[short]
        else:
            record["support_java_type_key"] = raw_cls

    # supportExcelTypeKey
    se = SUPPORT_EXCEL_RE.search(source)
    if se:
        record["support_excel_type_key"] = se.group("enum")

    # convertToJavaData
    cj = CONVERT_JAVA_RE.search(source)
    if cj:
        record["convert_to_java_data"] = _normalize_signature(
            cj.group("ret"), cj.group("params"), imports
        )

    # convertToExcelData
    ce = CONVERT_EXCEL_RE.search(source)
    if ce:
        record["convert_to_excel_data"] = _normalize_signature(
            ce.group("ret"), ce.group("params"), imports
        )

    return record


def _normalize_signature(
    ret: str, params_raw: str, imports: dict[str, str]
) -> dict[str, Any]:
    """Normalize a method signature into a structured form."""
    params = []
    for part in params_raw.split(","):
        part = part.strip()
        if not part:
            continue
        tokens = part.split()
        if len(tokens) >= 2:
            ptype = tokens[0]
            pname = tokens[-1]
            params.append({"type": ptype, "name": pname})
        elif len(tokens) == 1:
            params.append({"type": tokens[0], "name": ""})
    return {"return_type": ret, "parameters": params}


def scan_java_converters(java_root: Path) -> list[dict[str, Any]]:
    """Scan all Java converter source files and return parsed contracts."""
    converters_dir = (
        java_root
        / "easyexcel-core"
        / "src"
        / "main"
        / "java"
        / "com"
        / "alibaba"
        / "excel"
        / "converters"
    )
    if not converters_dir.is_dir():
        print(f"ERROR: converters directory not found: {converters_dir}", file=sys.stderr)
        sys.exit(1)

    results: list[dict[str, Any]] = []
    for java_file in sorted(converters_dir.rglob("*.java")):
        parsed = parse_converter_file(java_file)
        if parsed is not None:
            # Add relative path from converters root
            try:
                rel = java_file.relative_to(converters_dir)
                parent = str(rel.parent).replace("/", ".")
                # For files in root converters dir, parent will be "." -> empty
                parsed["package_path"] = "" if parent == "." else parent
            except ValueError:
                pass
            results.append(parsed)
    return results


def build_contract(
    java_converters: list[dict[str, Any]], rust_root: Path
) -> dict[str, Any]:
    """Build the final contract JSON structure."""
    # Separate interfaces and classes
    interfaces = [c for c in java_converters if c.get("kind") == "interface"]
    classes = [c for c in java_converters if c.get("kind") == "class"]

    # Group classes by package_path
    by_package: dict[str, list[dict[str, Any]]] = {}
    for cls in classes:
        pkg = cls.get("package_path", "")
        by_package.setdefault(pkg, []).append(cls)

    # Count by support_excel_type_key
    excel_type_counts: dict[str, int] = {}
    for cls in classes:
        ek = cls.get("support_excel_type_key", "UNKNOWN")
        excel_type_counts[ek] = excel_type_counts.get(ek, 0) + 1

    contract: dict[str, Any] = {
        "authority": "com.alibaba:easyexcel:4.0.3",
        "converter_interface": "com.alibaba.excel.converters.Converter",
        "nullable_object_converter_interface": "com.alibaba.excel.converters.NullableObjectConverter",
        "total_converter_classes": len(classes),
        "total_interfaces": len(interfaces),
        "excel_type_distribution": excel_type_counts,
        "converters": [],
    }

    for cls in sorted(classes, key=lambda c: c["class_name"]):
        pkg_suffix = cls.get("package_path", "")
        pkg = f"com.alibaba.excel.converters.{pkg_suffix}" if pkg_suffix else "com.alibaba.excel.converters"
        entry: dict[str, Any] = {
            "class_name": cls["class_name"],
            "package": pkg,
            "implements": cls.get("implements", "Converter"),
            "type_parameter": cls.get("type_parameter", ""),
            "support_java_type_key": cls.get("support_java_type_key", ""),
            "support_excel_type_key": cls.get("support_excel_type_key", ""),
        }
        if "convert_to_java_data" in cls:
            entry["convert_to_java_data"] = cls["convert_to_java_data"]
        if "convert_to_excel_data" in cls:
            entry["convert_to_excel_data"] = cls["convert_to_excel_data"]
        contract["converters"].append(entry)

    # Add interface entries
    for iface in sorted(interfaces, key=lambda c: c["class_name"]):
        pkg_suffix = iface.get("package_path", "")
        pkg = f"com.alibaba.excel.converters.{pkg_suffix}" if pkg_suffix else "com.alibaba.excel.converters"
        contract["converters"].append(
            {
                "class_name": iface["class_name"],
                "package": pkg,
                "kind": "interface",
                "extends": iface.get("extends", "Converter"),
                "type_parameter": iface.get("type_parameter", ""),
            }
        )

    return contract


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate converter_api.contract.json from Java source"
    )
    parser.add_argument(
        "--java-root",
        type=Path,
        required=True,
        help="Root of the Java easyexcel repository",
    )
    parser.add_argument(
        "--rust-root",
        type=Path,
        default=Path.cwd(),
        help="Root of the Rust easyexcel-rust repository",
    )
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Output path for converter_api.contract.json",
    )
    args = parser.parse_args()

    java_root = args.java_root.resolve()
    rust_root = args.rust_root.resolve()
    output = args.output.resolve()

    print(f"Scanning Java converters in: {java_root}")
    java_converters = scan_java_converters(java_root)
    print(f"Found {len(java_converters)} converter entries")

    contract = build_contract(java_converters, rust_root)

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(contract, ensure_ascii=False, indent="\t") + "\n",
        encoding="utf-8",
    )
    print(f"Wrote {output} ({len(contract['converters'])} entries)")

    # Summary
    classes = [c for c in java_converters if c.get("kind") == "class"]
    print(f"  Classes: {len(classes)}")
    print(f"  Interfaces: {len(java_converters) - len(classes)}")
    for ek, count in sorted(
        contract.get("excel_type_distribution", {}).items()
    ):
        print(f"  Excel type {ek}: {count}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
