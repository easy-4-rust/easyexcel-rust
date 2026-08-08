//! 工作表单元格样式上下文。
//!
//! 对应 Java：`com.alibaba.excel` 写入路径的内部样式辅助类型（无直接 Java 类）。

use crate::core::{ExcelCellStyle, ExcelColumn, ExcelFontStyle, ExcelWriteMetadata, WriteCellData};

use crate::write::cell_style::CellStyle;
use crate::write::excel_writer_core::WriteGlobalFlags;

#[derive(Clone, Copy)]
/// 对应 Java：com.alibaba.excel。
pub(crate) struct SheetStyleContext<'a> {
    pub(crate) explicit: Option<&'a CellStyle>,
    pub(crate) metadata: &'a ExcelWriteMetadata,
    pub(crate) is_head: bool,
    pub(crate) global: WriteGlobalFlags,
}

impl<'a> SheetStyleContext<'a> {
    /// 对应 Java：com.alibaba.excel。
    pub(crate) const fn head(
        explicit: &'a CellStyle,
        metadata: &'a ExcelWriteMetadata,
        global: WriteGlobalFlags,
    ) -> Self {
        Self {
            explicit: Some(explicit),
            metadata,
            is_head: true,
            global,
        }
    }

    /// 对应 Java：com.alibaba.excel。
    pub(crate) const fn content(
        explicit: Option<&'a CellStyle>,
        metadata: &'a ExcelWriteMetadata,
        global: WriteGlobalFlags,
    ) -> Self {
        Self {
            explicit,
            metadata,
            is_head: false,
            global,
        }
    }

    /// 对应 Java：com.alibaba.excel。
    pub(crate) const fn column(self, column: &'a ExcelColumn) -> CellFormatContext<'a> {
        let (cell, font) = if self.is_head {
            (
                match column.head_style {
                    Some(style) => Some(style),
                    None => self.metadata.head_style,
                },
                match column.head_font_style {
                    Some(style) => Some(style),
                    None => self.metadata.head_font_style,
                },
            )
        } else {
            (
                match column.content_style {
                    Some(style) => Some(style),
                    None => self.metadata.content_style,
                },
                match column.content_font_style {
                    Some(style) => Some(style),
                    None => self.metadata.content_font_style,
                },
            )
        };
        CellFormatContext {
            explicit: self.explicit,
            cell,
            font,
            handler_cell: None,
            converted_cell: None,
            converted_data_format: None,
            ignore_fill_style: false,
            global: self.global,
        }
    }
}

include!("sheet_style_context/cell_format_context.rs");
