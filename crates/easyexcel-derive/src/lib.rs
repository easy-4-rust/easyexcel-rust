//! `EasyExcel` 类型化行映射的派生宏。
//!
//! 宏通过 `easyexcel` 门面重导出，业务代码只需：
//!
//! ```
//! use easyexcel::{ExcelRow, NumberRoundingMode};
//!
//! #[derive(ExcelRow)]
//! #[excel(column_width = 18, head_row_height = 24)]
//! struct OrderRow {
//!     #[excel(value = ["订单", "编号"], index = 0)]
//!     id: String,
//!     #[excel(name = "金额", number_format = "0.00", rounding_mode = "HALF_UP")]
//!     amount: f64,
//! }
//!
//! let schema = OrderRow::schema();
//! assert_eq!(schema[0].head_names, Some(&["订单", "编号"][..]));
//! assert_eq!(schema[1].number_rounding_mode, Some(NumberRoundingMode::HalfUp));
//! ```
//!
//! `ExcelIgnoreUnannotated` 只把 `ExcelProperty` 等价声明视为映射字段：
//!
//! ```
//! use easyexcel::ExcelRow;
//!
//! #[derive(ExcelRow)]
//! #[excel(ignore_unannotated)]
//! struct StrictRow {
//!     #[excel(property)]
//!     included: String,
//!     #[excel(number_format = "0.00")]
//!     ignored_without_property: f64,
//! }
//!
//! assert_eq!(StrictRow::schema().len(), 1);
//! ```
//!
//! 冲突的强制列索引会在编译期拒绝：
//!
//! ```compile_fail
//! use easyexcel::ExcelRow;
//! #[derive(ExcelRow)]
//! struct DuplicateIndex {
//!     #[excel(index = 0)] first: String,
//!     #[excel(index = 0)] second: String,
//! }
//! ```

use proc_macro::TokenStream;

mod annotation;
mod crate_path;
mod expand;

/// 为结构体生成静态 Excel 列元数据以及双向行转换实现。
#[proc_macro_derive(ExcelRow, attributes(excel))]
pub fn derive_excel_row(input: TokenStream) -> TokenStream {
    expand::expand_excel_row_tokens(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
