//! 真实运行 `xtask` 二进制验证审计命令
//! （用法输出 + `migration-audit` 完整审计路径 + 各类 file-map 异常场景）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 仓库根目录 = xtask 包目录的上一级。
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parent")
        .to_path_buf()
}

/// 在指定目录运行 xtask。
fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn")
}

/// 在临时目录中构造 `docs/data/migration/file-map.csv`。
fn write_map(dir: &Path, content: &str) {
    let map_dir = dir.join("docs/data/migration");
    fs::create_dir_all(&map_dir).expect("create dirs");
    fs::write(map_dir.join("file-map.csv"), content).expect("write map");
}

#[test]
fn xtask_usage_without_args_exits_2() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(repo_root())
        .output()
        .expect("spawn");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("usage:"), "stderr: {stderr}");
}

#[test]
fn xtask_migration_audit_runs_full_audit() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(repo_root())
        .arg("migration-audit")
        .output()
        .expect("spawn");
    // 审计可能因磁盘状态返回非 0；重要的是完整执行了审计逻辑
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file-map rows:"), "stdout: {stdout}");
    assert!(
        stdout.contains("migration-audit ok") || stdout.contains("missing rust files"),
        "stdout: {stdout}"
    );
}

#[test]
fn xtask_migration_audit_strict_accepts_flag_alias() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(repo_root())
        .arg("--strict")
        .output()
        .expect("spawn");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // 无论 strict 审计通过与否，都完整执行了 strict 审计逻辑
    assert!(
        stdout.contains("migration-audit ok") || stderr.contains("strict audit"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn xtask_missing_map_file_fails() {
    // 对应 Java：审计入口对缺失映射文件的错误处理
    let dir = tempfile::tempdir().expect("tempdir");
    let output = run_in(dir.path(), &["migration-audit"]);
    assert!(!output.status.success(), "must fail: {output:?}");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("missing docs/data/migration/file-map.csv"),
        "{text}"
    );
}

#[test]
fn xtask_empty_line_and_bad_csv_row_fail() {
    // 空行跳过 + 列数不足的坏行报错
    let dir = tempfile::tempdir().expect("tempdir");
    write_map(
        dir.path(),
        "java,rust,capability,status,note,phase\n\nshort-row\n",
    );
    let output = run_in(dir.path(), &["migration-audit"]);
    assert!(!output.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains("bad csv row"), "{text}");
}

#[test]
fn xtask_duplicate_java_file_fails() {
    // 同一 java 文件出现两行 → 报错
    let dir = tempfile::tempdir().expect("tempdir");
    write_map(
        dir.path(),
        "java,rust,capability,status,note,phase\n\
         A.java,rust_a.rs,full,complete,note,phase1\n\
         A.java,rust_b.rs,full,complete,note,phase1\n",
    );
    let output = run_in(dir.path(), &["migration-audit"]);
    assert!(!output.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains("duplicate java_file"), "{text}");
}

#[test]
fn xtask_missing_rust_and_shared_rust_warn() {
    // 共享 rust 文件（警告）+ rust 文件缺失（报错）+ 未知状态（_ => 分支）
    let dir = tempfile::tempdir().expect("tempdir");
    write_map(
        dir.path(),
        "java,rust,capability,status,note,phase\n\
         A.java,shared.rs,full,complete,note,phase1\n\
         B.java,shared.rs,full,complete,note,phase1\n\
         C.java,does-not-exist.rs,full,planned,note,phase2\n\
         D.java,,full,weird,note,phase2\n",
    );
    let output = run_in(dir.path(), &["migration-audit"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("warn: shared rust_file"), "{stderr}");
    assert!(
        stderr.contains("missing rust file: does-not-exist.rs"),
        "{stderr}"
    );
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains("rust targets missing on disk"), "{text}");
}

#[test]
fn xtask_strict_accepts_all_complete_rows() {
    // strict 模式：全部行 complete → 通过（覆盖 unfinished==0 分支）
    let dir = tempfile::tempdir().expect("tempdir");
    write_map(
        dir.path(),
        "java,rust,capability,note,phase,status\n\
         A.java,,full,note,phase1,complete\n",
    );
    let output = run_in(dir.path(), &["--strict"]);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("migration-audit ok (strict=true)"),
        "{stdout}"
    );
}

#[test]
fn xtask_strict_rejects_unfinished_rows() {
    // strict 模式：存在非 complete/ignore/handle/excluded 行 → 报错
    let dir = tempfile::tempdir().expect("tempdir");
    write_map(
        dir.path(),
        "java,rust,capability,status,note,phase\n\
         A.java,,full,planned,note,phase2\n",
    );
    let output = run_in(dir.path(), &["--strict"]);
    assert!(!output.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains("strict audit"), "{text}");
}
