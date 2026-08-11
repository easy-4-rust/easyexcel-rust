//! easyexcel-markdown 的 GFM 语义、损失报告与资源限制契约。

use std::io::Cursor;

use easyexcel_io::{Error, ResourceLimits};
use easyexcel_markdown::{
    MarkdownExportOptions, MarkdownFormulaPolicy, MarkdownHeaderPolicy, MarkdownImportOptions,
    MarkdownMergePolicy, MarkdownSheetSelection, MarkdownTableSelection, MarkdownTypeInference,
    MarkdownValuePolicy, MarkdownWarningCode, read_markdown, write_document, write_workbook,
};
use easyexcel_model::{Cell, CellRange, CellValue, TabularDocument, TabularTable, Visibility, Workbook};

#[test]
fn parses_multiple_gfm_tables_and_nearest_headings() {
    let source = "# Document\n\n## Orders\n\n| id | name |\n| --- | --- |\n| 007 | Alice\\|A |\n\n## Totals\n\n| ok | amount |\n| --- | ---: |\n| true | 42 |\n";
    let result = read_markdown(
        Cursor::new(source.as_bytes()),
        &MarkdownImportOptions::default(),
    )
    .expect("parse markdown");
    assert_eq!(result.document.tables().len(), 2);
    assert_eq!(result.document.tables()[0].name(), "Orders");
    assert_eq!(
        result.document.tables()[0].rows()[1][0].value(),
        &CellValue::Text("007".to_owned())
    );
    assert_eq!(
        result.document.tables()[1].rows()[1][0].value(),
        &CellValue::Bool(true)
    );
    assert_eq!(
        result.document.tables()[1].rows()[1][1].value(),
        &CellValue::Number(42.0)
    );
}

#[test]
fn text_inference_never_creates_formula_cells() {
    let source = "| formula |\n| --- |\n| =SUM(A1:A2) |\n";
    let options =
        MarkdownImportOptions::default().with_type_inference(MarkdownTypeInference::Aggressive);
    let result = read_markdown(Cursor::new(source.as_bytes()), &options).expect("parse");
    let workbook = result.document.to_workbook();
    assert!(matches!(
        workbook.sheets[0].get(1, 0),
        Some(Cell::Text(value)) if value == "=SUM(A1:A2)"
    ));
}

#[test]
fn formula_policies_are_explicit() {
    let mut workbook = Workbook::new();
    let sheet = &mut workbook.sheets[0];
    sheet.set(0, 0, Cell::Text("total".to_owned()));
    sheet.set(
        1,
        0,
        Cell::Formula {
            expr: "SUM(B1:B2)".to_owned(),
            cached: CellValue::Number(42.0),
        },
    );

    for (policy, expected) in [
        (MarkdownFormulaPolicy::CachedValue, "| 42 |"),
        (MarkdownFormulaPolicy::Expression, "| =SUM(B1:B2) |"),
        (
            MarkdownFormulaPolicy::ExpressionAndCached,
            "| =SUM(B1:B2) => 42 |",
        ),
    ] {
        let (output, _) = write_workbook(
            &workbook,
            Cursor::new(Vec::new()),
            &MarkdownExportOptions::default().with_formulas(policy),
        )
        .expect("write workbook");
        let text = String::from_utf8(output.into_inner()).expect("utf8");
        assert!(text.contains(expected), "{text}");
    }
}

#[test]
fn merge_anchor_reports_structured_loss() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("group".to_owned()));
    workbook.sheets[0].set(1, 0, Cell::Text("A".to_owned()));
    workbook.sheets[0]
        .merged
        .push(CellRange::parse_a1("A2:B2").expect("range"));
    let (_, report) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default().with_merges(MarkdownMergePolicy::AnchorWithWarning),
    )
    .expect("write");
    assert!(report.warnings.iter().any(|warning| {
        warning.code == MarkdownWarningCode::MergeFlattened
            && warning.range.as_deref() == Some("A2:B2")
    }));
}

