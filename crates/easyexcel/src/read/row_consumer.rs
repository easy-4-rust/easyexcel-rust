//! Internal row-event consumer abstraction shared by the XLSX, XLS, and CSV engines.

use crate::core::{
    AnalysisContext, CellExtra, CellValue, ExcelRow, FormulaData, ReadListener, Result, RowData,
};
use crate::read::read_helpers::{
    analysis_context, header_map, is_empty_read_cell, trim_string_cells,
};
#[cfg(test)]
use crate::read::read_helpers::listener_result;
use crate::read::processor::default_analysis_event_processor::DefaultAnalysisEventProcessor;
use crate::read::read_options::ReadOptions;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

include!("row_consumer/read_flow.rs");

include!("row_consumer/source_row_metadata.rs");
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) trait RowConsumer {
    /// 返回消费者是否区分”源 XML 中不存在”与”显式空单元格”。
    fn requires_present_columns(&self) -> bool {
        true
    }

    /// 返回消费者是否需要公式元数据。
    fn requires_formulas(&self) -> bool {
        true
    }

    /// 返回消费者是否需要显示值。
    fn requires_display_values(&self) -> bool {
        true
    }

    /// 返回消费者是否需要精确 decimal 值。
    fn requires_decimal_values(&self) -> bool {
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn process(
        &mut self,
        sheet_no: usize,
        sheet_name: &str,
        row_index: u32,
        cells: Vec<CellValue>,
        metadata: SourceRowMetadata,
        options: &ReadOptions,
        headers: &mut Arc<HashMap<String, usize>>,
    ) -> Result<ReadFlow>;

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<ReadFlow>;

    fn after(&mut self, context: &AnalysisContext) -> Result<()>;

    /// 轻量快路径：跳过 `SourceRowMetadata` 装配，直接处理纯单元格数据。
    ///
    /// 默认实现回退到完整 `process`；`TypedRowConsumer` 会覆盖此方法以避免
    /// 构造空 HashMap/HashSet 和 `SourceRowMetadata`。
    #[allow(clippy::too_many_arguments)]
    fn process_fast(
        &mut self,
        sheet_no: usize,
        sheet_name: &str,
        row_index: u32,
        cells: Vec<CellValue>,
        options: &ReadOptions,
        headers: &mut Arc<HashMap<String, usize>>,
    ) -> Result<ReadFlow> {
        self.process(
            sheet_no,
            sheet_name,
            row_index,
            cells,
            SourceRowMetadata::default(),
            options,
            headers,
        )
    }
}

include!("row_consumer/typed_row_consumer.rs");

#[allow(clippy::too_many_arguments)]
fn process_row_with_metadata<T>(
    sheet_no: usize,
    sheet_name: &str,
    row_index: u32,
    mut cells: Vec<CellValue>,
    metadata: SourceRowMetadata,
    options: &ReadOptions,
    headers: &mut Arc<HashMap<String, usize>>,
    listener: &mut dyn ReadListener<T>,
) -> Result<ReadFlow>
where
    T: ExcelRow,
{
    let SourceRowMetadata {
        formulas,
        display_values,
        decimal_values,
        present_columns,
    } = metadata;
    if options.auto_trim {
        trim_string_cells(&mut cells);
    }
    let context = analysis_context(sheet_name, sheet_no, row_index, options);
    if row_index < options.head_row_number {
        let current_headers = Arc::new(header_map(&cells, &options.header_aliases));
        if row_index + 1 == options.head_row_number {
            *headers = Arc::clone(&current_headers);
        }
        return DefaultAnalysisEventProcessor::dispatch_head(
            listener,
            &current_headers,
            &context,
        );
    }
    if options.ignore_empty_row && cells.iter().all(is_empty_read_cell) {
        return Ok(ReadFlow::Continue);
    }

    let row = RowData::from_stream_parts(
        sheet_name,
        row_index,
        cells,
        Arc::clone(headers),
        formulas,
        display_values,
        decimal_values,
        present_columns,
        options.read_default_return,
        options.use_1904_windowing,
    );
    match T::from_row_with_converters(&row, &options.converters) {
        Ok(data) => {
            DefaultAnalysisEventProcessor::dispatch_data(listener, data, &context)
        }
        Err(error) => DefaultAnalysisEventProcessor::dispatch_error(listener, error, &context),
    }
}

#[cfg(test)]
mod mockall_contract_tests {
    use super::*;
    use crate::core::{ErrorAction, ExcelColumn, ExcelError, ExcelRow, RowData};
    use crate::read::listener::read_listener::MockReadListener;

    /// 最小测试行（ExcelRow 约束仅 `process` 需要；其余契约测试用 `u8`）。
    #[derive(Debug, Clone, PartialEq)]
    struct TestRow(String);

    impl ExcelRow for TestRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
            COLUMNS
        }

        fn from_row(row: &RowData) -> Result<Self> {
            Ok(Self(
                row.cell(&Self::schema()[0])
                    .map_or_else(String::new, CellValue::as_text),
            ))
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::String(self.0.clone())])
        }
    }

    fn context() -> AnalysisContext {
        AnalysisContext::new("Sheet1", 0, 0)
    }

    fn options() -> ReadOptions {
        ReadOptions {
            head_row_number: 1,
            ..ReadOptions::default()
        }
    }

    fn headers() -> Arc<HashMap<String, usize>> {
        Arc::new(HashMap::new())
    }

    fn metadata() -> SourceRowMetadata {
        SourceRowMetadata::default()
    }

    // ---- listener_result / listener_error：hasNext 与 onException 契约 ----

    #[test]
    fn invoke_ok_with_has_next_continues() {
        let mut mock = MockReadListener::<u8>::new();
        mock.expect_has_next().times(1).returning(|_| true);
        let flow = listener_result(Ok(()), &mut mock, &context()).unwrap();
        assert_eq!(flow, ReadFlow::Continue);
    }

    #[test]
    fn invoke_ok_without_has_next_stops() {
        let mut mock = MockReadListener::<u8>::new();
        mock.expect_has_next().times(1).returning(|_| false);
        let flow = listener_result(Ok(()), &mut mock, &context()).unwrap();
        assert_eq!(flow, ReadFlow::Stop);
    }

    #[test]
    fn invoke_error_routes_to_on_exception_continue() {
        let mut mock = MockReadListener::<u8>::new();
        mock.expect_on_exception()
            .times(1)
            .withf(|error, _| matches!(error, ExcelError::Format(_)))
            .returning(|_, _| ErrorAction::Continue);
        let flow = listener_result(
            Err(ExcelError::Format("boom".to_owned())),
            &mut mock,
            &context(),
        )
        .unwrap();
        assert_eq!(flow, ReadFlow::Continue);
    }

    #[test]
    fn invoke_error_on_exception_stop_propagates_error() {
        let mut mock = MockReadListener::<u8>::new();
        mock.expect_on_exception()
            .times(1)
            .returning(|_, _| ErrorAction::Stop);
        let result = listener_result(
            Err(ExcelError::Format("boom".to_owned())),
            &mut mock,
            &context(),
        );
        assert!(result.is_err(), "Stop 动作必须向上传播错误");
    }

    // ---- process：行分发契约（数据行 invoke / 表头行 invoke_head） ----

    #[test]
    fn data_row_dispatches_invoke_once_and_never_invoke_head() {
        let mut mock = MockReadListener::<TestRow>::new();
        mock.expect_invoke()
            .times(1)
            .withf(|data, _| data.0 == "42")
            .returning(|_, _| Ok(()));
        mock.expect_invoke_head().times(0);
        mock.expect_has_next().times(1).returning(|_| true);
        let mut consumer = TypedRowConsumer {
            listener: &mut mock,
        };
        let flow = consumer
            .process(
                0,
                "Sheet1",
                1,
                vec![CellValue::String("42".to_owned())],
                metadata(),
                &options(),
                &mut headers(),
            )
            .unwrap();
        assert_eq!(flow, ReadFlow::Continue);
    }

    #[test]
    fn head_row_dispatches_invoke_head_once_and_never_invoke() {
        let mut mock = MockReadListener::<TestRow>::new();
        mock.expect_invoke().times(0);
        mock.expect_invoke_head().times(1).returning(|_, _| Ok(()));
        mock.expect_has_next().times(1).returning(|_| true);
        let mut consumer = TypedRowConsumer {
            listener: &mut mock,
        };
        let flow = consumer
            .process(
                0,
                "Sheet1",
                0,
                vec![CellValue::String("head".to_owned())],
                metadata(),
                &options(),
                &mut headers(),
            )
            .unwrap();
        assert_eq!(flow, ReadFlow::Continue);
    }

    #[test]
    fn invoke_error_during_process_routes_to_on_exception() {
        let mut mock = MockReadListener::<TestRow>::new();
        mock.expect_invoke()
            .times(1)
            .returning(|_, _| Err(ExcelError::Format("row failed".to_owned())));
        mock.expect_on_exception()
            .times(1)
            .returning(|_, _| ErrorAction::Continue);
        let mut consumer = TypedRowConsumer {
            listener: &mut mock,
        };
        let flow = consumer
            .process(
                0,
                "Sheet1",
                1,
                vec![CellValue::String("42".to_owned())],
                metadata(),
                &options(),
                &mut headers(),
            )
            .unwrap();
        assert_eq!(flow, ReadFlow::Continue, "Continue 动作继续读取");
    }

    // ---- extra / after：附加事件与收尾契约 ----

    #[test]
    fn extra_event_dispatches_to_listener_extra_once() {
        let mut mock = MockReadListener::<TestRow>::new();
        mock.expect_extra().times(1).returning(|_, _| Ok(()));
        // extra 分发成功后管线同样执行 hasNext 停机检查（与 invoke 分支一致）
        mock.expect_has_next().times(1).returning(|_| true);
        let mut consumer = TypedRowConsumer {
            listener: &mut mock,
        };
        let extra = CellExtra::new(
            crate::core::CellExtraType::Comment,
            Some("note".to_owned()),
            0,
            0,
            0,
            0,
        );
        let flow = consumer.extra(&extra, &context()).unwrap();
        assert_eq!(flow, ReadFlow::Continue);
    }

    #[test]
    fn after_calls_do_after_all_analysed_once() {
        let mut mock = MockReadListener::<TestRow>::new();
        mock.expect_do_after_all_analysed()
            .times(1)
            .returning(|_| Ok(()));
        let mut consumer = TypedRowConsumer {
            listener: &mut mock,
        };
        consumer.after(&context()).unwrap();
    }
}
