//! Excel 写入器辅助类型。
//!
//! 对应 Java：内部辅助类型。
//! 原文件：easyexcel-writer 内部辅助类型。

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use easyexcel_core::event::NotRepeatExecutor;
use easyexcel_core::{
    ExcelColumn, ExcelWriteMetadata, Result, WriteCellContext, WriteHandler, WriteWorkbookContext,
};

use crate::write_options::WriteOptions;

/// Global write flags copied from [`WriteOptions`] for cell emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub(crate) struct WriteGlobalFlags {
    pub(crate) auto_trim: bool,
    pub(crate) use_1904_windowing: bool,
    pub(crate) use_scientific_format: bool,
}

impl From<&WriteOptions> for WriteGlobalFlags {
    fn from(options: &WriteOptions) -> Self {
        Self {
            auto_trim: options.auto_trim,
            use_1904_windowing: options.use_1904_windowing,
            use_scientific_format: options.use_scientific_format,
        }
    }
}

/// Returns the worksheet name after applying [`WriteOptions::auto_trim`].
#[allow(dead_code)]
pub(crate) fn effective_sheet_name(options: &WriteOptions) -> String {
    if options.auto_trim {
        options.sheet_name.trim().to_owned()
    } else {
        options.sheet_name.clone()
    }
}

/// Trims string cell text when auto-trim is enabled.
#[allow(dead_code)]
pub(crate) fn maybe_trim_cell_string(value: &str, auto_trim: bool) -> String {
    if auto_trim {
        value.trim().to_owned()
    } else {
        value.to_owned()
    }
}

/// 对应 Java：/reader extreme-magnitude scientific formatting threshold.
#[allow(dead_code)]
pub(crate) fn is_scientific_magnitude(value: f64) -> bool {
    let absolute = value.abs();
    absolute >= 1E11 || (absolute <= 1E-10 && absolute > 0.0)
}

/// Immutable Java-holder state shared by row/cell callback construction.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct StatefulSheetState {
    pub(crate) schema: &'static [ExcelColumn],
    pub(crate) metadata: ExcelWriteMetadata,
    pub(crate) options: WriteOptions,
    pub(crate) next_row: u32,
    pub(crate) next_data_index: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SharedHandlerUniqueValue(String);

impl NotRepeatExecutor for SharedHandlerUniqueValue {
    fn unique_value(&self) -> &str {
        &self.0
    }
}

/// Shared ownership for one real handler instance.
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct SharedWriteHandler {
    pub(crate) inner: Arc<Mutex<Box<dyn WriteHandler>>>,
    pub(crate) order: i32,
    pub(crate) unique_value: Option<SharedHandlerUniqueValue>,
}

