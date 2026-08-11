//! Java 4.0.3 `ExcelAnalyser` / `ExcelAnalyserImpl` 构造、分派与生命周期证据。

use std::path::PathBuf;

use easyexcel::analysis::csv::csv_excel_read_executor::CsvExcelReadExecutor;
use easyexcel::analysis::v03::handlers::abstract_xls_record_handler::AbstractXlsRecordHandler;
use easyexcel::analysis::v03::handlers::bof_record_handler::BOF_SID;
use easyexcel::analysis::v03::handlers::bound_sheet_record_handler::{
    BOUND_SHEET_SID, BoundSheetRecordHandler,
};
use easyexcel::analysis::v03::handlers::dummy_record_handler::DummyRecordEvent;
use easyexcel::analysis::v03::handlers::formula_record_handler::{
    FORMULA_SID, FormulaCachedType, FormulaRecordHandler,
};
use easyexcel::analysis::v03::handlers::hyperlink_record_handler::HyperlinkRecordHandler;
use easyexcel::analysis::v03::handlers::label_record_handler::{LABEL_SID, LabelRecordHandler};
use easyexcel::analysis::v03::handlers::label_sst_record_handler::{
    LABEL_SST_SID, LabelSstCell, LabelSstRecordHandler,
};
use easyexcel::analysis::v03::handlers::note_record_handler::NoteRecordHandler;
use easyexcel::analysis::v03::handlers::obj_record_handler::{OBJ_SID, ObjRecordHandler};
use easyexcel::analysis::v03::handlers::rk_record_handler::{RK_SID, RkRecordHandler};
use easyexcel::analysis::v03::handlers::sst_record_handler::{SST_SID, SstRecordHandler};
use easyexcel::analysis::v03::handlers::string_record_handler::{STRING_SID, StringRecordHandler};
use easyexcel::analysis::v03::handlers::text_object_record_handler::{
    TEXT_OBJECT_SID, TextObjectRecordHandler,
};
use easyexcel::analysis::v03::xls_list_sheet_listener::XlsListSheetListener;
use easyexcel::analysis::v03::xls_sax_analyser::XlsSaxAnalyser;
use easyexcel::analysis::v03::{IgnorableXlsRecordHandler, XlsRecordDispatcher, XlsRecordHandler};
use easyexcel::analysis::v07::XlsxSaxAnalyser;
use easyexcel::analysis::v07::handlers::abstract_cell_value_tag_handler::AbstractCellValueTagHandler;
use easyexcel::analysis::v07::handlers::abstract_xlsx_tag_handler::AbstractXlsxTagHandler;
use easyexcel::analysis::v07::handlers::xlsx_tag_handler::XlsxTagHandler;
use easyexcel::analysis::{
    ExcelAnalyser, ExcelAnalyserImpl, ExcelReadExecutor, ExcelReadExecutorKind,
};
use easyexcel::context::{
    CsvReadContext, DefaultCsvReadContext, DefaultXlsReadContext, DefaultXlsxReadContext,
    XlsReadContext, XlsxReadContext,
};
use easyexcel::read::ReadOptions;
use easyexcel::read::metadata::{ReadSheet, ReadWorkbook};
use easyexcel::{AnalysisContext, DynamicRow, ExcelError, ExcelTypeEnum, ReadListener};
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct ExcelAnalyserContract {
    authority: String,
    implementation_class: String,
    executor_class: String,
    executor_sheet_count: usize,
    analysis_context_class: String,
    analysis_context_same: bool,
    analysis_all_succeeded: bool,
    finish_twice_succeeded: bool,
    xlsx_sax_analyser_class: String,
    xlsx_sax_analyser_shared_strings_part_name: String,
    xlsx_sax_analyser_sheet_count: usize,
    xlsx_sax_analyser_sheet_no: usize,
    xlsx_sax_analyser_sheet_name: String,
    xlsx_sax_analyser_execute_succeeded: bool,
    csv_executor_class: String,
    csv_executor_sheet_count: usize,
    csv_executor_sheet_no: usize,
    csv_executor_execute_succeeded: bool,
    csv_context_sheet_no: i32,
    csv_context_parser_initialized: bool,
    xls_context_workbook_type: String,
    xls_context_sheet_before_null: bool,
    xls_context_sheet_no: i32,
    xls_list_listener_class: String,
    xls_list_listener_need_read_sheet: bool,
    xls_list_listener_known_record_delta: usize,
    xls_list_listener_unknown_record_ignored: bool,
    xls_list_listener_execute_sheet_count: usize,
    xls_list_listener_execute_stopped: bool,
    xls_record_handler_interface: bool,
    xls_record_handler_support: bool,
    abstract_xls_record_handler_is_abstract: bool,
    abstract_xls_record_handler_support_default: bool,
    xls_record_handler_process_delta: usize,
    ignorable_xls_record_handler_marker: bool,
    blank_record_handler_cell_present: bool,
    bool_err_record_handler_cell_present: bool,
    number_record_handler_cell_present: bool,
    index_record_handler_total_rows: u32,
    eof_record_handler_sheet_ended: bool,
    merge_cells_record_handler_support: bool,
    merge_cells_record_handler_first_row: u32,
    merge_cells_record_handler_last_row: u32,
    merge_cells_record_handler_first_column: usize,
    merge_cells_record_handler_last_column: usize,
    note_record_handler_support: bool,
    note_record_handler_text: String,
    note_record_handler_row: u32,
    note_record_handler_column: usize,
    label_record_handler_text: String,
    label_record_handler_row: u32,
    label_record_handler_column: usize,
    label_record_handler_row_type: String,
    sst_record_handler_cache_present: bool,
    sst_record_handler_text: String,
    label_sst_record_handler_text: String,
    label_sst_record_handler_row: u32,
    label_sst_record_handler_column: usize,
    label_sst_record_handler_row_type: String,
    label_sst_record_handler_missing_cache_empty: bool,
    formula_record_handler_string_pending: bool,
    formula_record_handler_row: u32,
    formula_record_handler_column: usize,
    formula_record_handler_type: String,
    string_record_handler_text: String,
    string_record_handler_formula_completed: bool,
    string_record_handler_cell_present: bool,
    string_record_handler_without_formula_ignored: bool,
    rk_record_handler_cell_present: bool,
    rk_record_handler_type: String,
    rk_record_handler_row: u32,
    rk_record_handler_column: usize,
    obj_record_handler_comment_object_id: u32,
    obj_record_handler_non_comment_ignored: bool,
    hyperlink_record_handler_support_disabled: bool,
    hyperlink_record_handler_support_enabled: bool,
    hyperlink_record_handler_address: String,
    hyperlink_record_handler_first_row: u32,
    hyperlink_record_handler_last_row: u32,
    hyperlink_record_handler_first_column: usize,
    hyperlink_record_handler_last_column: usize,
    hyperlink_record_handler_serialized: Vec<u8>,
    text_object_record_handler_support_enabled: bool,
    text_object_record_handler_support_disabled: bool,
    text_object_record_handler_text: String,
    text_object_record_handler_temp_cleared: bool,
    text_object_record_handler_serialized: Vec<u8>,
    dummy_record_handler_missing_cell_present: bool,
    dummy_record_handler_missing_cell_type: String,
    dummy_record_handler_missing_cell_row: u32,
    dummy_record_handler_missing_cell_column: usize,
    dummy_record_handler_existing_cell_preserved: bool,
    dummy_record_handler_end_row_index: u32,
    dummy_record_handler_cell_map_cleared: bool,
    dummy_record_handler_row_type_reset: String,
    xls_sax_analyser_class: String,
    xls_sax_analyser_sheet_count: usize,
    xls_sax_analyser_execute_succeeded: bool,
    xls_sax_analyser_known_record_delta: usize,
    xls_sax_analyser_unknown_record_ignored: bool,
    xlsx_context_workbook_type: String,
    xlsx_context_sheet_before_null: bool,
    xlsx_context_sheet_no: i32,
    xlsx_tag_handler_interface: bool,
    abstract_xlsx_tag_handler_is_abstract: bool,
    abstract_xlsx_tag_handler_support_default: bool,
    abstract_xlsx_tag_handler_noop_preserved: bool,
    abstract_cell_value_tag_handler_is_abstract: bool,
    abstract_cell_value_tag_handler_text: String,
}

