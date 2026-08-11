//! 写上下文实现。
//!
//! 对应 Java：`com.alibaba.excel.context.WriteContextImpl`

use crate::ConverterRegistry;
use crate::ExcelWriteHeadProperty;
use crate::Holder;
use crate::WriteSheetContext;
use crate::WriteWorkbookContext;
use crate::context::write_context::{WriteContext, WriteContextHolder, WriteContextHolderState};
use std::path::{Path, PathBuf};

/// 对应 Java：`WriteContextImpl implements WriteContext`.
///
/// Java owns POI workbook state; Rust exposes path and holder mirrors for
/// writer facades that delegate to `rust_xlsxwriter`.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteContextImpl {
    /// Output path. (Java `WriteWorkbookHolder.file`)
    path: PathBuf,
    /// Workbook-level handler context. (Java `WriteWorkbookHolder`)
    workbook_context: WriteWorkbookContext,
    /// Active sheet handler context. (Java `WriteSheetHolder`)
    sheet_context: Option<WriteSheetContext>,
    /// Active table index when writing table content. (Java `WriteTableHolder.tableNo`)
    table_no: Option<i32>,
    /// Resolved state of the current workbook/sheet/table holder.
    current_holder_state: WriteContextHolderState,
}
impl WriteContextImpl {
    /// 对应 Java：com.alibaba.excel.context.WriteContextImpl。 Creates a write context bound to an output path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            workbook_context: WriteWorkbookContext::new(&path),
            path,
            sheet_context: None,
            table_no: None,
            current_holder_state: WriteContextHolderState::default(),
        }
    }

    /// 对应 Java：com.alibaba.excel.context.WriteContextImpl。 Returns the output path. (Java `WriteWorkbookHolder.getFile()`)
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 对应 Java：com.alibaba.excel.context.WriteContextImpl。 Returns the workbook-level handler context.
    #[must_use]
    pub fn workbook_context(&self) -> &WriteWorkbookContext {
        &self.workbook_context
    }

    /// 对应 Java：com.alibaba.excel.context.WriteContextImpl。 Returns the active sheet handler context, if any.
    #[must_use]
    pub fn sheet_context(&self) -> Option<&WriteSheetContext> {
        self.sheet_context.as_ref()
    }

    /// Returns the active table index, if any.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.WriteContextImpl。
    pub const fn table_no(&self) -> Option<i32> {
        self.table_no
    }

    /// Returns the resolved current holder state.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.WriteContextImpl。
    pub const fn current_holder_state(&self) -> &WriteContextHolderState {
        &self.current_holder_state
    }

    /// 对应 Java：com.alibaba.excel.context.WriteContextImpl。 Replaces the resolved current holder state.
    pub fn set_current_holder_state(&mut self, state: WriteContextHolderState) {
        self.current_holder_state = state;
    }

    /// 对应 Java：com.alibaba.excel.context.WriteContextImpl。 Updates the active sheet context. (Java `WriteContextImpl` sheet switch)
    pub fn set_sheet_context(&mut self, sheet_name: impl Into<String>) {
        self.sheet_context = Some(WriteSheetContext::new(sheet_name));
        self.current_holder_state.holder_type = Holder::Sheet;
    }

    /// Updates the active table index. (Java `WriteContextImpl` table switch)
    /// 对应 Java：com.alibaba.excel.context.WriteContextImpl。
    pub const fn set_table_no(&mut self, table_no: Option<i32>) {
        self.table_no = table_no;
        self.current_holder_state.holder_type = if table_no.is_some() {
            Holder::Table
        } else if self.sheet_context.is_some() {
            Holder::Sheet
        } else {
            Holder::Workbook
        };
    }

    /// 选择或创建当前工作表。对应 Java：`WriteContext#currentSheet`。
    pub fn current_sheet(
        &mut self,
        write_sheet: &crate::write::metadata::WriteSheet,
        _write_type: crate::WriteType,
    ) {
        self.set_sheet_context(write_sheet.sheet_name.clone());
        self.table_no = None;
    }

    /// 选择当前表格。对应 Java：`WriteContext#currentTable`。
    pub const fn current_table(&mut self, write_table: &crate::WriteTable) {
        self.set_table_no(Some(write_table.table_no));
    }

    /// 返回工作簿 Holder 的后端中立视图。
    #[must_use]
    pub const fn write_workbook_holder(&self) -> &WriteWorkbookContext {
        &self.workbook_context
    }

    /// 返回当前工作表 Holder 的后端中立视图。
    #[must_use]
    pub const fn write_sheet_holder(&self) -> Option<&WriteSheetContext> {
        self.sheet_context.as_ref()
    }

    /// 返回当前表格编号。
    #[must_use]
    pub const fn write_table_holder(&self) -> Option<i32> {
        self.table_no
    }

    /// Java 已废弃 `getCurrentSheet()` 兼容入口。
    #[must_use]
    pub const fn get_current_sheet(&self) -> Option<&WriteSheetContext> {
        self.sheet_context.as_ref()
    }

    /// Java 已废弃 `needHead()` 兼容入口。
    #[must_use]
    pub const fn need_head(&self) -> bool {
        self.current_holder_state.need_head
    }

    /// Java 已废弃 `getOutputStream()` 的路径后端中立表示。
    #[must_use]
    pub const fn get_output_stream(&self) -> Option<&[u8]> {
        None
    }

    /// Java 已废弃 `getWorkbook()` 的后端中立工作簿视图。
    #[must_use]
    pub const fn get_workbook(&self) -> &WriteWorkbookContext {
        &self.workbook_context
    }

    /// 初始化当前 Holder 的表头元数据。实际行写出由资源拥有型 writer 执行。
    /// 对应 Java：`WriteContextImpl#initHead`。
    pub fn init_head(&mut self, property: &ExcelWriteHeadProperty) {
        if self.current_holder_state.need_head && property.has_head() {
            self.current_holder_state.excel_write_head_property = property.clone();
        }
    }
}
impl WriteContext for WriteContextImpl {
    fn current_write_holder(&self) -> &dyn WriteContextHolder {
        self
    }
}
impl WriteContextHolder for WriteContextImpl {
    fn path(&self) -> &Path {
        &self.path
    }

