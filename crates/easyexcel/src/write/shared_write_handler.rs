//! 共享 WriteHandler 写入处理链。
//!
//! 对应 Java：`com.alibaba.excel` 写入路径的 Handler 共享包装（内部类型）。

use std::collections::HashSet;
use std::{cell::RefCell, rc::Rc};

use crate::core::{
    ExcelCellStyle, ExcelColumn, ExcelWriteMetadata, Result, WriteCellContext, WriteFont,
    WriteHandler, WriteHandlerCapability, WriteRowContext, WriteSheetContext, WriteWorkbookContext,
};
use crate::event::NotRepeatExecutor;

use crate::write::write_options::WriteOptions;

include!("shared_write_handler/stateful_sheet_state.rs");

#[derive(Debug, Clone)]
struct SharedHandlerUniqueValue(String);

impl NotRepeatExecutor for SharedHandlerUniqueValue {
    fn unique_value(&self) -> &str {
        &self.0
    }
}

/// 对应 Java：com.alibaba.excel。 Shared ownership for one real handler instance.
///
/// Java Holder chains reference the same parent handler objects. Rust cannot
/// clone `Box<dyn WriteHandler>`, so effective Sheet/Table chains clone this
/// lightweight handle while all callbacks still mutate the original handler.
#[derive(Clone)]
pub(crate) struct SharedWriteHandler {
    inner: Rc<RefCell<Box<dyn WriteHandler>>>,
    order: i32,
    unique_value: Option<SharedHandlerUniqueValue>,
    requires_row_context: bool,
    requires_cell_context: bool,
    backend_capability: WriteHandlerCapability,
}

impl SharedWriteHandler {
    // 语义敏感：对应 Java Handler 在单线程写入链内的共享设计，`Box<dyn WriteHandler>`
    // 本身不要求 Send/Sync，写入链也只在当前线程顺序执行；Rc<RefCell<>> 既保留
    // Java parent handler 的共享实例语义，又避免每个 row/cell callback 获取 OS mutex。
    /// 对应 Java：com.alibaba.excel。
    pub(crate) fn new(handler: Box<dyn WriteHandler>) -> Self {
        let order = handler.order();
        let unique_value = handler
            .as_not_repeat_executor()
            .map(|executor| SharedHandlerUniqueValue(executor.unique_value().to_owned()));
        let requires_row_context = handler.requires_row_context();
        let requires_cell_context = handler.requires_cell_context();
        let backend_capability = handler.backend_capability();
        Self {
            inner: Rc::new(RefCell::new(handler)),
            order,
            unique_value,
            requires_row_context,
            requires_cell_context,
            backend_capability,
        }
    }
    /// 对应 Java：com.alibaba.excel。
    pub(crate) fn with_mut<R>(&self, action: impl FnOnce(&mut dyn WriteHandler) -> R) -> R {
        let mut handler = self.inner.borrow_mut();
        action(handler.as_mut())
    }
    /// 对应 Java：com.alibaba.excel。
    pub(crate) fn with_ref<R>(&self, action: impl FnOnce(&dyn WriteHandler) -> R) -> R {
        let handler = self.inner.borrow();
        action(handler.as_ref())
    }
}

impl WriteHandler for SharedWriteHandler {
    fn backend_capability(&self) -> WriteHandlerCapability {
        self.backend_capability
    }

    fn requires_row_context(&self) -> bool {
        self.requires_row_context
    }

    fn requires_cell_context(&self) -> bool {
        self.requires_cell_context
    }

    fn order(&self) -> i32 {
        self.order
    }

    fn as_not_repeat_executor(&self) -> Option<&dyn NotRepeatExecutor> {
        self.unique_value
            .as_ref()
            .map(|value| value as &dyn NotRepeatExecutor)
    }

    fn before_workbook_create(&mut self, context: &WriteWorkbookContext) -> Result<()> {
        self.with_mut(|handler| handler.before_workbook_create(context))
    }

    fn after_workbook_create(&mut self, context: &WriteWorkbookContext) -> Result<()> {
        self.with_mut(|handler| handler.after_workbook_create(context))
    }