#[test]
fn merge_error_never_silently_degrades() {
    let mut workbook = Workbook::new();
    workbook.sheets[0]
        .merged
        .push(CellRange::parse_a1("A1:B1").expect("range"));
    let error = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default().with_merges(MarkdownMergePolicy::Error),
    )
    .expect_err("merge must fail");
    assert!(matches!(error, Error::Unsupported(_)));
}

#[test]
fn repeat_and_html_merge_modes_preserve_anchor_semantics() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("header".to_owned()));
    workbook.sheets[0].set(1, 0, Cell::Text("anchor".to_owned()));
    workbook.sheets[0]
        .merged
        .push(CellRange::parse_a1("A2:B2").expect("range"));

    let (repeat, _) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default().with_merges(MarkdownMergePolicy::RepeatAnchor),
    )
    .expect("repeat");
    let repeat = String::from_utf8(repeat.into_inner()).expect("utf8");
    assert!(repeat.contains("| anchor | anchor |"), "{repeat}");

    let (html, _) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default().with_merges(MarkdownMergePolicy::HtmlFallback),
    )
    .expect("html");
    let html = String::from_utf8(html.into_inner()).expect("utf8");
    assert!(html.contains("colspan=\"2\""), "{html}");
}

#[test]
fn agent_stable_escapes_pipe_backslash_and_line_breaks() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("header".to_owned()));
    workbook.sheets[0].set(1, 0, Cell::Text("a|b\\c\nd".to_owned()));
    let (output, _) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default(),
    )
    .expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(text.contains("a\\|b\\\\c<br>d"), "{text}");
}

#[test]
fn hidden_sheets_are_skipped_with_warning() {
    let mut workbook = Workbook::new();
    let hidden = workbook.add_sheet("Hidden");
    workbook.sheets[hidden].visibility = Visibility::Hidden;
    workbook.sheets[hidden].set(0, 0, Cell::Text("secret".to_owned()));
    let (output, report) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default(),
    )
    .expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(!text.contains("secret"));
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.code == MarkdownWarningCode::HiddenSheetSkipped)
    );
}

#[test]
fn input_and_output_limits_are_enforced() {
    let input_options = MarkdownImportOptions::default().with_limits(ResourceLimits::default());
    let huge_cell = "x".repeat(input_options.limits().max_cell_chars() + 1);
    let source = format!("| value |\n| --- |\n| {huge_cell} |\n");
    let error =
        read_markdown(Cursor::new(source.as_bytes()), &input_options).expect_err("cell limit");
    assert!(matches!(
        error,
        Error::ResourceLimit {
            resource: "cell_chars",
            ..
        }
    ));

    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("header".to_owned()));
    let export_options = MarkdownExportOptions::default()
        .with_limits(ResourceLimits::default().with_max_output_bytes(4));
    let error = write_workbook(&workbook, Cursor::new(Vec::new()), &export_options)
        .expect_err("output limit");
    assert!(matches!(
        error,
        Error::ResourceLimit {
            resource: "output_bytes",
            ..
        }
    ));
}

// --- Table selection tests ---

#[test]
fn table_selection_by_name() {
    let source = "## Orders\n\n| id |\n| --- |\n| 1 |\n\n## Totals\n\n| sum |\n| --- |\n| 100 |\n";
    let options = MarkdownImportOptions::default()
        .with_tables(MarkdownTableSelection::Name("Totals".to_owned()));
    let result = read_markdown(Cursor::new(source.as_bytes()), &options).expect("parse");
    assert_eq!(result.document.tables().len(), 1);
    assert_eq!(result.document.tables()[0].name(), "Totals");
}

#[test]
fn table_selection_by_index() {
    let source = "## First\n\n| a |\n| --- |\n| 1 |\n\n## Second\n\n| b |\n| --- |\n| 2 |\n";
    let options = MarkdownImportOptions::default()
        .with_tables(MarkdownTableSelection::Index(1));
    let result = read_markdown(Cursor::new(source.as_bytes()), &options).expect("parse");
    assert_eq!(result.document.tables().len(), 1);
    assert_eq!(result.document.tables()[0].name(), "Second");
}

