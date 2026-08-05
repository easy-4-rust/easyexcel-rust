//! `easyexcel-csv` 公共路径兼容性测试。

use easyexcel_csv::csv::{CsvCharset as NestedCharset, CsvReadOptions as NestedReadOptions};
use easyexcel_csv::{CsvCharset, CsvReadOptions, CsvWriteOptions};

#[test]
fn root_and_csv_paths_resolve_to_the_same_types() {
    let root_charset: CsvCharset = NestedCharset::utf8();
    assert_eq!(root_charset, CsvCharset::utf8());

    let root_options: CsvReadOptions = NestedReadOptions::default();
    assert_eq!(root_options.sheet_name, "Sheet1");

    let _ = CsvWriteOptions::default();
}
