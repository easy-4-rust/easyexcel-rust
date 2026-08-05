//! 通过真实运行 `easyexcel-demo-fill` 二进制验证模板填充主流程。
//!
//! 二进制在临时工作目录下运行（其内部把产物写入 `target/` 子目录），
//! 验证退出码与填充产物存在。

use std::process::Command;

#[test]
fn demo_fill_binary_generates_filled_workbook() {
    let dir = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_easyexcel-demo-fill"))
        .current_dir(dir.path())
        .output()
        .expect("spawn");

    assert!(output.status.success(), "stderr: {:?}", output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("模板:"), "stdout: {stdout}");
    assert!(stdout.contains("输出:"), "stdout: {stdout}");

    let filled = dir.path().join("target/demo-fill-output.xlsx");
    assert!(filled.is_file(), "filled output missing");
    let bytes = std::fs::read(&filled).expect("read filled");
    assert_eq!(&bytes[..2], b"PK", "not an xlsx zip");
}
