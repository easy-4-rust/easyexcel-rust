//! 对应 Java：`com.alibaba.excel.write.handler.context.CellWriteHandlerContext`.

use crate::cell_value::CellValue;
use crate::enum_cell_data_type::CellDataType;
use crate::excel_column::ExcelColumn;
use crate::{
    WriteCellHandle, WriteContext, WriteHolderContext, WriteSheetHolderView, WriteTableHolderView,
    WriteWorkbookHolderView,
};

/// 私有：转换前原始值的三态缓存（对应旧实现 `Option<CellValue>` 加构造默认）。
///
/// - [`PendingOriginalValue::Explicit`]：`with_original_value` 显式设置，
///   `activate_original_value` 时移动（`take`）为 `original_value`，零克隆。
/// - [`PendingOriginalValue::ConstructionDefault`]：构造器默认（`with_original_value`
///   / `without_original_value` 均未调用），激活时以当前 `value` 兜底，等价于旧
///   实现构造时 `pending_original_value: Some(value)` 的 handler 可见语义。
/// - [`PendingOriginalValue::Cleared`]：`without_original_value` 显式清除（表头
///   单元格），激活后 `original_value` 保持 `None`。
#[derive(Debug, Clone, PartialEq, Default)]
enum PendingOriginalValue {
    Explicit(CellValue),
    ConstructionDefault,
    #[default]
    Cleared,
}

/// Mutable cell-level write lifecycle context.
///
/// 对应 Java：`CellWriteHandlerContext` (13 fields). Rust keeps only the
/// fields a handler actually mutates and exposes `skip: bool` so handlers
/// can suppress writing a cell without juggling the underlying POI types.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteCellContext {
    /// Worksheet name.
    pub sheet_name: String,
    /// Physical zero-based row index.
    pub row_index: u32,
    /// Physical zero-based column index.
    pub column_index: u16,
    /// Rust field name, when backed by a typed column.
    pub field: Option<&'static str>,
    /// Resolved static head/content metadata for this typed field.
    pub column: Option<&'static ExcelColumn>,
    /// Header label at this level, when this is a header cell.
    pub head_name: Option<String>,
    /// Whether this is a header cell.
    pub is_head: bool,
    /// Relative row index within head or content (Java `relativeRowIndex`).
    ///
    /// Used by `HorizontalCellStyleStrategy` to cycle content styles.
    pub relative_row_index: Option<usize>,
    /// Value before write handlers transform it.
    ///
    /// 对应 Java：`CellWriteHandlerContext.originalValue`. Typed writer
    /// paths replace the constructor default with the field's value before a
    /// registered or annotation converter runs.
    pub original_value: Option<CellValue>,
    /// Declared Rust field type before conversion.
    ///
    /// 对应 Java：`CellWriteHandlerContext.originalFieldClass`.
    pub original_field_type: Option<&'static str>,
    /// Source value held until Java's conversion stage begins.
    ///
    /// 热路径上该字段在 `new` 之后立即被 `with_original_value(...)` 覆盖，
    /// 因此构造器不再预克隆 value（旧实现为 `Some(value)`，每单元格浪费一次
    /// String 堆分配）；[`PendingOriginalValue::ConstructionDefault`] 状态由
    /// `activate_original_value` 以当前 `value` 兜底。
    pending_original_value: PendingOriginalValue,
    /// Declared field type held until Java's conversion stage begins.
    pending_original_field_type: Option<&'static str>,
    /// Converted cell data visible from `afterCellDataConverted` onward.
    ///
    /// Java permits multiple `WriteCellData` values. The current typed writer
    /// emits one scalar value, but retains the list shape for handler parity.
    pub cell_data_list: Vec<CellValue>,
    /// Target cell type selected by conversion.
    pub target_cell_data_type: Option<CellDataType>,
    /// Suppresses annotation/strategy style filling for this cell.
    ///
    /// 对应 Java：`CellWriteHandlerContext.ignoreFillStyle`.
    pub ignore_fill_style: bool,
    /// Value that will be written. A handler may replace it.
    pub value: CellValue,
    /// A handler may set this to suppress the physical cell.
    pub skip: bool,
    cell: WriteCellHandle,
    holders: WriteHolderContext,
}

impl WriteCellContext {
    /// Returns the mutable logical cell handle.
    #[must_use]
    pub const fn cell(&self) -> &WriteCellHandle {
        &self.cell
    }

