//! 与文件格式无关的工作簿模型门面。
//!
//! 这里重导出 [`easyexcel_model`] 的核心类型；这些名称与基础 crate 中的类型
//! 完全相同，不引入包装层或额外转换成本。

pub use easyexcel_model::{
    Cell, CellAddress, CellError, CellRange, CellValue, DateSystem, Error, Result, Sheet, Workbook,
    addr, dates, error, numfmt, styles, value,
};
