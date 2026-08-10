use crate::core::{ExcelCellStyle, WriteCellStyle, WriteFont};
use crate::write::sheet_style_context::CellFormatContext;

/// `AutoStreaming` journal 中相对基础 Sheet 样式的最终单元格样式增量。
///
/// 该对象只保存 Handler、Converter 和 `ignoreFillStyle` 在运行期产生的变化；
/// Schema、Sheet 及全局样式在晋升重放时由原配置重建，避免按单元格复制静态样式。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct JournalCellStyle {
    pub(crate) ignore_fill_style: bool,
    pub(crate) handler_cell: Option<ExcelCellStyle>,
    pub(crate) handler_font: Option<WriteFont>,
    pub(crate) converted_cell: Option<WriteCellStyle>,
    pub(crate) converted_data_format: Option<String>,
}

impl JournalCellStyle {
    pub(crate) fn from_context(context: &CellFormatContext<'_>) -> Option<Self> {
        if !context.ignore_fill_style
            && context.handler_cell.is_none()
            && context.handler_font.is_none()
            && context.converted_cell.is_none()
            && context.converted_data_format.is_none()
        {
            return None;
        }
        Some(Self {
            ignore_fill_style: context.ignore_fill_style,
            handler_cell: context.handler_cell,
            handler_font: context.handler_font.clone(),
            converted_cell: context.converted_cell.cloned(),
            converted_data_format: context.converted_data_format.map(str::to_owned),
        })
    }

    pub(crate) fn apply<'a>(&'a self, mut base: CellFormatContext<'a>) -> CellFormatContext<'a> {
        if self.ignore_fill_style {
            return base.without_fill_style();
        }
        base.handler_cell = self.handler_cell;
        base.handler_font = self.handler_font.clone();
        base.converted_cell = self.converted_cell.as_ref();
        base.converted_data_format = self.converted_data_format.as_deref();
        base
    }
}
