//! 通过真实运行 `easyexcel-demo-read` 二进制验证读取主流程。
//!
//! 先用 easyexcel 库生成样例 XLSX，再作为子进程运行二进制读取，
//! 验证退出码与输出行数（对应 Java demo 模块的 main 入口可执行性验证）。

use std::process::Command;

use chrono::NaiveDateTime;
use easyexcel::{EasyExcel, ExcelRow};

/// 与 demo-read 相同的演示行模型。
#[derive(Debug, Clone, ExcelRow)]
struct DemoRow {
    #[excel(name = "名称", index = 0)]
    name: String,
    #[excel(name = "日期", index = 1)]
    date: NaiveDateTime,
    #[excel(name = "数值", index = 2)]
    amount: f64,
}

#[test]
fn demo_read_binary_reads_generated_xlsx() {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = dir.path().join("demo-read.xlsx");

    let date = NaiveDateTime::parse_from_str("2024-06-01 12:00:00", "%Y-%m-%d %H:%M:%S")
        .expect("valid date");
    EasyExcel::write::<DemoRow>(&source)
        .sheet("数据")
        .do_write([
            DemoRow {
                name: "项目0".to_owned(),
                date,
                amount: 0.5,
            },
            DemoRow {
                name: "项目1".to_owned(),
                date,
                amount: 1.5,
            },
        ])
        .expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_easyexcel-demo-read"))
        .arg(&source)
        .output()
        .expect("spawn");

    assert!(output.status.success(), "stderr: {:?}", output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("读取 2 行"), "stdout: {stdout}");
    assert!(stdout.contains("项目0"), "stdout: {stdout}");
    assert!(stdout.contains("项目1"), "stdout: {stdout}");
}
