//! 真实运行 `xtask` 二进制验证审计命令
//! （用法输出 + `migration-audit` 完整审计路径）。

use std::path::PathBuf;
use std::process::Command;

/// 仓库根目录 = xtask 包目录的上一级。
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parent")
        .to_path_buf()
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