    /// Creates a cell handler context before cell conversion callbacks run.
    #[must_use]
    pub fn new(
        sheet_name: impl Into<String>,
        row_index: u32,
        column_index: u16,
        value: CellValue,
    ) -> Self {
        let sheet_name = sheet_name.into();
        Self {
            // 视图在回调中确实被读取（模板样式路径与兼容测试直接使用构造器
            // 视图），不能惰性化；热路径上由 with_write_context 整体替换。
            holders: WriteHolderContext::new()
                .with_sheet(WriteSheetHolderView::new(&sheet_name).with_last_row_index(row_index)),
            cell: WriteCellHandle::new(row_index, column_index, value.clone()),
            sheet_name,
            row_index,
            column_index,
            field: None,
            column: None,
            head_name: None,
            is_head: false,
            relative_row_index: None,
            original_value: None,
            original_field_type: None,
            // 构造器不再克隆 value：热路径立即被 with_original_value 覆盖；
            // 未被覆盖的路径由 activate_original_value 以当前 value 兜底。
            pending_original_value: PendingOriginalValue::ConstructionDefault,
            pending_original_field_type: None,
            cell_data_list: Vec::new(),
            target_cell_data_type: None,
            ignore_fill_style: false,
            value,
            skip: false,
        }
    }

    /// Attaches typed column metadata.
    #[must_use]
    pub const fn with_column(mut self, column: &'static ExcelColumn) -> Self {
        self.field = if column.field.is_empty() {
            None
        } else {
            Some(column.field)
        };
        self.pending_original_field_type = column.field_type;
        self.column = Some(column);
        self
    }

    /// Replaces the source value captured before conversion.
    #[must_use]
    pub fn with_original_value(mut self, original_value: CellValue) -> Self {
        self.pending_original_value = PendingOriginalValue::Explicit(original_value);
        self
    }

    /// Clears the source value for header cells.
    ///
    /// Java does not assign `originalValue` while creating head rows.
    #[must_use]
    pub fn without_original_value(mut self) -> Self {
        self.original_value = None;
        self.original_field_type = None;
        self.pending_original_value = PendingOriginalValue::Cleared;
        self.pending_original_field_type = None;
        self
    }

    /// Makes pre-converter metadata visible at Java's conversion stage.
    ///
    /// 显式设置的 pending 采用移动（`take`）而非克隆：`original_value` 与转换
    /// 前值共享同一份底层缓冲，热路径上每次回调省去一次 `CellValue` 克隆
    /// （`CellValue::String` 时为一次 String 堆分配）。构造器默认状态（未调用
    /// `with_original_value` / `without_original_value`，如表头、模板缺失原始值
    /// 的单元格）时以当前 `value` 兜底，保持与旧实现
    /// `pending_original_value: Some(value)` 一致的 handler 可见语义。
    pub fn activate_original_value(&mut self) {
        self.original_value = match std::mem::take(&mut self.pending_original_value) {
            PendingOriginalValue::Explicit(value) => Some(value),
            PendingOriginalValue::ConstructionDefault => Some(self.value.clone()),
            PendingOriginalValue::Cleared => None,
        };
        self.original_field_type = self.pending_original_field_type.take();
    }

    /// Marks a header cell and records its current label.
    #[must_use]
    pub fn with_head(mut self, head_name: impl Into<String>) -> Self {
        self.is_head = true;
        self.head_name = Some(head_name.into());
        self
    }

    /// Sets the relative row index.
    #[must_use]
    pub const fn with_relative_row_index(mut self, relative_row_index: Option<usize>) -> Self {
        self.relative_row_index = relative_row_index;
        self
    }

    /// Returns the first converted cell value.
    ///
    /// 对应 Java：`CellWriteHandlerContext.getFirstCellData()`.
    #[must_use]
    pub fn first_cell_data(&self) -> Option<&CellValue> {
        self.cell_data_list.first()
    }

    /// Refreshes conversion metadata after a handler changes [`Self::value`].
    pub fn refresh_converted_data(&mut self) {
        self.target_cell_data_type = Some(self.value.data_type());
        self.cell_data_list.clear();
        self.cell_data_list.push(self.value.clone());
    }

    /// Applies mutations requested through [`Self::cell`].
    ///
    /// Writer backends call this after the logical callback chain and before
    /// committing the physical cell.
    pub fn apply_cell_mutations(&mut self) {
        if let Some(value) = self.cell.requested_value() {
            self.value = value;
            self.refresh_converted_data();
        }
        if let Some(skip) = self.cell.requested_skip() {
            self.skip = skip;
        }
    }

    /// Synchronizes the logical handle after compatibility callbacks mutate
    /// [`Self::value`] directly.
    pub fn sync_cell_handle(&self) {
        self.cell.sync_value(&self.value);
    }

