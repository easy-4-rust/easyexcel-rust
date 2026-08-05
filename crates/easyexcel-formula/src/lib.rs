//! Excel 公式解析、求值、动态数组和函数注册表。
//!
//! 公式实现迁自 `easy-4-rust/xls` fork，并通过 `easyexcel-model` 操作统一
//! 工作簿模型，避免 EasyExcel 门面继续依赖完整 `xls` 包。

#![allow(
    missing_docs,
    reason = "迁入的公式引擎仍保留上游语义注释；中文 API 文档按来源矩阵持续补齐"
)]

pub mod formula;

pub use formula::{
    Array, CellRef, Context, Engine, Evaluator, Expr, RecalcReport, RefRange, Value, parse,
    parse_detailed,
};
