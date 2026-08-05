//! `ExcelRow` 派生宏的编译期契约。

#[test]
fn excel_row_ui_contracts() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
    cases.compile_fail("tests/ui/fail/*.rs");
}