impl SharedWriteHandler {
    #[allow(dead_code)]
    // 语义敏感：对应 Java Handler 在单线程写入链内的共享设计，`Box<dyn WriteHandler>`
    // 本身不要求 Send/Sync，Arc<Mutex<>> 仅为生命周期共享，无需线程安全约束。
    #[allow(clippy::arc_with_non_send_sync)]
    pub(crate) fn new(handler: Box<dyn WriteHandler>) -> Self {
        let order = handler.order();
        let unique_value = handler
            .as_not_repeat_executor()
            .map(|executor| SharedHandlerUniqueValue(executor.unique_value().to_owned()));
        Self {
            inner: Arc::new(Mutex::new(handler)),
            order,
            unique_value,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn with_mut<R>(&self, action: impl FnOnce(&mut dyn WriteHandler) -> R) -> R {
        let mut handler = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        action(handler.as_mut())
    }

    #[allow(dead_code)]
    pub(crate) fn with_ref<R>(&self, action: impl FnOnce(&dyn WriteHandler) -> R) -> R {
        let handler = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        action(handler.as_ref())
    }
}

impl WriteHandler for SharedWriteHandler {
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

    fn before_sheet_create(&mut self, context: &easyexcel_core::WriteSheetContext) -> Result<()> {
        self.with_mut(|handler| handler.before_sheet_create(context))
    }

    fn after_sheet_create(&mut self, context: &easyexcel_core::WriteSheetContext) -> Result<()> {
        self.with_mut(|handler| handler.after_sheet_create(context))
    }

    fn after_sheet_dispose(&mut self, context: &easyexcel_core::WriteSheetContext) -> Result<()> {
        self.with_mut(|handler| handler.after_sheet_dispose(context))
    }

    fn before_row_create(&mut self, context: &easyexcel_core::WriteRowContext) -> Result<()> {
        self.with_mut(|handler| handler.before_row_create(context))
    }

    fn after_row_create(&mut self, context: &easyexcel_core::WriteRowContext) -> Result<()> {
        self.with_mut(|handler| handler.after_row_create(context))
    }

    fn after_row_dispose(&mut self, context: &easyexcel_core::WriteRowContext) -> Result<()> {
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

    fn style_cell_style(
        &self,
        context: &WriteCellContext,
    ) -> Option<easyexcel_core::ExcelCellStyle> {
        self.with_ref(|handler| handler.style_cell_style(context))
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
    ) -> Option<easyexcel_core::metadata::property::OnceAbsoluteMergeProperty> {
        self.with_ref(|handler| handler.style_once_absolute_merge())
    }

    fn style_loop_merge(
        &self,
    ) -> Option<(easyexcel_core::metadata::property::LoopMergeProperty, usize)> {
        self.with_ref(|handler| handler.style_loop_merge())
    }
}

#[allow(dead_code)]
pub(crate) fn share_handlers(handlers: Vec<Box<dyn WriteHandler>>) -> Vec<SharedWriteHandler> {
    handlers.into_iter().map(SharedWriteHandler::new).collect()
}

#[allow(dead_code)]
pub(crate) fn boxed_handlers(handlers: &[SharedWriteHandler]) -> Vec<Box<dyn WriteHandler>> {
    handlers
        .iter()
        .cloned()
        .map(|handler| Box::new(handler) as Box<dyn WriteHandler>)
        .collect()
}

#[allow(dead_code)]
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

/// Java `AbstractWriteHolder`'s own/effective execution-chain pair.
#[derive(Clone, Default)]
#[allow(dead_code)]
pub(crate) struct HandlerExecutionScope {
    pub(crate) own: Vec<SharedWriteHandler>,
    pub(crate) effective: Vec<SharedWriteHandler>,
}

impl HandlerExecutionScope {
    #[allow(dead_code)]
    pub(crate) fn root(handlers: &[SharedWriteHandler]) -> Self {
        let own = normalized_shared_handlers(handlers.to_vec());
        Self {
            effective: own.clone(),
            own,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn child(own_handlers: &[SharedWriteHandler], parent: &Self) -> Self {
        let own_candidates = own_handlers.to_vec();
        let own = normalized_shared_handlers(own_candidates.clone());
        let mut effective_candidates = own_candidates;
        effective_candidates.extend(parent.effective.iter().cloned());
        Self {
            own,
            effective: normalized_shared_handlers(effective_candidates),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn own_boxed(&self) -> Vec<Box<dyn WriteHandler>> {
        boxed_handlers(&self.own)
    }

    #[allow(dead_code)]
    pub(crate) fn effective_boxed(&self) -> Vec<Box<dyn WriteHandler>> {
        boxed_handlers(&self.effective)
    }
}

/// 用于 CSV 捕获输出的包装器。
#[derive(Clone, Default)]
pub(crate) struct CapturedOutput(pub(crate) Arc<Mutex<Vec<u8>>>);

impl std::io::Write for CapturedOutput {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("CSV capture lock poisoned"))?
            .extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Test helper to drain captured output bytes.
#[allow(dead_code)]
pub(crate) fn take_captured_output_helper(output: &CapturedOutput) -> Vec<u8> {
    output
        .0
        .lock()
        .map_err(|_| std::io::Error::other("CSV capture lock poisoned"))
        .map(|mut bytes| std::mem::take(&mut *bytes))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_core::{
        CellValue, ExcelCellStyle, WriteHandler, WriteRowContext, WriteSheetContext,
        metadata::property::{LoopMergeProperty, OnceAbsoluteMergeProperty},
    };

    struct RecordingHandler(Arc<Mutex<Vec<&'static str>>>);

    impl RecordingHandler {
        fn record(&self, event: &'static str) {
            self.0.lock().expect("event log mutex poisoned").push(event);
        }
    }

    impl WriteHandler for RecordingHandler {
        fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            self.record("before_workbook_create");
            Ok(())
        }

        fn after_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            self.record("after_workbook_create");
            Ok(())
        }

        fn after_workbook_dispose(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            self.record("after_workbook_dispose");
            Ok(())
        }

        fn before_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
            self.record("before_sheet_create");
            Ok(())
        }

        fn after_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
            self.record("after_sheet_create");
            Ok(())
        }

        fn after_sheet_dispose(&mut self, _context: &WriteSheetContext) -> Result<()> {
            self.record("after_sheet_dispose");
            Ok(())
        }

        fn before_row_create(&mut self, _context: &WriteRowContext) -> Result<()> {
            self.record("before_row_create");
            Ok(())
        }

        fn after_row_create(&mut self, _context: &WriteRowContext) -> Result<()> {
            self.record("after_row_create");
            Ok(())
        }

        fn after_row_dispose(&mut self, _context: &WriteRowContext) -> Result<()> {
            self.record("after_row_dispose");
            Ok(())
        }

        fn before_cell_create(&mut self, _context: &mut WriteCellContext) -> Result<()> {
            self.record("before_cell_create");
            Ok(())
        }

        fn after_cell_create(&mut self, _context: &WriteCellContext) -> Result<()> {
            self.record("after_cell_create");
            Ok(())
        }

        fn after_cell_data_converted(&mut self, _context: &WriteCellContext) -> Result<()> {
            self.record("after_cell_data_converted");
            Ok(())
        }

        fn after_cell_dispose(&mut self, _context: &WriteCellContext) -> Result<()> {
            self.record("after_cell_dispose");
            Ok(())
        }

        fn style_cell_style(&self, _context: &WriteCellContext) -> Option<ExcelCellStyle> {
            Some(ExcelCellStyle::default())
        }

        fn style_column_width(&self, _column_index: usize) -> Option<u16> {
            Some(11)
        }

        fn style_head_row_height(&self) -> Option<u16> {
            Some(21)
        }

        fn style_content_row_height(&self) -> Option<u16> {
            Some(31)
        }

        fn style_auto_column_width(&self) -> bool {
            true
        }

        fn style_once_absolute_merge(&self) -> Option<OnceAbsoluteMergeProperty> {
            Some(OnceAbsoluteMergeProperty {
                first_row_index: 0,
                last_row_index: 1,
                first_column_index: 0,
                last_column_index: 1,
            })
        }

        fn style_loop_merge(&self) -> Option<(LoopMergeProperty, usize)> {
            Some((
                LoopMergeProperty {
                    each_row: 2,
                    column_extend: 1,
                },
                3,
            ))
        }
    }

    #[test]
    fn shared_handler_forwards_all_callbacks_and_style_queries() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut shared = SharedWriteHandler::new(Box::new(RecordingHandler(Arc::clone(&events))));

        let workbook_context = WriteWorkbookContext::new("out.xlsx");
        let sheet_context = WriteSheetContext::new("Sheet1");
        let row_context = WriteRowContext::new("Sheet1", 0, None, false);
        let mut cell_context =
            WriteCellContext::new("Sheet1", 0, 0, CellValue::String("x".to_owned()));

        shared.before_workbook_create(&workbook_context).unwrap();
        shared.after_workbook_create(&workbook_context).unwrap();
        shared.after_workbook_dispose(&workbook_context).unwrap();
        shared.before_sheet_create(&sheet_context).unwrap();
        shared.after_sheet_create(&sheet_context).unwrap();
        shared.after_sheet_dispose(&sheet_context).unwrap();
        shared.before_row_create(&row_context).unwrap();
        shared.after_row_create(&row_context).unwrap();
        shared.after_row_dispose(&row_context).unwrap();
        shared.before_cell_create(&mut cell_context).unwrap();
        shared.after_cell_create(&cell_context).unwrap();
        shared.after_cell_data_converted(&cell_context).unwrap();
        shared.after_cell_dispose(&cell_context).unwrap();

        assert_eq!(
            shared.style_cell_style(&cell_context),
            Some(ExcelCellStyle::default())
        );
        assert_eq!(shared.style_column_width(0), Some(11));
        assert_eq!(shared.style_head_row_height(), Some(21));
        assert_eq!(shared.style_content_row_height(), Some(31));
        assert!(shared.style_auto_column_width());
        assert_eq!(
            shared.style_once_absolute_merge(),
            Some(OnceAbsoluteMergeProperty {
                first_row_index: 0,
                last_row_index: 1,
                first_column_index: 0,
                last_column_index: 1,
            })
        );
        assert_eq!(
            shared.style_loop_merge(),
            Some((
                LoopMergeProperty {
                    each_row: 2,
                    column_extend: 1,
                },
                3,
            ))
        );

        let log = events.lock().expect("event log mutex poisoned");
        assert_eq!(log.len(), 13);
        assert_eq!(log[0], "before_workbook_create");
        assert_eq!(log[12], "after_cell_dispose");
    }

    #[test]
    fn shared_handler_unique_value_reports_inner_string() {
        let unique = SharedHandlerUniqueValue("abc".to_owned());
        assert_eq!(unique.unique_value(), "abc");
        assert_eq!(unique.unique_value(), unique.unique_value());
    }
}
