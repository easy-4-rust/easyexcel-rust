//! `easyexcel-model` 公共模块路径兼容性测试。

use easyexcel_model::model::{Cell as NestedCell, CellAddress as NestedAddress};
use easyexcel_model::{Cell, CellAddress, Workbook, addr, dates, numfmt, styles, value};

#[test]
fn root_and_model_paths_resolve_to_the_same_types() {
    let root_cell: Cell = NestedCell::Empty;
    assert_eq!(root_cell, Cell::Empty);

    let root_address: CellAddress = NestedAddress::new(1, 2);
    assert_eq!(root_address, addr::CellAddress::new(1, 2));

    let workbook = Workbook::new();
    assert_eq!(workbook.sheets.len(), 1);

    let _ = dates::DateSystem::Date1900;
    let _ = numfmt::builtin_format_code(0);
    let _ = styles::CellStyle::default();
    let _ = value::CellValue::Empty;
}