    /// Attaches the real writer holder state visible for this cell callback.
    #[must_use]
    pub fn with_holder_context(
        mut self,
        workbook: WriteWorkbookHolderView,
        sheet_no: i32,
        table_no: Option<i32>,
    ) -> Self {
        let holder_type = if table_no.is_some() {
            crate::Holder::Table
        } else {
            crate::Holder::Sheet
        };
        self = self.with_resolved_holder_context(
            workbook,
            sheet_no,
            table_no,
            crate::WriteContextHolderState {
                holder_type,
                ..crate::WriteContextHolderState::default()
            },
        );
        self
    }

    /// Replaces compatibility holder data with a live-context snapshot.
    ///
    /// 内联等价 `WriteHolderContext::from_write_context(context)` 加
    /// `with_callback_sheet(&self.sheet_name, Some(self.row_index))`：旧实现会
    /// 先后两次以相同名称构造 sheet 视图（`with_callback_sheet` 整体重建并丢弃
    /// 前一视图，table 同理），热路径上每单元格多两次 String 克隆。此处合并为
    /// 一次构造，最终视图内容（`sheet_name`、`sheet_no`、`last_row_index`、
    /// `has_data`、table）与旧实现完全一致。
    #[must_use]
    pub fn with_write_context(mut self, context: &dyn WriteContext) -> Self {
        let holder = context.current_write_holder();
        let mut holders = WriteHolderContext::new()
            .with_workbook(WriteWorkbookHolderView::new(holder.path()))
            .with_current_holder_state(crate::WriteContextHolderState::from_holder(holder));
        if let Some(_sheet_name) = holder.sheet_name() {
            // sheet_no 与 table 来源与旧实现一致；回调视图名称以 self.sheet_name
            // 为准（旧 with_callback_sheet 的最终视图亦以 self.sheet_name 重建）。
            let mut sheet =
                WriteSheetHolderView::new(&self.sheet_name).with_last_row_index(self.row_index);
            if let Some(sheet_no) = holder.sheet_no() {
                sheet = sheet.with_sheet_no(sheet_no);
            }
            holders = holders.with_sheet(sheet);
            if let Some(table_no) = holder.table_no() {
                holders = holders.with_table(WriteTableHolderView::new(table_no, &self.sheet_name));
            }
        } else {
            // 与旧 with_callback_sheet 的兜底一致：live holder 无 sheet 时仍以
            // 自身 sheet 名提供回调视图（sheet_no 为 None，last_row_index 为当前行）。
            holders = holders.with_sheet(
                WriteSheetHolderView::new(&self.sheet_name).with_last_row_index(self.row_index),
            );
        }
        self.holders = holders;
        self
    }

    /// Attaches all holder views and the resolved Java `currentWriteHolder()` state.
    #[must_use]
    pub fn with_resolved_holder_context(
        mut self,
        workbook: WriteWorkbookHolderView,
        sheet_no: i32,
        table_no: Option<i32>,
        current_holder_state: crate::WriteContextHolderState,
    ) -> Self {
        let sheet = WriteSheetHolderView::new(&self.sheet_name)
            .with_sheet_no(sheet_no)
            .with_last_row_index(self.row_index);
        self.holders = WriteHolderContext::new()
            .with_workbook(workbook)
            .with_sheet(sheet)
            .with_current_holder_state(current_holder_state);
        if let Some(table_no) = table_no {
            self.holders = self
                .holders
                .with_table(WriteTableHolderView::new(table_no, &self.sheet_name));
        }
        self
    }

    /// Returns the active workbook holder view, when supplied by the writer.
    #[must_use]
    pub const fn write_workbook_holder(&self) -> Option<&WriteWorkbookHolderView> {
        self.holders.workbook()
    }

    /// Returns the active sheet holder view.
    ///
    /// # Panics
    ///
    /// Panics when the callback was created without a sheet holder
    /// (cell callbacks always carry one).
    #[must_use]
    pub fn write_sheet_holder(&self) -> &WriteSheetHolderView {
        self.holders
            .sheet()
            .expect("cell contexts always carry a sheet holder")
    }

    /// Returns the active table holder view for table callbacks.
    #[must_use]
    pub const fn write_table_holder(&self) -> Option<&WriteTableHolderView> {
        self.holders.table()
    }