    fn workbook_context(&self) -> Option<&WriteWorkbookContext> {
        Some(&self.workbook_context)
    }

    fn sheet_context(&self) -> Option<&WriteSheetContext> {
        self.sheet_context.as_ref()
    }

    fn table_no(&self) -> Option<i32> {
        self.table_no
    }

    fn holder_type(&self) -> Holder {
        self.current_holder_state.holder_type
    }

    fn excel_write_head_property(&self) -> &ExcelWriteHeadProperty {
        &self.current_holder_state.excel_write_head_property
    }

    fn converter_map(&self) -> &ConverterRegistry {
        &self.current_holder_state.converter_map
    }

    fn need_head(&self) -> bool {
        self.current_holder_state.need_head
    }

    fn automatic_merge_head(&self) -> bool {
        self.current_holder_state.automatic_merge_head
    }

    fn relative_head_row_index(&self) -> i32 {
        self.current_holder_state.relative_head_row_index
    }

    fn order_by_include_column(&self) -> bool {
        self.current_holder_state.order_by_include_column
    }

    fn include_column_indexes(&self) -> Option<&[usize]> {
        self.current_holder_state.include_column_indexes.as_deref()
    }

    fn include_column_field_names(&self) -> Option<&[String]> {
        self.current_holder_state
            .include_column_field_names
            .as_deref()
    }

    fn exclude_column_indexes(&self) -> &[usize] {
        &self.current_holder_state.exclude_column_indexes
    }

    fn exclude_column_field_names(&self) -> &[String] {
        &self.current_holder_state.exclude_column_field_names
    }
}
