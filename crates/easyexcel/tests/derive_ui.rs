//! `ExcelRow` 派生宏的编译期契约。

#[test]
fn excel_row_ui_contracts() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/derive_ui/pass/*.rs");
    cases.compile_fail("tests/derive_ui/fail/*.rs");
}
