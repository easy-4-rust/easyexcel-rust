//! easyexcel-markdown 的 GFM 语义、损失报告与资源限制契约。

use std::io::Cursor;

use easyexcel_io::{Error, ResourceLimits};
use easyexcel_markdown::{
    MarkdownExportOptions, MarkdownFormulaPolicy, MarkdownImportOptions, MarkdownMergePolicy,
    MarkdownTypeInference, MarkdownWarningCode, read_markdown, write_workbook,
};
use easyexcel_model::{Cell, CellRange, CellValue, Visibility, Workbook};

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