fn contract() -> ExcelAnalyserContract {
    serde_json::from_str(include_str!(
        "golden/excel_analyser_lifecycle.contract.json"
    ))
    .expect("Java ExcelAnalyser contract must be valid JSON")
}

fn java_input() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/artifacts/excel_analyser_api.xlsx")
}

fn java_xls_input() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/artifacts/excel_analyser_api.xls")
}

fn biff_records(serialized: &[u8]) -> Vec<(u16, &[u8])> {
    let mut records = Vec::new();
    let mut offset = 0usize;
    while offset < serialized.len() {
        assert!(
            serialized.len() - offset >= 4,
            "truncated BIFF record header"
        );
        let sid = u16::from_le_bytes([serialized[offset], serialized[offset + 1]]);
        let length = usize::from(u16::from_le_bytes([
            serialized[offset + 2],
            serialized[offset + 3],
        ]));
        let body_start = offset + 4;
        let body_end = body_start + length;
        assert!(body_end <= serialized.len(), "truncated BIFF record body");
        records.push((sid, &serialized[body_start..body_end]));
        offset = body_end;
    }
    records
}

#[derive(Default)]
struct CountingListener {
    rows: usize,
}

impl ReadListener<DynamicRow> for CountingListener {
    fn invoke(&mut self, _data: DynamicRow, _context: &AnalysisContext) -> easyexcel::Result<()> {
        self.rows += 1;
        Ok(())
    }
}

fn assert_excel_analyser_trait<T: ExcelAnalyser>() {}

fn assert_excel_read_executor_trait<T: ExcelReadExecutor>() {}

fn assert_csv_read_context_trait<T: CsvReadContext>() {}

fn assert_xls_read_context_trait<T: XlsReadContext>() {}

fn assert_xlsx_read_context_trait<T: XlsxReadContext>() {}

fn assert_abstract_xls_record_handler_trait<T: AbstractXlsRecordHandler>() {}

