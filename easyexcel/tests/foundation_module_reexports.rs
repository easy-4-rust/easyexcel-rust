//! `EasyExcel` 基础组件门面的外部编译兼容测试。

use easyexcel::csv::{CsvReadOptions, CsvWriteOptions};
use easyexcel::io::{Format, ResourceLimits};
use easyexcel::model::{Cell, Workbook};

#[test]
fn foundation_types_are_available_from_the_easyexcel_facade() {
    let read_options = CsvReadOptions::default();
    let write_options = CsvWriteOptions::default();
    let limits = ResourceLimits::default();
    let format = Format::Csv;

    let mut workbook = Workbook::new();
    workbook.sheets[0].set(0, 0, Cell::Text("value".to_owned()));

    assert_eq!(workbook.display_cell(0, 0, 0), "value");

    // 门面只做类型重导出，必须与基础 crates 保持同一类型身份。
    accept_csv_read_options(read_options);
    accept_csv_write_options(write_options);
    accept_resource_limits(limits);
    accept_format(format);
    accept_workbook(workbook);
}

fn accept_csv_read_options(_: easyexcel_csv::CsvReadOptions) {}

fn accept_csv_write_options(_: easyexcel_csv::CsvWriteOptions) {}

fn accept_resource_limits(_: easyexcel_io::ResourceLimits) {}

fn accept_format(_: easyexcel_io::Format) {}

fn accept_workbook(_: easyexcel_model::Workbook) {}
