//! 对应 Java：`com.alibaba.excel.write.handler.DefaultWriteHandlerLoader`.

use crate::core::WriteHandler;
use crate::core::support::ExcelTypeEnum;

use crate::writer::handler::r#impl::impl_default_row_write_handler::DefaultRowWriteHandler;
use crate::writer::handler::r#impl::impl_dimension_workbook_write_handler::DimensionWorkbookWriteHandler;
use crate::writer::handler::r#impl::impl_fill_style_cell_write_handler::FillStyleCellWriteHandler;
use crate::writer::style::default_style::DefaultStyle;

/// 对应 Java：`DefaultWriteHandlerLoader.loadDefaultHandler(Boolean useDefaultStyle, ExcelTypeEnum excelType)`.
///
/// Returns the Java-equivalent default handler set for each output type.
pub struct DefaultWriteHandlerLoader;

impl DefaultWriteHandlerLoader {
    /// Returns the default XLSX handler list with default style enabled.
    ///
    /// This no-argument form is retained for earlier Rust callers. Use
    /// [`Self::load_default_handler_for`] for Java's complete parameterized
    /// behavior.
    #[must_use]
    pub fn load_default_handler() -> Vec<Box<dyn WriteHandler>> {
        Self::load_default_handler_for(true, ExcelTypeEnum::Xlsx)
    }

    /// 对应 Java：`loadDefaultHandler(Boolean useDefaultStyle, ExcelTypeEnum excelType)`.
    #[must_use]
    pub fn load_default_handler_for(
        use_default_style: bool,
        excel_type: ExcelTypeEnum,
    ) -> Vec<Box<dyn WriteHandler>> {
        let mut handlers: Vec<Box<dyn WriteHandler>> = Vec::new();
        if excel_type == ExcelTypeEnum::Xlsx {
            handlers.push(Box::new(DimensionWorkbookWriteHandler::new()));
        }
        handlers.push(Box::new(DefaultRowWriteHandler::new()));
        handlers.push(Box::new(FillStyleCellWriteHandler::new()));
        if use_default_style && excel_type != ExcelTypeEnum::Csv {
            handlers.push(Box::new(DefaultStyle::new()));
        }
        handlers
    }
}