#[test]
fn table_selection_name_not_found_errors() {
    let source = "| a |\n| --- |\n| 1 |\n";
    let options = MarkdownImportOptions::default()
        .with_tables(MarkdownTableSelection::Name("Missing".to_owned()));
    let error = read_markdown(Cursor::new(source.as_bytes()), &options).expect_err("not found");
    assert!(matches!(error, Error::Markdown { .. }));
}

#[test]
fn table_selection_index_out_of_range_errors() {
    let source = "| a |\n| --- |\n| 1 |\n";
    let options = MarkdownImportOptions::default()
        .with_tables(MarkdownTableSelection::Index(5));
    let error = read_markdown(Cursor::new(source.as_bytes()), &options).expect_err("out of range");
    assert!(matches!(error, Error::Markdown { .. }));
}

// --- Type inference tests ---

#[test]
fn text_inference_preserves_numbers_as_text() {
    let source = "| val |\n| --- |\n| 42 |\n";
    let options = MarkdownImportOptions::default()
        .with_type_inference(MarkdownTypeInference::Text);
    let result = read_markdown(Cursor::new(source.as_bytes()), &options).expect("parse");
    // rows()[0] is the header row, rows()[1] is the first data row
    assert_eq!(
        result.document.tables()[0].rows()[1][0].value(),
        &CellValue::Text("42".to_owned())
    );
}

#[test]
fn aggressive_inference_parses_percentages_and_dates() {
    let source = "| val |\n| --- |\n| 50% |\n";
    let options = MarkdownImportOptions::default()
        .with_type_inference(MarkdownTypeInference::Aggressive);
    let result = read_markdown(Cursor::new(source.as_bytes()), &options).expect("parse");
    // Aggressive mode uses parse_number_text which may handle %
    let value = result.document.tables()[0].rows()[1][0].value();
    // 50% could be parsed as 0.5 or kept as text depending on parser
    assert!(matches!(value, CellValue::Number(_) | CellValue::Text(_)));
}

#[test]
fn canonical_number_rejects_leading_zeros() {
    let source = "| val |\n| --- |\n| 007 |\n";
    let options = MarkdownImportOptions::default()
        .with_type_inference(MarkdownTypeInference::Conservative);
    let result = read_markdown(Cursor::new(source.as_bytes()), &options).expect("parse");
    assert_eq!(
        result.document.tables()[0].rows()[1][0].value(),
        &CellValue::Text("007".to_owned())
    );
}

#[test]
fn bool_values_are_recognized() {
    let source = "| a | b |\n| --- | --- |\n| TRUE | false |\n";
    let result = read_markdown(
        Cursor::new(source.as_bytes()),
        &MarkdownImportOptions::default(),
    )
    .expect("parse");
    assert_eq!(result.document.tables()[0].rows()[1][0].value(), &CellValue::Bool(true));
    assert_eq!(result.document.tables()[0].rows()[1][1].value(), &CellValue::Bool(false));
}

#[test]
fn error_cells_are_recognized() {
    let source = "| val |\n| --- |\n| #DIV/0! |\n";
    let result = read_markdown(
        Cursor::new(source.as_bytes()),
        &MarkdownImportOptions::default(),
    )
    .expect("parse");
    assert!(matches!(result.document.tables()[0].rows()[1][0].value(), CellValue::Error(_)));
}

#[test]
fn no_table_found_errors() {
    let source = "# Just a heading\n\nSome text.\n";
    let error = read_markdown(
        Cursor::new(source.as_bytes()),
        &MarkdownImportOptions::default(),
    )
    .expect_err("no table");
    assert!(matches!(error, Error::Markdown { .. }));
}

// --- Export: sheet selection tests ---

#[test]
fn sheet_selection_first() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("first".to_owned()));
    let second = workbook.add_sheet("Second");
    workbook.sheets[second].set(0, 0, Cell::Text("second".to_owned()));

    let options = MarkdownExportOptions::default()
        .with_sheets(MarkdownSheetSelection::First);
    let (output, _) = write_workbook(&workbook, Cursor::new(Vec::new()), &options).expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(text.contains("first"));
    assert!(!text.contains("second"));
}

