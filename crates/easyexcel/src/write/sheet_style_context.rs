//! 工作表单元格样式上下文。
//!
//! 对应 Java：`com.alibaba.excel` 写入路径的内部样式辅助类型（无直接 Java 类）。

use crate::core::{ExcelCellStyle, ExcelColumn, ExcelFontStyle, ExcelWriteMetadata, WriteCellData};

use crate::write::cell_style::CellStyle;
use crate::write::excel_writer_core::WriteGlobalFlags;

#[derive(Clone, Copy)]
pub(crate) struct SheetStyleContext<'a> {
    pub(crate) explicit: Option<&'a CellStyle>,
    pub(crate) metadata: &'a ExcelWriteMetadata,
    pub(crate) is_head: bool,
    pub(crate) global: WriteGlobalFlags,
}

impl<'a> SheetStyleContext<'a> {
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
            global: self.global,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CellFormatContext<'a> {
    pub(crate) explicit: Option<&'a CellStyle>,
    pub(crate) cell: Option<ExcelCellStyle>,
    pub(crate) font: Option<ExcelFontStyle>,
    /// Style contributed by registered `WriteHandler` strategies
    /// (Java `AbstractCellStyleStrategy` merge into `WriteCellData`).
    pub(crate) handler_cell: Option<ExcelCellStyle>,
    /// Style returned by `Converter::convert_to_excel_data`.
    pub(crate) converted_cell: Option<ExcelCellStyle>,
    /// Owned runtime format carried by `WriteCellData::DataFormatData`.
    pub(crate) converted_data_format: Option<&'a str>,
    pub(crate) global: WriteGlobalFlags,
}

impl<'a> CellFormatContext<'a> {
    /// Attaches a strategy-derived cell style (Java `WriteCellStyle.merge`).
    #[must_use]
    pub(crate) const fn with_handler_cell(mut self, handler_cell: Option<ExcelCellStyle>) -> Self {
        self.handler_cell = handler_cell;
        self
    }

    /// Attaches converter-produced style metadata without flattening it into
    /// the scalar value.
    #[must_use]
    pub(crate) fn with_converted_cell(mut self, cell: &'a WriteCellData) -> Self {
        self.converted_cell = cell.write_cell_style().copied();
        self.converted_data_format = cell.data_format_data().and_then(|data| data.format());
        self
    }

    /// 对应 Java：`ignoreFillStyle`: retain non-style write flags while
    /// suppressing explicit, annotation and strategy style materialization.
    pub(crate) const fn without_fill_style(mut self) -> Self {
        self.explicit = None;
        self.cell = None;
        self.font = None;
        self.handler_cell = None;
        self.converted_cell = None;
        self.converted_data_format = None;
        self
    }
}
