#!/usr/bin/env python3
"""通用 Java→Rust 目录/文件 1:1 对应核对脚本（迁移规范第六节实现）。

规则（与迁移规范一致）：
1. Rust 目录 = Java 包路径去除顶级包名前缀，**保留完整层级**（禁止扁平化）。
2. 文件名：Java PascalCase → Rust snake_case（连写大写如 `URL` → `url`，
   不做逐字母拆分）。
3. 类型名保持 PascalCase 不变（本脚本只核对文件，不核对类型名）。
4. 排除 `package-info.java`；Rust 侧 `mod.rs`/`lib.rs`/测试文件/Rust 独有
   实现文件属于排除项，不计入缺失。

用法：
    python3 scripts/verify-java-rust-layout.py \
        --java-root <Java 包根，如 src/main/java/com/alibaba/excel> \
        --rust-root <Rust crate src 根，如 easyexcel/src>

输出：
    精确匹配 / 文件名匹配但路径不同（扁平化、错位）/ 完全缺失
    存在任一偏差时退出码为 1。
"""
import argparse
import os
import re
import sys
from collections import defaultdict


def to_snake(name: str) -> str:
    """Java PascalCase → Rust snake_case，正确处理连续大写缩写。

    DefaultConverterLoader → default_converter_loader
    URLImageConverter     → url_image_converter（不是 u_r_l_image_converter）
    """
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    s = re.sub(r"([a-z\d])([A-Z])", r"\1_\2", s)
    return s.lower()


def collect_java_files(java_root: str) -> list[str]:
    files = []
    for dirpath, _dirnames, filenames in os.walk(java_root):
        for fname in filenames:
            if fname.endswith(".java") and fname != "package-info.java":
                rel = os.path.relpath(os.path.join(dirpath, fname), java_root)
                files.append(rel)
    return files


def find_by_name(rust_root: str, fname: str) -> list[str]:
    hits = []
    for dirpath, _dirnames, filenames in os.walk(rust_root):
        if fname in filenames:
            hits.append(os.path.relpath(os.path.join(dirpath, fname), rust_root))
    return hits


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-root", required=True, help="Java 包根目录")
    parser.add_argument("--rust-root", required=True, help="Rust crate src 根目录")
    args = parser.parse_args()

    java_files = collect_java_files(args.java_root)
    exact: list[str] = []
    misplaced: list[tuple[str, str, str]] = []  # (java, expected, actual)
    missing: list[tuple[str, str]] = []  # (java, expected)

    for rel in sorted(java_files):
        pkg_dir = os.path.dirname(rel)
        stem = os.path.splitext(os.path.basename(rel))[0]
        expected = os.path.join(args.rust_root, pkg_dir, to_snake(stem) + ".rs")
        if os.path.exists(expected):
            exact.append(rel)
            continue
        hits = find_by_name(args.rust_root, to_snake(stem) + ".rs")
        if hits:
            misplaced.append(
                (rel, os.path.relpath(expected, args.rust_root), hits[0])
            )
        else:
            missing.append((rel, os.path.relpath(expected, args.rust_root)))

    print(f"Java 文件: {len(java_files)}")
    print(f"精确匹配: {len(exact)}")
    print(f"文件名匹配但路径不同: {len(misplaced)}")
    print(f"完全缺失: {len(missing)}")

    if misplaced:
        print("\n===== 文件名匹配但路径不同（扁平化/错位/跨包放置）=====")
        groups: dict[str, list[tuple[str, str]]] = defaultdict(list)
        for _rel, expected, actual in misplaced:
            groups[os.path.dirname(expected)].append(
                (os.path.basename(expected), actual)
            )
        for key in sorted(groups):
            print(f"  期望 {key}/")
            for expected_name, actual in groups[key]:
                print(f"    {expected_name}  →  实际 {actual}")
    if missing:
        print("\n===== 完全缺失 =====")
        for _rel, expected in missing:
            print(f"  {expected}")

    return 1 if (misplaced or missing) else 0


if __name__ == "__main__":
    sys.exit(main())