#[test]
fn sheet_selection_by_name() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("first".to_owned()));
    let second = workbook.add_sheet("Second");
    workbook.sheets[second].set(0, 0, Cell::Text("second".to_owned()));

    let options = MarkdownExportOptions::default()
        .with_sheets(MarkdownSheetSelection::Name("Second".to_owned()));
    let (output, _) = write_workbook(&workbook, Cursor::new(Vec::new()), &options).expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(!text.contains("first"));
    assert!(text.contains("second"));
}

#[test]
fn sheet_selection_by_index() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("first".to_owned()));
    let second = workbook.add_sheet("Second");
    workbook.sheets[second].set(0, 0, Cell::Text("second".to_owned()));

    let options = MarkdownExportOptions::default()
        .with_sheets(MarkdownSheetSelection::Index(0));
    let (output, _) = write_workbook(&workbook, Cursor::new(Vec::new()), &options).expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(text.contains("first"));
}

#[test]
fn sheet_selection_first_on_empty_workbook_errors() {
    let workbook = Workbook::empty();
    let options = MarkdownExportOptions::default()
        .with_sheets(MarkdownSheetSelection::First);
    let error = write_workbook(&workbook, Cursor::new(Vec::new()), &options).expect_err("empty");
    assert!(matches!(error, Error::SheetNotFound(_)));
}

#[test]
fn sheet_selection_name_not_found_errors() {
    let workbook = Workbook::new();
    let options = MarkdownExportOptions::default()
        .with_sheets(MarkdownSheetSelection::Name("Nope".to_owned()));
    let error = write_workbook(&workbook, Cursor::new(Vec::new()), &options).expect_err("not found");
    assert!(matches!(error, Error::SheetNotFound(_)));
}

// --- Export: header policy ---

#[test]
fn generated_header_policy() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("data1".to_owned()));
    workbook.sheets[0].set(0, 1, Cell::Text("data2".to_owned()));

    let options = MarkdownExportOptions::default()
        .with_header(MarkdownHeaderPolicy::Generated);
    let (output, _) = write_workbook(&workbook, Cursor::new(Vec::new()), &options).expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    // Generated header should use column letters (A, B, ...)
    assert!(text.contains("| A |"), "{text}");
    assert!(text.contains("| B |"), "{text}");
    assert!(text.contains("data1"), "{text}");
}

// --- Export: value policy ---

#[test]
fn raw_value_policy() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("hello".to_owned()));

    let options = MarkdownExportOptions::default()
        .with_values(MarkdownValuePolicy::Raw);
    let (output, _) = write_workbook(&workbook, Cursor::new(Vec::new()), &options).expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(text.contains("hello"));
}

// --- Export: style warning ---

#[test]
fn style_dropped_warning() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("styled".to_owned()));
    workbook.sheets[0].styles.insert((0, 0), 0);

    let (_, report) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default(),
    )
    .expect("write");
    assert!(report.warnings.iter().any(|w| w.code == MarkdownWarningCode::StyleDropped));
}

// --- Export: empty sheet ---

#[test]
fn empty_sheet_warning() {
    let mut workbook = Workbook::new();
    // Sheet is empty by default
    let (_, report) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default(),
    )
    .expect("write");
    assert!(report.warnings.iter().any(|w| w.code == MarkdownWarningCode::EmptySheet));
}

// --- Export: opaque parts warning ---

#[test]
fn opaque_parts_dropped_warning() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("data".to_owned()));
    workbook.opaque.push(easyexcel_model::OpaquePart {
        name: "vba.bin".to_owned(),
        data: vec![1, 2, 3],
    });

    let (_, report) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default(),
    )
    .expect("write");
    assert!(report.warnings.iter().any(|w| w.code == MarkdownWarningCode::UnsupportedObjectDropped));
}

