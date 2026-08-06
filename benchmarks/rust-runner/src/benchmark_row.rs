//! Java/Rust 性能测试共用的四列表格模型。

use chrono::{Duration, NaiveDate};
use easyexcel::ExcelRow;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 与 Java `BenchmarkRow` 语义一致的性能测试行。
#[derive(Debug, Clone, ExcelRow)]
pub(crate) struct BenchmarkRow {
    #[excel(name = "ID", index = 0)]
    pub(crate) id: i64,
    #[excel(name = "Name", index = 1)]
    pub(crate) name: String,
    #[excel(name = "Date", index = 2)]
    pub(crate) date: NaiveDate,
    #[excel(name = "Score", index = 3)]
    pub(crate) score: f64,
}

impl BenchmarkRow {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按统一契约生成第 `id` 行。
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn from_id(id: i64) -> Self {
        let base = NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid benchmark epoch");
        Self {
            id,
            name: format!("row-{id}"),
            date: base + Duration::days(id.rem_euclid(28)),
            score: id as f64 * 0.5,
        }
    }
}