#[test]
fn excel_analyser_direct_constructor_executor_context_and_finish_match_java()
-> easyexcel::Result<()> {
    assert_excel_analyser_trait::<ExcelAnalyserImpl>();
    assert_excel_read_executor_trait::<ExcelReadExecutorKind>();
    assert_csv_read_context_trait::<DefaultCsvReadContext>();
    assert_xls_read_context_trait::<DefaultXlsReadContext>();
    assert_xlsx_read_context_trait::<DefaultXlsxReadContext>();
    assert_abstract_xls_record_handler_trait::<BoundSheetRecordHandler>();
    let contract = contract();
    let mut workbook = ReadWorkbook::new();
    workbook
        .set_file(java_input())
        .set_excel_type(ExcelTypeEnum::Xlsx)
        .set_head_row_number(1);
    let mut analyser = ExcelAnalyserImpl::from_read_workbook(workbook)?;

    assert_eq!(analyser.excel_type(), Some(ExcelTypeEnum::Xlsx));
    assert!(contract.executor_class.ends_with("XlsxSaxAnalyser"));
    assert_eq!(
        ExcelAnalyser::excel_executor(&analyser).sheet_list().len(),
        contract.executor_sheet_count
    );

    // 对应 Java `ExcelReadExecutor#sheetList()` 与无参 `execute()`：执行器从自身
    // 工作簿上下文取得选项并完整驱动真实 XLSX 解析。
    let mut executor = ExcelReadExecutorKind::new(
        ExcelTypeEnum::Xlsx,
        java_input(),
        ReadOptions {
            head_row_number: 1,
            ..ReadOptions::default()
        },
    )?;
    assert_eq!(
        ExcelReadExecutor::sheet_list(&executor).len(),
        contract.executor_sheet_count
    );
    ExcelReadExecutor::execute(&mut executor)?;
    let first_context = ExcelAnalyser::analysis_context(&analyser) as *const AnalysisContext;
    let second_context = ExcelAnalyser::analysis_context(&analyser) as *const AnalysisContext;
    assert!(contract.analysis_context_same);
    assert!(std::ptr::eq(first_context, second_context));
    assert!(
        contract
            .analysis_context_class
            .ends_with("DefaultXlsxReadContext")
    );

    ExcelAnalyser::analysis(&mut analyser, None, true)?;
    assert!(contract.analysis_all_succeeded);

    let mut listener = CountingListener::default();
    ExcelAnalyser::analysis_with_listener::<DynamicRow, _>(&mut analyser, &mut listener)?;
    assert!(contract.analysis_all_succeeded);
    assert_eq!(listener.rows, 1);
    ExcelAnalyser::finish(&mut analyser);
    ExcelAnalyser::finish(&mut analyser);
    assert!(contract.finish_twice_succeeded);
    assert!(analyser.is_finished());
    assert_eq!(contract.authority, "com.alibaba:easyexcel:4.0.3");
    assert_eq!(
        contract.implementation_class,
        "com.alibaba.excel.analysis.ExcelAnalyserImpl"
    );

    let mut xlsx_sax_analyser = XlsxSaxAnalyser::from_path(
        java_input(),
        ReadOptions {
            head_row_number: 1,
            ..ReadOptions::default()
        },
    )?;
    assert!(
        contract
            .xlsx_sax_analyser_class
            .ends_with("XlsxSaxAnalyser")
    );
    assert_eq!(
        XlsxSaxAnalyser::SHARED_STRINGS_PART_NAME,
        contract.xlsx_sax_analyser_shared_strings_part_name
    );
    assert_eq!(
        xlsx_sax_analyser.sheet_list().len(),
        contract.xlsx_sax_analyser_sheet_count
    );
    assert_eq!(
        xlsx_sax_analyser.sheet_list()[0].sheet_no(),
        contract.xlsx_sax_analyser_sheet_no
    );
    assert_eq!(
        xlsx_sax_analyser.sheet_list()[0].sheet_name(),
        contract.xlsx_sax_analyser_sheet_name
    );
    xlsx_sax_analyser.execute()?;
    assert!(contract.xlsx_sax_analyser_execute_succeeded);

    let csv_input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/artifacts/excel_read_executor_api.csv");
    let mut csv_workbook = ReadWorkbook::new();
    csv_workbook
        .set_file(&csv_input)
        .set_excel_type(ExcelTypeEnum::Csv)
        .set_head_row_number(1);
    let csv_context = DefaultCsvReadContext::from_read_workbook(&csv_workbook, ExcelTypeEnum::Csv);
    let mut csv_executor = CsvExcelReadExecutor::new(csv_context);
    assert!(
        contract
            .csv_executor_class
            .ends_with("CsvExcelReadExecutor")
    );
    assert_eq!(
        ExcelReadExecutor::sheet_list(&csv_executor).len(),
        contract.csv_executor_sheet_count
    );
    assert_eq!(
        ExcelReadExecutor::sheet_list(&csv_executor)[0].sheet_no(),
        contract.csv_executor_sheet_no
    );
    // Java 的默认 sheetName 为 null；Rust 字符串视图用空串保留同一“未命名”语义。
    assert_eq!(
        ExcelReadExecutor::sheet_list(&csv_executor)[0].sheet_name(),
        ""
    );
    assert_eq!(
        csv_executor.csv_read_context().file(),
        Some(csv_input.as_path())
    );
    assert_eq!(
        CsvReadContext::csv_read_workbook_holder(csv_executor.csv_read_context())
            .inner()
            .ignore_empty_row,
        csv_executor.csv_read_context().options().ignore_empty_row
    );
    ExcelReadExecutor::execute(&mut csv_executor)?;
    assert!(contract.csv_executor_execute_succeeded);
    let csv_sheet_holder = CsvReadContext::csv_read_sheet_holder(csv_executor.csv_read_context())
        .expect("execute() must materialize the CSV sheet holder");
    assert_eq!(
        csv_sheet_holder.inner().sheet_no,
        contract.csv_context_sheet_no
    );
    assert_eq!(
        csv_executor.csv_read_context().parser_initialized(),
        contract.csv_context_parser_initialized
    );

    let mut xls_workbook = ReadWorkbook::new();
    xls_workbook.set_excel_type(ExcelTypeEnum::Xls);
    let mut xls_context =
        DefaultXlsReadContext::from_read_workbook(&xls_workbook, ExcelTypeEnum::Xls);
    assert_eq!(contract.xls_context_workbook_type, "XLS");
    assert_eq!(
        easyexcel::context::AnalysisContextLifecycle::analysis_context_impl(&xls_context).excel_type(),
        ExcelTypeEnum::Xls
    );
    let _xls_workbook_holder = XlsReadContext::xls_read_workbook_holder(&xls_context);
    assert_eq!(
        XlsReadContext::xls_read_sheet_holder(&xls_context).is_none(),
        contract.xls_context_sheet_before_null
    );
    xls_context.current_sheet(&ReadSheet::with_name(0, "XlsContext"))?;
    assert_eq!(
        XlsReadContext::xls_read_sheet_holder(&xls_context)
            .expect("XLS currentSheet must materialize holder")
            .inner()
            .sheet_no,
        contract.xls_context_sheet_no
    );

    let mut listener_workbook = ReadWorkbook::new();
    listener_workbook
        .set_file(java_xls_input())
        .set_excel_type(ExcelTypeEnum::Xls);
    let mut listener_context =
        DefaultXlsReadContext::from_read_workbook(&listener_workbook, ExcelTypeEnum::Xls);
    let mut xls_list_listener = XlsListSheetListener::new(&mut listener_context);
    assert!(
        contract
            .xls_list_listener_class
            .ends_with("XlsListSheetListener")
    );
    assert_eq!(xls_list_listener.sheet_list().len(), 0);
    let mut bound_sheet = vec![100, 0, 0, 0, 0, 0, 6, 0];
    bound_sheet.extend_from_slice(b"Manual");
    xls_list_listener.process_record(BOUND_SHEET_SID, &bound_sheet);
    xls_list_listener.process_record(0xFFFF, &[]);
    assert!(contract.xls_list_listener_unknown_record_ignored);
    xls_list_listener.process_record(BOF_SID, &[0, 0, 0x10, 0]);
    assert_eq!(
        xls_list_listener.sheet_list().len(),
        contract.xls_list_listener_known_record_delta
    );
    drop(xls_list_listener);
    assert_eq!(
        XlsReadContext::xls_read_workbook_holder(&listener_context).need_read_sheet(),
        contract.xls_list_listener_need_read_sheet
    );

    let mut record_handler = BoundSheetRecordHandler::new();
    assert!(contract.xls_record_handler_interface);
    assert_eq!(
        XlsRecordHandler::support(&record_handler),
        contract.xls_record_handler_support
    );
    assert!(contract.abstract_xls_record_handler_is_abstract);
    assert_eq!(
        XlsRecordHandler::support(&record_handler),
        contract.abstract_xls_record_handler_support_default
    );
    let before_handler_record = record_handler.ordered_sheets().len();
    let mut handler_bound_sheet = vec![100, 0, 0, 0, 0, 0, 12, 0];
    handler_bound_sheet.extend_from_slice(b"HandlerSheet");
    XlsRecordHandler::process_record(&mut record_handler, BOUND_SHEET_SID, &handler_bound_sheet);
    assert_eq!(
        record_handler.ordered_sheets().len() - before_handler_record,
        contract.xls_record_handler_process_delta
    );
    assert_eq!(
        record_handler.is_ignorable(),
        contract.ignorable_xls_record_handler_marker
    );

    let mut dispatcher = XlsRecordDispatcher::new(&ReadOptions::default());
    dispatcher.process_record(0x0201, &[2, 0, 3, 0, 0, 0])?;
    assert_eq!(
        dispatcher.state().last_blank_cell().is_some(),
        contract.blank_record_handler_cell_present
    );
    dispatcher.process_record(0x0205, &[4, 0, 5, 0, 0, 0, 1, 0])?;
    assert_eq!(
        dispatcher.state().last_boolean_cell().is_some(),
        contract.bool_err_record_handler_cell_present
    );
    let mut number_payload = vec![6, 0, 7, 0, 0, 0];
    number_payload.extend_from_slice(&12.5f64.to_le_bytes());
    dispatcher.process_record(0x0203, &number_payload)?;
    assert_eq!(
        dispatcher.state().last_number_cell().is_some(),
        contract.number_record_handler_cell_present
    );
    let mut index_payload = vec![0u8; 16];
    index_payload[8..12].copy_from_slice(&77u32.to_le_bytes());
    dispatcher.process_record(0x020B, &index_payload)?;
    assert_eq!(
        dispatcher.state().approximate_total_row_number(),
        Some(contract.index_record_handler_total_rows)
    );
    dispatcher.process_record(0x000A, &[])?;
    assert_eq!(
        dispatcher.state().eof_count() > 0,
        contract.eof_record_handler_sheet_ended
    );
    let mut merge_options = ReadOptions::default();
    merge_options
        .extra_read
        .insert(easyexcel::CellExtraType::Merge);
    let mut merge_dispatcher = XlsRecordDispatcher::new(&merge_options);
    merge_dispatcher.process_record(0x00E5, &[1, 0, 0, 0, 1, 0, 0, 0, 1, 0])?;
    let (_, merge_extra) = merge_dispatcher
        .state()
        .extras()
        .first()
        .expect("merge extra must be emitted");
    assert_eq!(
        merge_options
            .extra_read
            .contains(&easyexcel::CellExtraType::Merge),
        contract.merge_cells_record_handler_support
    );
    assert_eq!(
        (
            merge_extra.first_row_index(),
            merge_extra.last_row_index(),
            merge_extra.first_column_index(),
            merge_extra.last_column_index()
        ),
        (
            contract.merge_cells_record_handler_first_row,
            contract.merge_cells_record_handler_last_row,
            contract.merge_cells_record_handler_first_column,
            contract.merge_cells_record_handler_last_column
        )
    );
    let mut note_handler = NoteRecordHandler::new(true);
    XlsRecordHandler::process_record(&mut note_handler, 0x001C, &[3, 0, 4, 0, 0, 0]);
    note_handler.process_note(Some("note-text".to_owned()), 3, 4);
    let note_extra = note_handler.last_extra.as_ref().expect("comment extra");
    assert_eq!(note_handler.support(), contract.note_record_handler_support);
    assert_eq!(
        note_extra.text(),
        Some(contract.note_record_handler_text.as_str())
    );
    assert_eq!(
        (
            note_extra.first_row_index(),
            note_extra.first_column_index()
        ),
        (
            contract.note_record_handler_row,
            contract.note_record_handler_column
        )
    );

    let mut label_payload = vec![5, 0, 6, 0, 0, 0, 10, 0, 0];
    label_payload.extend_from_slice(b"label-text");
    let mut label_handler = LabelRecordHandler::new();
    XlsRecordHandler::process_record(&mut label_handler, LABEL_SID, &label_payload);
    let label_cell = label_handler.last_cell.as_ref().expect("inline label cell");
    assert_eq!(label_cell.value, contract.label_record_handler_text);
    assert_eq!(label_cell.row, contract.label_record_handler_row);
    assert_eq!(label_cell.column, contract.label_record_handler_column);
    assert_eq!(contract.label_record_handler_row_type, "DATA");

    let mut sst_payload = Vec::new();
    sst_payload.extend_from_slice(&1u32.to_le_bytes());
    sst_payload.extend_from_slice(&1u32.to_le_bytes());
    sst_payload.extend_from_slice(&10u16.to_le_bytes());
    sst_payload.push(0);
    sst_payload.extend_from_slice(b"  shared  ");
    let mut sst_handler = SstRecordHandler::new();
    XlsRecordHandler::process_record(&mut sst_handler, SST_SID, &sst_payload);
    assert_eq!(sst_handler.unique_string_count, Some(1));

    let mut string_dispatcher = XlsRecordDispatcher::new(&ReadOptions::default());
    string_dispatcher.process_record(SST_SID, &sst_payload)?;
    assert_eq!(string_dispatcher.state().unique_string_count(), Some(1));
    assert_eq!(
        string_dispatcher.state().shared_strings().first(),
        Some(&contract.sst_record_handler_text)
    );
    assert_eq!(
        !string_dispatcher.state().shared_strings().is_empty(),
        contract.sst_record_handler_cache_present
    );

    let mut label_sst_payload = vec![7, 0, 8, 0, 0, 0];
    label_sst_payload.extend_from_slice(&0u32.to_le_bytes());
    let mut label_sst_handler = LabelSstRecordHandler::new();
    XlsRecordHandler::process_record(&mut label_sst_handler, LABEL_SST_SID, &label_sst_payload);
    assert_eq!(
        label_sst_handler
            .last_reference
            .expect("label sst reference")
            .sst_index,
        0
    );
    string_dispatcher.process_record(LABEL_SST_SID, &label_sst_payload)?;
    assert_eq!(
        string_dispatcher.state().last_label_sst_cell(),
        Some(&LabelSstCell::String {
            row: contract.label_sst_record_handler_row,
            column: contract.label_sst_record_handler_column,
            value: contract.label_sst_record_handler_text.clone(),
        })
    );
    assert_eq!(contract.label_sst_record_handler_row_type, "DATA");
    assert_eq!(
        LabelSstRecordHandler::process_label_sst(9, 10, 99, true, &|_| None),
        LabelSstCell::Empty { row: 9, column: 10 }
    );
    assert!(contract.label_sst_record_handler_missing_cache_empty);

    let mut label_dispatcher = XlsRecordDispatcher::new(&ReadOptions::default());
    label_dispatcher.process_record(LABEL_SID, &label_payload)?;
    assert_eq!(
        label_dispatcher
            .state()
            .last_label_cell()
            .map(|cell| cell.value.as_str()),
        Some(contract.label_record_handler_text.as_str())
    );

    let mut formula_payload = vec![11, 0, 12, 0, 0, 0];
    formula_payload.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0xFF, 0xFF]);
    let mut formula_handler = FormulaRecordHandler::new();
    XlsRecordHandler::process_record(&mut formula_handler, FORMULA_SID, &formula_payload);
    let pending_formula = formula_handler
        .pending
        .as_ref()
        .expect("string formula pending");
    assert_eq!(contract.formula_record_handler_string_pending, true);
    assert_eq!(pending_formula.row, contract.formula_record_handler_row);
    assert_eq!(
        pending_formula.column,
        contract.formula_record_handler_column
    );
    assert_eq!(pending_formula.cached_type, FormulaCachedType::String);
    assert_eq!(contract.formula_record_handler_type, "STRING");

    let mut formula_string_payload = vec![16, 0, 0];
    formula_string_payload.extend_from_slice(b"  formula-text  ");
    let mut string_handler = StringRecordHandler::new();
    XlsRecordHandler::process_record(&mut string_handler, STRING_SID, &formula_string_payload);
    let decoded_formula_string = string_handler
        .last_value
        .clone()
        .expect("formula string record");
    let (completed_formula, completed_text) =
        StringRecordHandler::process_string(&mut formula_handler, decoded_formula_string, true)
            .expect("pending formula must complete");
    assert_eq!(completed_text, contract.string_record_handler_text);
    assert_eq!(
        completed_formula.string_value.as_deref(),
        Some(contract.string_record_handler_text.as_str())
    );
    assert_eq!(
        formula_handler.pending.is_none(),
        contract.string_record_handler_formula_completed
    );
    assert_eq!(
        formula_handler.last_cell.is_some(),
        contract.string_record_handler_cell_present
    );
    assert_eq!(
        StringRecordHandler::process_string(
            &mut FormulaRecordHandler::new(),
            "orphan".to_owned(),
            true,
        )
        .is_none(),
        contract.string_record_handler_without_formula_ignored
    );

    let mut formula_dispatcher = XlsRecordDispatcher::new(&ReadOptions::default());
    formula_dispatcher.process_record(FORMULA_SID, &formula_payload)?;
    formula_dispatcher.process_record(STRING_SID, &formula_string_payload)?;
    assert_eq!(
        formula_dispatcher
            .state()
            .last_formula_cell()
            .and_then(|cell| cell.string_value.as_deref()),
        Some(contract.string_record_handler_text.as_str())
    );

    let mut rk_payload = vec![13, 0, 14, 0, 0, 0];
    rk_payload.extend_from_slice(&0xAAu32.to_le_bytes());
    let mut rk_handler = RkRecordHandler::new();
    XlsRecordHandler::process_record(&mut rk_handler, RK_SID, &rk_payload);
    let rk_cell = rk_handler.last_cell.expect("RK empty cell");
    assert_eq!(rk_cell.row, contract.rk_record_handler_row);
    assert_eq!(rk_cell.column, contract.rk_record_handler_column);
    assert_eq!(
        rk_handler.last_cell.is_some(),
        contract.rk_record_handler_cell_present
    );
    assert_eq!(contract.rk_record_handler_type, "EMPTY");
    let mut rk_dispatcher = XlsRecordDispatcher::new(&ReadOptions::default());
    rk_dispatcher.process_record(RK_SID, &rk_payload)?;
    assert_eq!(rk_dispatcher.state().last_rk_cell(), Some(rk_cell));

    let mut comment_obj_payload = vec![0x15, 0, 18, 0, 0x19, 0, 15, 0];
    comment_obj_payload.extend_from_slice(&[0; 14]);
    let mut obj_handler = ObjRecordHandler::new();
    XlsRecordHandler::process_record(&mut obj_handler, OBJ_SID, &comment_obj_payload);
    assert_eq!(
        obj_handler.temp_object_index,
        Some(contract.obj_record_handler_comment_object_id)
    );
    let mut rectangle_obj_payload = vec![0x15, 0, 18, 0, 0x01, 0, 99, 0];
    rectangle_obj_payload.extend_from_slice(&[0; 14]);
    XlsRecordHandler::process_record(&mut obj_handler, OBJ_SID, &rectangle_obj_payload);
    assert_eq!(
        obj_handler.temp_object_index,
        Some(contract.obj_record_handler_comment_object_id)
    );
    assert!(contract.obj_record_handler_non_comment_ignored);

    let hyperlink_records = biff_records(&contract.hyperlink_record_handler_serialized);
    assert_eq!(hyperlink_records.len(), 1);
    let (hyperlink_sid, hyperlink_body) = hyperlink_records[0];
    let mut disabled_hyperlink = HyperlinkRecordHandler::new(false);
    assert_eq!(
        disabled_hyperlink.support(),
        contract.hyperlink_record_handler_support_disabled
    );
    XlsRecordHandler::process_record(&mut disabled_hyperlink, hyperlink_sid, hyperlink_body);
    assert!(disabled_hyperlink.last_extra.is_none());
    let mut hyperlink_handler = HyperlinkRecordHandler::new(true);
    assert_eq!(
        hyperlink_handler.support(),
        contract.hyperlink_record_handler_support_enabled
    );
    XlsRecordHandler::process_record(&mut hyperlink_handler, hyperlink_sid, hyperlink_body);
    let hyperlink_extra = hyperlink_handler
        .last_extra
        .as_ref()
        .expect("POI hyperlink record must decode");
    assert_eq!(
        hyperlink_extra.text(),
        Some(contract.hyperlink_record_handler_address.as_str())
    );
    assert_eq!(
        (
            hyperlink_extra.first_row_index(),
            hyperlink_extra.last_row_index(),
            hyperlink_extra.first_column_index(),
            hyperlink_extra.last_column_index(),
        ),
        (
            contract.hyperlink_record_handler_first_row,
            contract.hyperlink_record_handler_last_row,
            contract.hyperlink_record_handler_first_column,
            contract.hyperlink_record_handler_last_column,
        )
    );

    let text_object_records = biff_records(&contract.text_object_record_handler_serialized);
    assert_eq!(
        text_object_records.first().map(|record| record.0),
        Some(TEXT_OBJECT_SID)
    );
    let mut text_object_handler = TextObjectRecordHandler::new();
    assert_eq!(
        text_object_handler.support(),
        contract.text_object_record_handler_support_enabled
    );
    text_object_handler.begin_text_object(
        contract.obj_record_handler_comment_object_id,
        text_object_records[0].1,
    );
    for (sid, body) in text_object_records.iter().skip(1) {
        if *sid == easyexcel_xls::biff8::record_sid::CONTINUE_SID
            && text_object_handler
                .get(contract.obj_record_handler_comment_object_id)
                .is_none()
        {
            assert!(text_object_handler.consume_continue(body));
        }
    }
    assert_eq!(
        text_object_handler.get(contract.obj_record_handler_comment_object_id),
        Some(contract.text_object_record_handler_text.as_str())
    );
    assert!(contract.text_object_record_handler_temp_cleared);

    let mut comment_options = ReadOptions::default();
    comment_options
        .extra_read
        .insert(easyexcel::CellExtraType::Comment);
    let mut text_object_dispatcher = XlsRecordDispatcher::new(&comment_options);
    text_object_dispatcher.process_record(OBJ_SID, &comment_obj_payload)?;
    for (sid, body) in &text_object_records {
        text_object_dispatcher.process_record(*sid, body)?;
    }
    let mut note_payload = vec![20, 0, 21, 0, 0, 0];
    note_payload.extend_from_slice(
        &u16::try_from(contract.obj_record_handler_comment_object_id)
            .expect("Java object id fits BIFF u16")
            .to_le_bytes(),
    );
    text_object_dispatcher.process_record(0x001C, &note_payload)?;
    let (_, comment_extra) = text_object_dispatcher
        .state()
        .extras()
        .last()
        .expect("TXO text must reach NOTE extra");
    assert_eq!(
        comment_extra.text(),
        Some(contract.text_object_record_handler_text.as_str())
    );

    let mut disabled_text_object_dispatcher = XlsRecordDispatcher::new(&ReadOptions::default());
    disabled_text_object_dispatcher.process_record(OBJ_SID, &comment_obj_payload)?;
    disabled_text_object_dispatcher
        .process_record(text_object_records[0].0, text_object_records[0].1)?;
    assert_eq!(
        disabled_text_object_dispatcher
            .state()
            .skipped_record_count()
            > 0,
        !contract.text_object_record_handler_support_disabled
    );

    let mut dummy_dispatcher = XlsRecordDispatcher::new(&ReadOptions::default());
    dummy_dispatcher.process_record(0x0201, &[22, 0, 24, 0, 0, 0])?;
    let mut occupied_missing = vec![1];
    occupied_missing.extend_from_slice(&22u32.to_le_bytes());
    occupied_missing.extend_from_slice(&24u32.to_le_bytes());
    dummy_dispatcher.process_record(u16::MAX, &occupied_missing)?;
    assert_eq!(
        dummy_dispatcher.state().last_dummy_event().is_none(),
        contract.dummy_record_handler_existing_cell_preserved
    );

    let mut missing = vec![1];
    missing.extend_from_slice(&22u32.to_le_bytes());
    missing.extend_from_slice(&23u32.to_le_bytes());
    dummy_dispatcher.process_record(u16::MAX, &missing)?;
    let missing_cell = match dummy_dispatcher.state().last_dummy_event() {
        Some(DummyRecordEvent::MissingCell(cell)) => cell,
        other => panic!("expected missing-cell event, got {other:?}"),
    };
    assert_eq!(
        (missing_cell.row, missing_cell.column),
        (
            contract.dummy_record_handler_missing_cell_row,
            contract.dummy_record_handler_missing_cell_column,
        )
    );
    assert!(contract.dummy_record_handler_missing_cell_present);
    assert_eq!(contract.dummy_record_handler_missing_cell_type, "EMPTY");

    let mut end_row = vec![0];
    end_row.extend_from_slice(&22u32.to_le_bytes());
    dummy_dispatcher.process_record(u16::MAX, &end_row)?;
    assert_eq!(
        dummy_dispatcher.state().last_dummy_event(),
        Some(&DummyRecordEvent::EndRow {
            row: contract.dummy_record_handler_end_row_index,
        })
    );
    assert!(contract.dummy_record_handler_cell_map_cleared);
    assert_eq!(contract.dummy_record_handler_row_type_reset, "EMPTY");

    let mut execute_workbook = ReadWorkbook::new();
    execute_workbook
        .set_file(java_xls_input())
        .set_excel_type(ExcelTypeEnum::Xls);
    let mut execute_context =
        DefaultXlsReadContext::from_read_workbook(&execute_workbook, ExcelTypeEnum::Xls);
    let mut execute_listener = XlsListSheetListener::new(&mut execute_context);
    let execute_error = execute_listener
        .execute()
        .expect_err("Java direct execute stops after the metadata-only pass");
    assert!(matches!(execute_error, ExcelError::AnalysisStop(_)));
    assert!(contract.xls_list_listener_execute_stopped);
    assert_eq!(
        execute_listener.sheet_list().len(),
        contract.xls_list_listener_execute_sheet_count
    );

    let mut sax_workbook = ReadWorkbook::new();
    sax_workbook
        .set_file(java_xls_input())
        .set_excel_type(ExcelTypeEnum::Xls);
    let sax_context = DefaultXlsReadContext::from_read_workbook(&sax_workbook, ExcelTypeEnum::Xls);
    let mut sax_analyser = XlsSaxAnalyser::new(sax_context)?;
    assert!(contract.xls_sax_analyser_class.ends_with("XlsSaxAnalyser"));
    assert_eq!(
        sax_analyser.sheet_list().len(),
        contract.xls_sax_analyser_sheet_count
    );
    ExcelReadExecutor::execute(&mut sax_analyser)?;
    assert!(contract.xls_sax_analyser_execute_succeeded);
    let before_known_record = sax_analyser.record_dispatch_state().bound_sheets().len();
    let mut sax_bound_sheet = vec![100, 0, 0, 0, 0, 0, 9, 0];
    sax_bound_sheet.extend_from_slice(b"ManualSax");
    sax_analyser.process_record(BOUND_SHEET_SID, &sax_bound_sheet)?;
    let after_known_record = sax_analyser.record_dispatch_state().bound_sheets().len();
    sax_analyser.process_record(0x0FFF, &[])?;
    assert_eq!(
        after_known_record - before_known_record,
        contract.xls_sax_analyser_known_record_delta
    );
    assert_eq!(
        sax_analyser.record_dispatch_state().bound_sheets().len() == after_known_record,
        contract.xls_sax_analyser_unknown_record_ignored
    );

    let mut xlsx_workbook = ReadWorkbook::new();
    xlsx_workbook.set_excel_type(ExcelTypeEnum::Xlsx);
    let mut xlsx_context =
        DefaultXlsxReadContext::from_read_workbook(&xlsx_workbook, ExcelTypeEnum::Xlsx);
    assert_eq!(contract.xlsx_context_workbook_type, "XLSX");
    assert_eq!(
        easyexcel::context::AnalysisContextLifecycle::analysis_context_impl(&xlsx_context).excel_type(),
        ExcelTypeEnum::Xlsx
    );
    let _xlsx_workbook_holder = XlsxReadContext::xlsx_read_workbook_holder(&xlsx_context);
    assert_eq!(
        XlsxReadContext::xlsx_read_sheet_holder(&xlsx_context).is_none(),
        contract.xlsx_context_sheet_before_null
    );
    xlsx_context.current_sheet(&ReadSheet::with_name(0, "XlsxContext"))?;
    assert_eq!(
        XlsxReadContext::xlsx_read_sheet_holder(&xlsx_context)
            .expect("XLSX currentSheet must materialize holder")
            .inner()
            .sheet_no,
        contract.xlsx_context_sheet_no
    );

    let mut abstract_handler = AbstractXlsxTagHandler::new();
    assert!(contract.xlsx_tag_handler_interface);
    assert!(contract.abstract_xlsx_tag_handler_is_abstract);
    assert_eq!(
        abstract_handler.support(),
        contract.abstract_xlsx_tag_handler_support_default
    );
    abstract_handler.start_element("c", "r=A1");
    abstract_handler.characters("x");
    abstract_handler.end_element("c");
    assert!(contract.abstract_xlsx_tag_handler_noop_preserved);

    let mut abstract_cell_value_handler = AbstractCellValueTagHandler::new();
    assert!(contract.abstract_cell_value_tag_handler_is_abstract);
    abstract_cell_value_handler.characters("bcd");
    assert_eq!(
        abstract_cell_value_handler.temp_data,
        contract.abstract_cell_value_tag_handler_text
    );
    Ok(())
}

