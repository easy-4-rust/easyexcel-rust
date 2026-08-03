//! 图片像素布局计算。
//!
//! 对应 Java：`com.alibaba.excel` 写入路径的内部图片布局辅助类型（无直接 Java 类）。

use std::collections::HashMap;

use crate::core::{ExcelColumn, ExcelWriteMetadata, Result, WriteHandler};

use crate::write::excel_writer_core::{
    collect_handler_content_row_height, collect_handler_head_row_height, excel_column_width_pixels,
    excel_row_height_pixels, to_column,
};
use crate::write::write_options::WriteOptions;

#[derive(Debug)]
pub(crate) struct ImageLayout {
    pub(crate) column_widths: HashMap<u16, u32>,
    pub(crate) head_rows: u32,
    pub(crate) head_row_height: u32,
    pub(crate) content_row_height: u32,
}

impl Default for ImageLayout {
    fn default() -> Self {
        Self {
            column_widths: HashMap::new(),
            head_rows: 0,
            head_row_height: 20,
            content_row_height: 20,
        }
    }
}

impl ImageLayout {
    /// Builds image pixel layout from explicit options, annotation widths, and
    /// registered column-width strategies
    /// (Java `SimpleColumnWidthStyleStrategy` / `AbstractColumnWidthStyleStrategy`).
    ///
    /// Precedence: explicit `WriteOptions` widths win; registered handler
    /// strategies overwrite annotation/`@ColumnWidth` values for schema
    /// columns. Columns outside the typed schema keep Excel default `64` px.
    pub(crate) fn new(
        columns: &[(usize, usize, &'static ExcelColumn)],
        options: &WriteOptions,
        metadata: &ExcelWriteMetadata,
        head_rows: u32,
        handlers: &[Box<dyn WriteHandler>],
    ) -> Result<Self> {
        let mut column_widths = HashMap::new();
        // Explicit WriteOptions widths win (same precedence as sheet write path).
        for (column, width) in &options.column_widths {
            column_widths.insert(*column, excel_column_width_pixels(*width));
        }
        // Annotation `@ColumnWidth` / type-level column width.
        for (physical_index, _, column) in columns {
            let physical_index = to_column(*physical_index)?;
            if column_widths.contains_key(&physical_index) {
                continue;
            }
            if let Some(width) = column.column_width.or(metadata.column_width) {
                column_widths.insert(physical_index, excel_column_width_pixels(width));
            }
        }
        // Registered handler strategies override annotation widths so image
        // pixel layout matches `apply_handler_column_widths` (Java
        // `SimpleColumnWidthStyleStrategy` / `setColumnWidth` after annotations).
        for (physical_index, _, _) in columns {
            let physical_index = to_column(*physical_index)?;
            if options
                .column_widths
                .iter()
                .any(|(explicit, _)| *explicit == physical_index)
            {
                continue;
            }
            for handler in handlers {
                if let Some(width) = handler.style_column_width(usize::from(physical_index)) {
                    column_widths.insert(physical_index, excel_column_width_pixels(width));
                }
            }
        }
        Ok(Self {
            column_widths,
            head_rows,
            head_row_height: excel_row_height_pixels(
                collect_handler_head_row_height(handlers).or(metadata.head_row_height),
            ),
            content_row_height: excel_row_height_pixels(
                collect_handler_content_row_height(handlers).or(metadata.content_row_height),
            ),
        })
    }

    pub(crate) fn column_width(&self, column: u16) -> u32 {
        self.column_widths.get(&column).copied().unwrap_or(64)
    }

    pub(crate) const fn row_height(&self, row: u32) -> u32 {
        if row < self.head_rows {
            self.head_row_height
        } else {
            self.content_row_height
        }
    }
}
