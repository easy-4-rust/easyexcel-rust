//! Backend-neutral read-only views of Java write holders.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{
    ExcelWriteHeadProperty, Holder, WriteContext, WriteContextHolder, WriteContextHolderState,
};

include!("write_holder_context/write_workbook_holder_view.rs");

include!("write_holder_context/write_sheet_holder_view.rs");

include!("write_holder_context/write_table_holder_view.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Holder set captured for a concrete write-handler callback.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteHolderContext {
    workbook: Option<WriteWorkbookHolderView>,
    sheet: Option<WriteSheetHolderView>,
    table: Option<WriteTableHolderView>,
    current_holder_state: Arc<WriteContextHolderState>,
}

impl WriteHolderContext {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates an empty holder set for compatibility constructors.
    #[must_use]
    pub fn new() -> Self {
        Self {
            workbook: None,
            sheet: None,
            table: None,
            current_holder_state: Arc::new(WriteContextHolderState::default()),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Attaches the active workbook holder view.
    #[must_use]
    pub fn with_workbook(mut self, workbook: WriteWorkbookHolderView) -> Self {
        self.workbook = Some(workbook);
        self
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Attaches the active sheet holder view.
    #[must_use]
    pub fn with_sheet(mut self, sheet: WriteSheetHolderView) -> Self {
        self.sheet = Some(sheet);
        self
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Attaches the active table holder view.
    #[must_use]
    pub fn with_table(mut self, table: WriteTableHolderView) -> Self {
        self.table = Some(table);
        self
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Attaches the fully resolved Java `currentWriteHolder()` state.
    #[must_use]
    pub fn with_current_holder_state(mut self, state: WriteContextHolderState) -> Self {
        self.current_holder_state = Arc::new(state);
        self
    }

    /// 复用已经解析的不可变 holder 快照，避免每个单元格深拷贝表头、转换器与列选择。
    #[must_use]
    pub(crate) fn with_shared_current_holder_state(
        mut self,
        state: Arc<WriteContextHolderState>,
    ) -> Self {
        self.current_holder_state = state;
        self
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Captures all backend-neutral holder state from a live write context.
    #[must_use]
    pub fn from_write_context(context: &dyn WriteContext) -> Self {
        let holder = context.current_write_holder();
        let mut snapshot = Self::new()
            .with_workbook(WriteWorkbookHolderView::new(holder.path()))
            .with_current_holder_state(WriteContextHolderState::from_holder(holder));

        if let Some(sheet_name) = holder.sheet_name() {
            let mut sheet = WriteSheetHolderView::new(sheet_name);
            if let Some(sheet_no) = holder.sheet_no() {
                sheet = sheet.with_sheet_no(sheet_no);
            }
            if let Some(last_row_index) = holder.last_row_index() {
                sheet = sheet.with_last_row_index(last_row_index);
            }
            snapshot = snapshot.with_sheet(sheet);
            if let Some(table_no) = holder.table_no() {
                snapshot = snapshot.with_table(WriteTableHolderView::new(table_no, sheet_name));
            }
        }
        snapshot
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Sets callback-specific sheet and optional latest-row state while
    /// preserving the live holder's resolved sheet number.
    #[must_use]
    pub fn with_callback_sheet(
        mut self,
        sheet_name: impl Into<String>,
        last_row_index: Option<u32>,
    ) -> Self {
        let sheet_name = sheet_name.into();
        let mut sheet = WriteSheetHolderView::new(&sheet_name);
        if let Some(sheet_no) = self.sheet.as_ref().and_then(WriteSheetHolderView::sheet_no) {
            sheet = sheet.with_sheet_no(sheet_no);
        }
        if let Some(last_row_index) = last_row_index {
            sheet = sheet.with_last_row_index(last_row_index);
        }
        self.sheet = Some(sheet);
        if let Some(table_no) = self.table.as_ref().map(WriteTableHolderView::table_no) {
            self.table = Some(WriteTableHolderView::new(table_no, sheet_name));
        }
        self
    }

    /// Returns the active workbook holder view.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn workbook(&self) -> Option<&WriteWorkbookHolderView> {
        self.workbook.as_ref()
    }

    /// Returns the active sheet holder view.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn sheet(&self) -> Option<&WriteSheetHolderView> {
        self.sheet.as_ref()
    }

    /// Returns the active table holder view.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn table(&self) -> Option<&WriteTableHolderView> {
        self.table.as_ref()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the active write holder through the Java-compatible context API.
    #[must_use]
    pub fn current_write_holder(&self) -> &dyn WriteContextHolder {
        self
    }
}

impl Default for WriteHolderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteContext for WriteHolderContext {
    fn current_write_holder(&self) -> &dyn WriteContextHolder {
        self
    }
}

impl WriteContextHolder for WriteHolderContext {
    fn path(&self) -> &Path {
        self.workbook
            .as_ref()
            .map_or_else(|| Path::new(""), WriteWorkbookHolderView::path)
    }

    fn table_no(&self) -> Option<i32> {
        self.table.as_ref().map(WriteTableHolderView::table_no)
    }

    fn sheet_name(&self) -> Option<&str> {
        self.sheet.as_ref().map(WriteSheetHolderView::sheet_name)
    }

    fn sheet_no(&self) -> Option<i32> {
        self.sheet.as_ref().and_then(WriteSheetHolderView::sheet_no)
    }

    fn last_row_index(&self) -> Option<u32> {
        self.sheet
            .as_ref()
            .and_then(WriteSheetHolderView::last_row_index)
    }

    fn has_data(&self) -> bool {
        self.sheet
            .as_ref()
            .is_some_and(WriteSheetHolderView::has_data)
    }

    fn holder_type(&self) -> Holder {
        self.current_holder_state.holder_type
    }

    fn excel_write_head_property(&self) -> &ExcelWriteHeadProperty {
        &self.current_holder_state.excel_write_head_property
    }

    fn converter_map(&self) -> &crate::ConverterRegistry {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CellValue, WriteCellContext, WriteRowContext, WriteSheetContext};

    #[test]
    fn holder_views_preserve_real_workbook_sheet_row_and_table_state() {
        let workbook = WriteWorkbookHolderView::new("target.xlsx");
        let sheet =
            WriteSheetContext::new("Users").with_holder_context(workbook.clone(), 2, Some(7));
        assert_eq!(
            sheet
                .write_workbook_holder()
                .map(WriteWorkbookHolderView::path),
            Some(Path::new("target.xlsx"))
        );
        assert_eq!(sheet.write_sheet_holder().sheet_name(), "Users");
        assert_eq!(sheet.write_sheet_holder().sheet_no(), Some(2));
        assert_eq!(
            sheet
                .write_table_holder()
                .map(WriteTableHolderView::table_no),
            Some(7)
        );

        let row = WriteRowContext::new("Users", 42, Some(3), false).with_holder_context(
            workbook.clone(),
            2,
            Some(7),
        );
        assert_eq!(row.write_sheet_holder().last_row_index(), Some(42));
        assert!(row.write_sheet_holder().has_data());

        let cell = WriteCellContext::new("Users", 42, 1, CellValue::Int(9)).with_holder_context(
            workbook,
            2,
            Some(7),
        );
        assert_eq!(
            cell.write_workbook_holder()
                .map(WriteWorkbookHolderView::path),
            Some(Path::new("target.xlsx"))
        );
        assert_eq!(
            cell.write_table_holder()
                .map(WriteTableHolderView::parent_sheet_name),
            Some("Users")
        );
    }

    #[test]
    fn compatibility_contexts_report_unknown_state_as_absent() {
        let sheet = WriteSheetContext::new("Sheet1");
        assert!(sheet.write_workbook_holder().is_none());
        assert_eq!(sheet.write_sheet_holder().sheet_no(), None);
        assert!(sheet.write_table_holder().is_none());
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn default_matches_new() {
        // 对应 Java：WriteHolderContext 默认构造
        let context = WriteHolderContext::default();
        assert_eq!(context.sheet_no(), None);
        assert_eq!(context.last_row_index(), None);
        assert!(!context.has_data());
    }

    #[test]
    fn holder_trait_sheet_accessors_with_sheet() {
        // 对应 Java：WriteContextHolder 的 sheet 访问器
        let context = WriteHolderContext::new()
            .with_sheet(
                WriteSheetHolderView::new("Sheet1")
                    .with_sheet_no(4)
                    .with_last_row_index(9),
            )
            .with_table(WriteTableHolderView::new(3, "Sheet1"));
        assert_eq!(context.sheet_no(), Some(4));
        assert_eq!(context.last_row_index(), Some(9));
        assert!(context.has_data());
        assert_eq!(context.sheet_name(), Some("Sheet1"));
        assert_eq!(context.table_no(), Some(3));
    }

    #[test]
    fn from_write_context_captures_table_branch() {
        // 对应 Java：快照包含 sheet 与 table 分支
        let mut live = crate::WriteContextImpl::new("live.xlsx");
        live.set_sheet_context("Users");
        live.set_table_no(Some(5));
        let snapshot = WriteHolderContext::from_write_context(&live);
        assert_eq!(snapshot.table_no(), Some(5));
        assert_eq!(snapshot.sheet_name(), Some("Users"));
        assert_eq!(snapshot.path(), Path::new("live.xlsx"));
        assert_eq!(snapshot.holder_type(), Holder::Table);
    }

    #[test]
    fn from_write_context_without_sheet_stays_workbook() {
        // 对应 Java：无 sheet 时快照仅含 workbook
        let live = crate::WriteContextImpl::new("bare.xlsx");
        let snapshot = WriteHolderContext::from_write_context(&live);
        assert!(snapshot.sheet_name().is_none());
        assert_eq!(snapshot.path(), Path::new("bare.xlsx"));
    }
}