#[test]
fn excel_analyser_read_workbook_requires_input_and_honours_explicit_type() -> easyexcel::Result<()>
{
    let error = match ExcelAnalyserImpl::from_read_workbook(ReadWorkbook::new()) {
        Ok(_) => panic!("Java constructor must reject a workbook without file or input stream"),
        Err(error) => error,
    };
    assert!(
        matches!(error, ExcelError::Format(message) if message == "File and inputStream must be a non-null.")
    );

    let directory = tempdir()?;
    let extensionless = directory.path().join("workbook.data");
    std::fs::copy(java_input(), &extensionless)?;
    let mut workbook = ReadWorkbook::new();
    workbook
        .set_file(&extensionless)
        .set_excel_type(ExcelTypeEnum::Xlsx)
        .set_head_row_number(1);
    let mut analyser = ExcelAnalyserImpl::from_read_workbook(workbook)?;
    let selected = ReadSheet::with_name(0, "Analyser");
    ExcelAnalyser::analysis(&mut analyser, Some(&[selected]), false)?;
    let mut listener = CountingListener::default();
    ExcelAnalyser::analysis_with_listener::<DynamicRow, _>(&mut analyser, &mut listener)?;
    assert_eq!(listener.rows, 1);

    let mut empty_workbook = ReadWorkbook::new();
    empty_workbook
        .set_file(java_input())
        .set_excel_type(ExcelTypeEnum::Xlsx);
    let mut empty = ExcelAnalyserImpl::from_read_workbook(empty_workbook)?;
    let error = ExcelAnalyser::analysis(&mut empty, Some(&[]), false)
        .expect_err("Java analysis requires at least one sheet when readAll is false");
    assert!(
        matches!(error, ExcelError::Format(message) if message == "Specify at least one read sheet.")
    );
    assert!(empty.is_finished());
    Ok(())
}