    fn after_workbook_dispose(&mut self, context: &WriteWorkbookContext) -> Result<()> {
        self.with_mut(|handler| handler.after_workbook_dispose(context))
    }

    fn before_sheet_create(&mut self, context: &WriteSheetContext) -> Result<()> {
        self.with_mut(|handler| handler.before_sheet_create(context))
    }

    fn after_sheet_create(&mut self, context: &WriteSheetContext) -> Result<()> {
        self.with_mut(|handler| handler.after_sheet_create(context))
    }

    fn after_sheet_dispose(&mut self, context: &WriteSheetContext) -> Result<()> {
        self.with_mut(|handler| handler.after_sheet_dispose(context))
    }

    fn before_row_create(&mut self, context: &WriteRowContext) -> Result<()> {
        self.with_mut(|handler| handler.before_row_create(context))
    }

    fn after_row_create(&mut self, context: &WriteRowContext) -> Result<()> {
        self.with_mut(|handler| handler.after_row_create(context))
    }

    fn after_row_dispose(&mut self, context: &WriteRowContext) -> Result<()> {
        self.with_mut(|handler| handler.after_row_dispose(context))
    }

    fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
        self.with_mut(|handler| handler.before_cell_create(context))
    }

    fn after_cell_create(&mut self, context: &WriteCellContext) -> Result<()> {
        self.with_mut(|handler| handler.after_cell_create(context))
    }

    fn after_cell_data_converted(&mut self, context: &WriteCellContext) -> Result<()> {
        self.with_mut(|handler| handler.after_cell_data_converted(context))
    }

    fn after_cell_dispose(&mut self, context: &WriteCellContext) -> Result<()> {
        self.with_mut(|handler| handler.after_cell_dispose(context))
    }

    fn style_cell_style(&self, context: &WriteCellContext) -> Option<ExcelCellStyle> {
        self.with_ref(|handler| handler.style_cell_style(context))
    }

    fn style_write_font(&self, context: &WriteCellContext) -> Option<WriteFont> {
        self.with_ref(|handler| handler.style_write_font(context))
    }

    fn style_column_width(&self, column_index: usize) -> Option<u16> {
        self.with_ref(|handler| handler.style_column_width(column_index))
    }

    fn style_head_row_height(&self) -> Option<u16> {
        self.with_ref(|handler| handler.style_head_row_height())
    }

    fn style_content_row_height(&self) -> Option<u16> {
        self.with_ref(|handler| handler.style_content_row_height())
    }

    fn style_auto_column_width(&self) -> bool {
        self.with_ref(|handler| handler.style_auto_column_width())
    }

    fn style_once_absolute_merge(
        &self,
    ) -> Option<crate::metadata::property::OnceAbsoluteMergeProperty> {
        self.with_ref(|handler| handler.style_once_absolute_merge())
    }

    fn style_loop_merge(&self) -> Option<(crate::metadata::property::LoopMergeProperty, usize)> {
        self.with_ref(|handler| handler.style_loop_merge())
    }
}
/// 对应 Java：com.alibaba.excel。
pub(crate) fn share_handlers(handlers: Vec<Box<dyn WriteHandler>>) -> Vec<SharedWriteHandler> {
    handlers.into_iter().map(SharedWriteHandler::new).collect()
}
/// 对应 Java：com.alibaba.excel。
pub(crate) fn boxed_handlers(handlers: &[SharedWriteHandler]) -> Vec<Box<dyn WriteHandler>> {
    handlers
        .iter()
        .cloned()
        .map(|handler| Box::new(handler) as Box<dyn WriteHandler>)
        .collect()
}
/// 对应 Java：com.alibaba.excel。
pub(crate) fn normalized_shared_handlers(
    mut handlers: Vec<SharedWriteHandler>,
) -> Vec<SharedWriteHandler> {
    handlers.sort_by_key(SharedWriteHandler::order);
    let mut unique_values = HashSet::new();
    handlers.retain(|handler| {
        handler
            .unique_value
            .as_ref()
            .is_none_or(|value| unique_values.insert(value.unique_value().to_owned()))
    });
    handlers
}
