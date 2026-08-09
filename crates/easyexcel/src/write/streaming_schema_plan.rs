//! XLSX 流式写入的一次性 schema 与 Handler 分派计划。

use crate::core::{ExcelColumn, ExcelRow, Result, WriteHandler};
use crate::write::excel_writer_core::selected_columns;
use crate::write::write_options::WriteOptions;

/// 在进入行循环前预计算不会随行变化的 schema/分派信息。
///
/// 对应 Java：无直接对应对象；Rust 性能扩展。它把列过滤、schema 下标和
/// Handler 上下文需求从百万行循环中移到 Sheet 初始化阶段。
pub(crate) struct StreamingSchemaPlan {
    columns: Vec<(usize, usize, &'static ExcelColumn)>,
    date_formats: Vec<(String, String)>,
    selected_schema_indexes: Option<Vec<usize>>,
    requires_handler_context: bool,
}

impl StreamingSchemaPlan {
    /// 为指定行类型和 Sheet 写入选项编译计划。
    pub(crate) fn compile<T>(
        options: &WriteOptions,
        handlers: &[Box<dyn WriteHandler>],
    ) -> Result<Self>
    where
        T: ExcelRow,
    {
        let columns = selected_columns(T::schema(), options)?;
        let selected_schema_indexes = (!T::schema().is_empty()).then(|| {
            columns
                .iter()
                .map(|(_, schema_index, _)| *schema_index)
                .collect()
        });
        let date_formats = columns
            .iter()
            .map(|(_, _, column)| {
                (
                    easyexcel_format::excel_date_format_code(
                        column.effective_date_time_format(),
                        "yyyy-mm-dd",
                    ),
                    easyexcel_format::excel_date_format_code(
                        column.effective_date_time_format(),
                        "yyyy-mm-dd hh:mm:ss",
                    ),
                )
            })
            .collect();
        let requires_handler_context = handlers.iter().any(|handler| {
            handler.requires_row_context() || handler.requires_cell_context()
        });
        Ok(Self {
            columns,
            date_formats,
            selected_schema_indexes,
            requires_handler_context,
        })
    }

    /// 返回物理列、schema 下标与列元数据。
    #[must_use]
    pub(crate) fn columns(&self) -> &[(usize, usize, &'static ExcelColumn)] {
        &self.columns
    }

    /// 返回与 [`Self::columns`] 一一对应的 Excel 日期/日期时间格式代码。
    #[must_use]
    pub(crate) fn date_formats(&self) -> &[(String, String)] {
        &self.date_formats
    }

    /// 返回传给派生转换器的 schema 下标过滤器。
    #[must_use]
    pub(crate) fn selected_schema_indexes(&self) -> Option<&[usize]> {
        self.selected_schema_indexes.as_deref()
    }

    /// 返回是否必须构造 Java 兼容的 Row/Cell Handler 上下文。
    #[must_use]
    pub(crate) const fn requires_handler_context(&self) -> bool {
        self.requires_handler_context
    }
}
