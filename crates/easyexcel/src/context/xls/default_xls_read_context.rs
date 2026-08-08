//! 对应 Java：`com.alibaba.excel.context.xls.DefaultXlsReadContext`

use std::path::{Path, PathBuf};

use crate::ReadOptions;
use crate::context::analysis_context_impl::AnalysisContextImpl;
use crate::context::read_sheet::ReadSheet;
use crate::context::xls::xls_read_context::XlsReadContext;
use crate::read::holder::xls::xls_read_sheet_holder::XlsReadSheetHolder;
use crate::read::holder::xls::xls_read_workbook_holder::XlsReadWorkbookHolder;
use crate::read::metadata::ReadWorkbook;
use crate::support::ExcelTypeEnum;

/// 对应 Java：`DefaultXlsReadContext extends AnalysisContextImpl implements XlsReadContext`.
#[derive(Debug, Clone)]
pub struct DefaultXlsReadContext {
    inner: AnalysisContextImpl,
    xls_read_workbook_holder: XlsReadWorkbookHolder,
    xls_read_sheet_holder: Option<XlsReadSheetHolder>,
    file: Option<PathBuf>,
    options: ReadOptions,
}
impl DefaultXlsReadContext {
    /// 对应 Java：`DefaultXlsReadContext(ReadWorkbook, ExcelTypeEnum)`.
    #[must_use]
    pub fn new(options: &ReadOptions) -> Self {
        Self {
            inner: AnalysisContextImpl::new(ExcelTypeEnum::Xls, options),
            xls_read_workbook_holder: XlsReadWorkbookHolder::from_options(options),
            xls_read_sheet_holder: None,
            file: None,
            options: options.clone(),
        }
    }

    /// 使用 Java `ReadWorkbook` 与实际格式创建 XLS 读取上下文。
    ///
    /// 对应 Java：`DefaultXlsReadContext(ReadWorkbook, ExcelTypeEnum)`。
    #[must_use]
    pub fn from_read_workbook(
        read_workbook: &ReadWorkbook,
        actual_excel_type: ExcelTypeEnum,
    ) -> Self {
        let options = read_workbook.options().clone();
        Self {
            inner: AnalysisContextImpl::new(actual_excel_type, &options),
            xls_read_workbook_holder: XlsReadWorkbookHolder::from_options(&options),
            xls_read_sheet_holder: None,
            file: read_workbook.file().map(Path::to_path_buf),
            options,
        }
    }

    /// 返回上下文持有的 XLS 输入文件。
    #[must_use]
    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// 返回上下文持有的有效读取选项。
    #[must_use]
    pub const fn options(&self) -> &ReadOptions {
        &self.options
    }

    /// 为内部执行器绑定路径和读取选项。
    ///
    /// Java 从 `ReadWorkbookHolder` 取得同一状态；这是 Rust 路径型扩展入口。
    pub(crate) fn bind_input(&mut self, file: impl Into<PathBuf>, options: ReadOptions) {
        self.file = Some(file.into());
        self.options = options;
    }

    /// 对应 Java：com.alibaba.excel.context.xls.DefaultXlsReadContext。 Selects the current sheet and materializes the typed XLS holder.
    ///
    /// # Errors
    ///
    /// 当 `read_sheet.sheet_no()` 超出 `i32` 范围时返回 `ExcelError::Format`。
    pub fn current_sheet(&mut self, read_sheet: &ReadSheet) -> crate::core::Result<()> {
        self.inner.current_sheet(read_sheet)?;
        let sheet_no = i32::try_from(read_sheet.sheet_no()).map_err(|_| {
            crate::core::ExcelError::Format("sheet index exceeds i32 range".to_owned())
        })?;
        self.xls_read_sheet_holder =
            Some(XlsReadSheetHolder::new(sheet_no, read_sheet.sheet_name()));
        Ok(())
    }

    /// Returns mutable XLS workbook state for record listeners.
    /// 对应 Java：com.alibaba.excel.context.xls.DefaultXlsReadContext。
    pub const fn xls_read_workbook_holder_mut(&mut self) -> &mut XlsReadWorkbookHolder {
        &mut self.xls_read_workbook_holder
    }
}
impl XlsReadContext for DefaultXlsReadContext {
    fn analysis_context_impl(&self) -> &AnalysisContextImpl {
        &self.inner
    }

    fn xls_read_workbook_holder(&self) -> &XlsReadWorkbookHolder {
        &self.xls_read_workbook_holder
    }

    fn xls_read_sheet_holder(&self) -> Option<&XlsReadSheetHolder> {
        self.xls_read_sheet_holder.as_ref()
    }
}
