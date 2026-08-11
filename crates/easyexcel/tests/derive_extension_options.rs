//! 测试 derive 宏的扩展属性：conditional、data_validation、image、comment、hyperlink、formula、filter。

use easyexcel::ExcelRow;

// --- conditional ---

#[derive(ExcelRow)]
struct ConditionalRow {
    #[excel(
        conditional(
            condition = ">100",
            font_color = "FF0000",
            background_color = "FFFF00"
        )
    )]
    amount: f64,
}

#[test]
fn conditional_attribute_parses() {
    let schema = ConditionalRow::schema();
    assert_eq!(schema.len(), 1);
    let col = &schema[0];
    let cond = col.conditional_format.as_ref().expect("conditional should be set");
    assert_eq!(cond.0, ">100");
    assert_eq!(cond.1, "FF0000");
    assert_eq!(cond.2, "FFFF00");
}

// --- data_validation ---

#[derive(ExcelRow)]
struct DataValidationRow {
    #[excel(
        data_validation(
            type = "list",
            operator = "between",
            formula1 = "\"A,B,C\"",
            formula2 = ""
        )
    )]
    category: String,
}

#[test]
fn data_validation_attribute_parses() {
    let schema = DataValidationRow::schema();
    assert_eq!(schema.len(), 1);
    let col = &schema[0];
    let dv = col.data_validation.as_ref().expect("data_validation should be set");
    assert_eq!(dv.data_type, "list");
    assert_eq!(dv.operator, "between");
    assert_eq!(dv.formula1, "\"A,B,C\"");
    assert_eq!(dv.formula2, "");
}

// --- image ---

#[derive(ExcelRow)]
struct ImageRow {
    #[excel(image = "A1:B2")]
    photo: Vec<u8>,
}

#[test]
fn image_attribute_parses() {
    let schema = ImageRow::schema();
    assert_eq!(schema.len(), 1);
    assert_eq!(schema[0].image_path.as_deref(), Some("A1:B2"));
}

// --- comment ---

#[derive(ExcelRow)]
struct CommentRow {
    #[excel(comment = "This is a note")]
    value: String,
}

#[test]
fn comment_attribute_parses() {
    let schema = CommentRow::schema();
    assert_eq!(schema.len(), 1);
    assert_eq!(schema[0].comment.as_deref(), Some("This is a note"));
}

// --- hyperlink ---

#[derive(ExcelRow)]
struct HyperlinkRow {
    #[excel(hyperlink = "https://example.com")]
    link: String,
}

#[test]
fn hyperlink_attribute_parses() {
    let schema = HyperlinkRow::schema();
    assert_eq!(schema.len(), 1);
    assert_eq!(schema[0].hyperlink.as_deref(), Some("https://example.com"));
}

// --- formula ---

#[derive(ExcelRow)]
struct FormulaRow {
    #[excel(formula = "SUM(A1:A10)")]
    total: f64,
}

#[test]
fn formula_attribute_parses() {
    let schema = FormulaRow::schema();
    assert_eq!(schema.len(), 1);
    assert_eq!(schema[0].formula.as_deref(), Some("SUM(A1:A10)"));
}

// --- filter ---

#[derive(ExcelRow)]
struct FilterRow {
    #[excel(filter)]
    name: String,
    #[excel(filter)]
    age: i32,
}

#[test]
fn filter_attribute_parses() {
    let schema = FilterRow::schema();
    assert_eq!(schema.len(), 2);
    assert!(schema[0].auto_filter, "name column should have filter");
    assert!(schema[1].auto_filter, "age column should have filter");
}

// --- multiple extension options on one field ---

#[derive(ExcelRow)]
struct MultiExtensionRow {
    #[excel(
        comment = "Key metric",
        formula = "A1*B1",
        conditional(condition = ">0", font_color = "00FF00", background_color = "")
    )]
    metric: f64,
}

#[test]
fn multiple_extensions_on_one_field() {
    let schema = MultiExtensionRow::schema();
    assert_eq!(schema.len(), 1);
    let col = &schema[0];
    assert_eq!(col.comment.as_deref(), Some("Key metric"));
    assert_eq!(col.formula.as_deref(), Some("A1*B1"));
    assert!(col.conditional_format.is_some());
}