    /// Returns all holder views captured for this callback.
    #[must_use]
    pub const fn write_context(&self) -> &WriteHolderContext {
        &self.holders
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::WriteContextHolder;

    #[test]
    fn activate_original_value_keeps_constructor_default() {
        // 对应 Java：未显式覆盖 pending 的路径（如表头以外但无原始值的模板
        // 单元格）在 activate 后 original_value 仍为构造值
        let mut context =
            WriteCellContext::new("Sheet1", 1, 0, CellValue::String("src".to_owned()));
        context.activate_original_value();
        assert_eq!(
            context.original_value,
            Some(CellValue::String("src".to_owned()))
        );
        assert_eq!(context.original_field_type, None);
    }

    #[test]
    fn activate_original_value_uses_explicit_override() {
        // 对应 Java：热路径每单元格 new 后立即 with_original_value 覆盖默认
        let mut context =
            WriteCellContext::new("Sheet1", 1, 0, CellValue::String("src".to_owned()))
                .with_original_value(CellValue::String("original".to_owned()));
        context.activate_original_value();
        assert_eq!(
            context.original_value,
            Some(CellValue::String("original".to_owned()))
        );
    }

    #[test]
    fn activate_original_value_stays_cleared_for_head_cells() {
        // 对应 Java：表头单元格 originalValue 保持空（without_original_value）
        let mut context = WriteCellContext::new("Sheet1", 1, 0, CellValue::String("h".to_owned()))
            .with_head("H")
            .without_original_value();
        context.activate_original_value();
        assert_eq!(context.original_value, None);
        assert_eq!(context.original_field_type, None);
    }

    #[test]
    fn activate_original_value_moves_pending_without_clone() {
        // 显式 pending 采用移动而非克隆：original_value 与传入值内容一致
        let mut context = WriteCellContext::new("Sheet1", 1, 0, CellValue::Int(1))
            .with_original_value(CellValue::String("shared".to_owned()));
        context.activate_original_value();
        assert_eq!(
            context.original_value,
            Some(CellValue::String("shared".to_owned()))
        );
    }

    #[test]
    fn with_write_context_keeps_holder_view_equivalence() {
        // 优化后内联合并视图构造，最终视图内容与旧实现
        // from_write_context + with_callback_sheet 完全一致
        let live_context = crate::WriteHolderContext::new()
            .with_workbook(crate::WriteWorkbookHolderView::new("live.xlsx"))
            .with_sheet(crate::WriteSheetHolderView::new("Users").with_sheet_no(4))
            .with_table(crate::WriteTableHolderView::new(3, "Users"))
            .with_current_holder_state(crate::WriteContextHolderState {
                holder_type: crate::Holder::Table,
                ..crate::WriteContextHolderState::default()
            });
        let context = WriteCellContext::new("Users", 7, 1, CellValue::Int(9))
            .with_write_context(&live_context);
        let expected = WriteHolderContext::from_write_context(&live_context)
            .with_callback_sheet("Users", Some(7));
        assert_eq!(context.write_context(), &expected);
    }

    #[test]
    fn with_write_context_keeps_fallback_without_live_sheet() {
        // live holder 无 sheet 时仍提供以自身 sheet 名构造的回调视图
        // （与旧 with_callback_sheet 兜底一致）
        let workbook_only = crate::WriteHolderContext::new()
            .with_workbook(crate::WriteWorkbookHolderView::new("out.xlsx"));
        let context = WriteCellContext::new("Data", 3, 0, CellValue::Int(1))
            .with_write_context(&workbook_only);
        assert_eq!(context.write_sheet_holder().sheet_name(), "Data");
        assert_eq!(context.write_sheet_holder().sheet_no(), None);
        assert_eq!(context.write_sheet_holder().last_row_index(), Some(3));
        assert!(context.write_table_holder().is_none());
    }

    #[test]
    fn with_holder_context_resolves_table_and_sheet() {
        // 对应 Java：table_no 存在时为 Table，否则为 Sheet
        let table = WriteCellContext::new("Sheet1", 1, 0, CellValue::Int(1)).with_holder_context(
            WriteWorkbookHolderView::new("out.xlsx"),
            2,
            Some(3),
        );
        assert_eq!(table.write_context().holder_type(), crate::Holder::Table);

        let sheet = WriteCellContext::new("Sheet1", 1, 0, CellValue::Int(1)).with_holder_context(
            WriteWorkbookHolderView::new("out.xlsx"),
            2,
            None,
        );
        assert_eq!(sheet.write_context().holder_type(), crate::Holder::Sheet);
    }

    #[test]
    fn apply_cell_mutations_and_sync_handle() {
        // 对应 Java：handler 变更经 handle 提交
        let mut context = WriteCellContext::new("Sheet1", 1, 0, CellValue::Int(1));
        context.value = CellValue::String("changed".to_owned());
        context.sync_cell_handle();
        context.apply_cell_mutations();
        assert_eq!(context.value, CellValue::String("changed".to_owned()));
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    #[test]
    fn apply_cell_mutations_honors_skip_request() {
        // 对应 Java：handler 通过 handle 请求跳过该单元格（setSkipped → skip）
        let mut context = WriteCellContext::new("Sheet1", 1, 0, CellValue::Int(1));
        context.cell().set_skipped(true);
        context.apply_cell_mutations();
        assert!(context.skip);
    }
}
