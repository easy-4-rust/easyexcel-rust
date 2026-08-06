//! `EasyExcel` 门面下 XLS、XLSX、CSV 与 Markdown 的端到端契约。

use std::fs;

use easyexcel::markdown::{
    MarkdownConversionMode, MarkdownExportOptions, MarkdownImportOptions, export_path, import_path,
};
use easyexcel::model::{Cell, Workbook};
use easyexcel::{EasyExcel, ExcelError};

fn fixture_workbook() -> Workbook {
    let mut workbook = Workbook::new();
    "Data".clone_into(&mut workbook.sheets[0].name);
    workbook.sheets[0].set(0, 0, Cell::Text("id".to_owned()));
    workbook.sheets[0].set(0, 1, Cell::Text("name".to_owned()));
    workbook.sheets[0].set(1, 0, Cell::Text("007".to_owned()));
    workbook.sheets[0].set(1, 1, Cell::Text("Alice".to_owned()));
    workbook
}

#[test]
fn xls_xlsx_and_csv_export_to_markdown() {
    let directory = tempfile::tempdir().expect("directory");
    let workbook = fixture_workbook();
    let xlsx = directory.path().join("fixture.xlsx");
    let xls = directory.path().join("fixture.xls");
    let csv = directory.path().join("fixture.csv");
    easyexcel::xlsx::write_path(&workbook, &xlsx).expect("xlsx");
    easyexcel::xls::write_path(&workbook, &xls).expect("xls");
    easyexcel::csv::write_csv(
        &workbook,
        0,
        fs::File::create(&csv).expect("csv file"),
        &easyexcel::csv::CsvWriteOptions::default(),
    )
    .expect("csv");

    for input in [&xlsx, &xls, &csv] {
        let output = input.with_extension("md");
        let report = export_path(input, &output, &MarkdownExportOptions::default())
            .expect("export markdown");
        assert!(report.rows_processed >= 2);
        let markdown = fs::read_to_string(output).expect("markdown");
        assert!(markdown.contains("| id | name |"), "{markdown}");
        assert!(markdown.contains("| 007 | Alice |"), "{markdown}");
    }
}

#[test]
fn markdown_imports_to_xls_xlsx_and_csv_and_reopens() {
    let directory = tempfile::tempdir().expect("directory");
    let markdown = directory.path().join("fixture.md");
    fs::write(
        &markdown,
        "## Data\n\n| id | name |\n| --- | --- |\n| 007 | Alice |\n",
    )
    .expect("fixture");

    for extension in ["xlsx", "xls", "csv"] {
        let output = directory.path().join(format!("output.{extension}"));
        import_path(&markdown, &output, &MarkdownImportOptions::default()).expect("import");
        let workbook = match extension {
            "xlsx" => easyexcel::xlsx::read_path(&output).expect("read xlsx"),
            "xls" => easyexcel::xls::read_path(&output).expect("read xls"),
            "csv" => easyexcel::csv::read_csv(
                fs::File::open(&output).expect("open csv"),
                &easyexcel::csv::CsvReadOptions::default(),
            )
            .expect("read csv"),
            _ => unreachable!(),
        };
        assert_eq!(workbook.sheets[0].value(1, 0).to_display_string(), "007");
        assert_eq!(workbook.sheets[0].value(1, 1).to_display_string(), "Alice");
    }
}

#[test]
fn event_and_workbook_modes_generate_the_same_plain_xlsx_markdown() {
    let directory = tempfile::tempdir().expect("directory");
    let xlsx = directory.path().join("fixture.xlsx");
    easyexcel::xlsx::write_path(&fixture_workbook(), &xlsx).expect("xlsx");
    let event = directory.path().join("event.md");
    let workbook = directory.path().join("workbook.md");
    export_path(
        &xlsx,
        &event,
        &MarkdownExportOptions::default().with_mode(MarkdownConversionMode::Event),
    )
    .expect("event");
    export_path(
        &xlsx,
        &workbook,
        &MarkdownExportOptions::default().with_mode(MarkdownConversionMode::Workbook),
    )
    .expect("workbook");
    assert_eq!(
        fs::read_to_string(event).expect("event text"),
        fs::read_to_string(workbook).expect("workbook text")
    );
}

#[test]
fn xls_event_is_explicitly_unsupported() {
    let directory = tempfile::tempdir().expect("directory");
    let xls = directory.path().join("fixture.xls");
    easyexcel::xls::write_path(&fixture_workbook(), &xls).expect("xls");
    let error = export_path(
        &xls,
        directory.path().join("fixture.md"),
        &MarkdownExportOptions::default().with_mode(MarkdownConversionMode::Event),
    )
    .expect_err("xls event must fail");
    assert!(matches!(error, ExcelError::Unsupported(_)));
}

#[test]
fn multiple_markdown_tables_require_selection_for_csv() {
    let directory = tempfile::tempdir().expect("directory");
    let markdown = directory.path().join("multiple.md");
    fs::write(
        &markdown,
        "## A\n\n| a |\n| --- |\n| 1 |\n\n## B\n\n| b |\n| --- |\n| 2 |\n",
    )
    .expect("fixture");
    let error = import_path(
        &markdown,
        directory.path().join("multiple.csv"),
        &MarkdownImportOptions::default(),
    )
    .expect_err("selection required");
    assert!(matches!(error, ExcelError::Unsupported(_)));
}

#[test]
fn public_easyexcel_builders_are_the_only_required_entrypoint() {
    let directory = tempfile::tempdir().expect("directory");
    let markdown = directory.path().join("fixture.md");
    let xlsx = directory.path().join("fixture.xlsx");
    fs::write(&markdown, "| id |\n| --- |\n| 007 |\n").expect("fixture");
    EasyExcel::import_markdown(&markdown, &xlsx)
        .conservative_types()
        .apply_header_style(true)
        .do_import()
        .expect("builder import");
    let exported = directory.path().join("roundtrip.md");
    EasyExcel::export_markdown(&xlsx, &exported)
        .mode(MarkdownConversionMode::Auto)
        .do_export()
        .expect("builder export");
    assert!(
        fs::read_to_string(exported)
            .expect("output")
            .contains("007")
    );
}