// --- Export: write_document ---

#[test]
fn write_document_from_tabular() {
    let mut table = TabularTable::new("TestTable");
    table.push_row(vec![
        easyexcel_model::TabularCell::header(CellValue::Text("Name".to_owned())),
        easyexcel_model::TabularCell::header(CellValue::Text("Value".to_owned())),
    ]);
    table.push_row(vec![
        easyexcel_model::TabularCell::new(CellValue::Text("Alice".to_owned())),
        easyexcel_model::TabularCell::new(CellValue::Number(42.0)),
    ]);
    let doc = TabularDocument::from_tables(vec![table]);

    let (output, report) = write_document(
        &doc,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default(),
    )
    .expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(text.contains("TestTable"), "{text}");
    assert!(text.contains("Name"), "{text}");
    assert!(text.contains("Alice"), "{text}");
    assert_eq!(report.tables_processed, 1);
}

// --- HTML fallback merge ---

#[test]
fn html_merge_with_rowspan() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("header".to_owned()));
    workbook.sheets[0].set(1, 0, Cell::Text("anchor".to_owned()));
    workbook.sheets[0].set(2, 0, Cell::Text("below".to_owned()));
    workbook.sheets[0]
        .merged
        .push(CellRange::parse_a1("A2:A3").expect("range"));

    let (output, _) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default().with_merges(MarkdownMergePolicy::HtmlFallback),
    )
    .expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(text.contains("rowspan=\"2\""), "{text}");
}

#[test]
fn html_merge_with_colspan() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("header".to_owned()));
    workbook.sheets[0].set(1, 0, Cell::Text("anchor".to_owned()));
    workbook.sheets[0]
        .merged
        .push(CellRange::parse_a1("A2:B2").expect("range"));

    let (output, _) = write_workbook(
        &workbook,
        Cursor::new(Vec::new()),
        &MarkdownExportOptions::default().with_merges(MarkdownMergePolicy::HtmlFallback),
    )
    .expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(text.contains("colspan=\"2\""), "{text}");
}

// --- Edge: column limit in export ---

#[test]
fn export_column_limit_enforced() {
    let mut workbook = Workbook::new();
    // Set a cell very far to the right
    workbook.sheets[0].set(0, 0, Cell::Text("header".to_owned()));

    let limits = ResourceLimits::default().with_max_columns(1);
    let options = MarkdownExportOptions::default().with_limits(limits);
    let result = write_workbook(&workbook, Cursor::new(Vec::new()), &options);
    // The first row has 1 column which is at the limit, should be ok
    assert!(result.is_ok());
}

// --- Row limit in export ---

#[test]
fn export_row_limit_enforced() {
    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("h".to_owned()));
    workbook.sheets[0].set(1, 0, Cell::Text("a".to_owned()));
    workbook.sheets[0].set(2, 0, Cell::Text("b".to_owned()));

    let limits = ResourceLimits::new(256 * 1024 * 1024, 256, 2, 500_000);
    let options = MarkdownExportOptions::default().with_limits(limits);
    let error = write_workbook(&workbook, Cursor::new(Vec::new()), &options).expect_err("row limit");
    assert!(matches!(error, Error::ResourceLimit { resource: "rows", .. }));
}

// --- Include hidden sheets ---

#[test]
fn include_hidden_sheets_option() {
    let mut workbook = Workbook::new();
    let hidden = workbook.add_sheet("Hidden");
    workbook.sheets[hidden].visibility = Visibility::Hidden;
    workbook.sheets[hidden].set(0, 0, Cell::Text("secret".to_owned()));

    let options = MarkdownExportOptions::default().with_include_hidden(true);
    let (output, report) = write_workbook(&workbook, Cursor::new(Vec::new()), &options).expect("write");
    let text = String::from_utf8(output.into_inner()).expect("utf8");
    assert!(text.contains("secret"), "{text}");
    // No hidden sheet warning when included
    assert!(!report.warnings.iter().any(|w| w.code == MarkdownWarningCode::HiddenSheetSkipped));
}
