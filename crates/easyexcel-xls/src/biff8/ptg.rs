//! BIFF8 公式 RPN（Ptg 令牌）编码器。
//!
//! 对应 Java：`org.apache.poi.hssf.record.formula.*`（FORMULA 记录 rgce 编码）。
//! 字节布局参考：`[MS-XLS] 2.5.198`（`Ptg` 系列）、Apache POI `ss/formula/ptg`、
//! xlwt `ExcelFormulaParser`（经 `Excel` / `LibreOffice` 实测验证）。
//!
//! 支持：A1 风格引用（含 `$` 绝对/相对）、区域引用、算术/比较/文本运算符、
//! 内建函数（257 个，`[MS-XLS] 2.5.198.7` 索引）、字符串/布尔/错误常量、
//! 百分比、一元正负号、空参数（tMissArg），以及工作簿内 3D 单元格/区域引用。

use easyexcel_io::Error as ExcelError;

mod biff8_link_table;
pub(crate) use biff8_link_table::Biff8LinkTable;

include!("ptg/builtin_functions_to_parser.rs");
include!("ptg/parser_impl_to_tests.rs");
