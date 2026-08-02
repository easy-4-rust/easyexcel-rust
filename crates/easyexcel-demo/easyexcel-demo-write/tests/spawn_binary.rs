//! 通过真实运行 `easyexcel-demo-write` 二进制验证主流程
//! （对应 Java demo 模块的 main 入口可执行性验证）。

use std::process::Command;

/// 将二进制作为子进程运行，写入临时路径，验证退出码与产物。
#[test]
fn demo_write_binary_generates_xlsx() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = dir.path().join("demo-write.xlsx");

    let status = Command::new(env!("CARGO_BIN_EXE_easyexcel-demo-write"))
        .arg(&output)
        .output()
        .expect("spawn");

    assert!(status.status.success(), "stderr: {:?}", status.stderr);
    let stdout = String::from_utf8_lossy(&status.stdout);
    assert!(stdout.contains("已写入 5 行"), "stdout: {stdout}");
    assert!(output.is_file(), "output file missing");
    let bytes = std::fs::read(&output).expect("read output");
    assert_eq!(&bytes[..2], b"PK", "not an xlsx zip");
}