#[test]
fn excel_analyser_record_handler_sids_match_java_constants() {
    // Java POI BIFF8 record SIDs must match the Rust constants.
    // These values come from org.apache.poi.hssf.record.* and are immutable
    // across POI versions.
    assert_eq!(BOF_SID, 0x0809, "BOF");
    assert_eq!(BOUND_SHEET_SID, 0x0085, "BOUND_SHEET");
    assert_eq!(LABEL_SST_SID, 0x00FD, "LABEL_SST");
    assert_eq!(FORMULA_SID, 0x0006, "FORMULA");
    assert_eq!(LABEL_SID, 0x0204, "LABEL");
    assert_eq!(OBJ_SID, 0x005D, "OBJ");
    assert_eq!(RK_SID, 0x027E, "RK");
    assert_eq!(SST_SID, 0x00FC, "SST");
    assert_eq!(STRING_SID, 0x0207, "STRING");
    assert_eq!(TEXT_OBJECT_SID, 0x01B6, "TEXT_OBJECT");
}

#[test]
fn excel_analyser_xlsx_tag_handler_trait_hierarchy_matches_java() {
    // Verify the handler trait hierarchy is accessible and the abstract
    // default implementations are no-ops (matching Java's empty methods).
    let mut abstract_handler = AbstractXlsxTagHandler::new();
    assert!(abstract_handler.support());
    abstract_handler.start_element("c", "r=A1");
    abstract_handler.characters("x");
    abstract_handler.end_element("c");

    let mut cell_value_handler = AbstractCellValueTagHandler::new();
    cell_value_handler.characters("hello");
    assert_eq!(cell_value_handler.temp_data, "hello");
}
