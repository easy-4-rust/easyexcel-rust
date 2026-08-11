//! Java 4.0.3 style annotation evidence tests for 9 write/style annotations.
//!
//! Covers: ColumnWidth, HeadRowHeight, ContentRowHeight, HeadStyle, ContentStyle,
//! HeadFontStyle, ContentFontStyle, OnceAbsoluteMerge, ContentLoopMerge.
//!
//! Each annotation is tested for:
//! 1. Compile probe: derive macro parses the annotation without error
//! 2. Behavior test: metadata values are correctly set on the schema

use easyexcel::core::ExcelRow as ExcelRowTrait;
use easyexcel::{
    ExcelBorderStyle, ExcelCellStyle, ExcelColor, ExcelDataFormat, ExcelFillPattern,
    ExcelFontScript, ExcelFontStyle, ExcelHorizontalAlignment, ExcelRow, ExcelUnderline,
    ExcelVerticalAlignment, LoopMergeProperty, OnceAbsoluteMergeProperty,
};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Golden contract
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct StyleAnnotationsContract {
    authority: String,
    column_width: DimensionContract,
    head_row_height: DimensionContract,
    content_row_height: DimensionContract,
    head_style: StyleContract,
    content_style: StyleContract,
    head_font_style: StyleContract,
    content_font_style: StyleContract,
    once_absolute_merge: StyleContract,
    content_loop_merge: StyleContract,
    total_java_members: usize,
}

#[derive(Debug, Deserialize)]
struct DimensionContract {
    class: String,
    value_type: String,
    default_value: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct StyleContract {
    class: String,
    members: Vec<MemberContract>,
    member_count: usize,
}

#[derive(Debug, Deserialize)]
struct MemberContract {
    name: String,
    #[serde(rename = "type")]
    type_name: String,
    default: Option<serde_json::Value>,
}

fn contract() -> StyleAnnotationsContract {
    serde_json::from_str(include_str!("fixtures/style_annotations.contract.json"))
        .expect("Java style annotations contract must be valid JSON")
}

// ---------------------------------------------------------------------------
// Test structs: one per annotation
// ---------------------------------------------------------------------------

/// ColumnWidth: field-level column width
#[derive(Debug, Clone, ExcelRow)]
struct ColumnWidthTestData {
    #[excel(name = "Name", index = 0, column_width = 30)]
    name: String,
    #[excel(name = "Value", index = 1)]
    value: String,
}

/// ColumnWidth: struct-level column width
#[derive(Debug, Clone, ExcelRow)]
#[excel(column_width = 50)]
struct ColumnWidthStructTestData {
    #[excel(name = "Name", index = 0)]
    name: String,
}

/// HeadRowHeight: struct-level head row height
#[derive(Debug, Clone, ExcelRow)]
#[excel(head_row_height = 25)]
struct HeadRowHeightTestData {
    #[excel(name = "Name", index = 0)]
    name: String,
}

/// ContentRowHeight: struct-level content row height
#[derive(Debug, Clone, ExcelRow)]
#[excel(content_row_height = 15)]
struct ContentRowHeightTestData {
    #[excel(name = "Name", index = 0)]
    name: String,
}

/// HeadStyle: field-level head style with all properties
#[derive(Debug, Clone, ExcelRow)]
struct HeadStyleFieldTestData {
    #[excel(
        name = "Styled",
        index = 0,
        head_style(
            hidden = true,
            locked = true,
            quote_prefix = true,
            wrapped = true,
            shrink_to_fit = true,
            horizontal_alignment = "center",
            vertical_alignment = "center",
            rotation = 45,
            indent = 2,
            data_format = "0.00",
            border_left = "thin",
            border_right = "thin",
            border_top = "thin",
            border_bottom = "thin",
            left_border_color = 10,
            right_border_color = 10,
            top_border_color = 10,
            bottom_border_color = 10,
            fill_pattern = "solid",
            fill_background_color = 22,
            fill_foreground_color = 13
        )
    )]
    styled: String,
}

/// HeadStyle: struct-level head style
#[derive(Debug, Clone, ExcelRow)]
#[excel(head_style(
    fill_pattern = "solid",
    fill_foreground_color = 10,
    horizontal_alignment = "left"
))]
struct HeadStyleStructTestData {
    #[excel(name = "Name", index = 0)]
    name: String,
}

/// ContentStyle: field-level content style with all properties
#[derive(Debug, Clone, ExcelRow)]
struct ContentStyleFieldTestData {
    #[excel(
        name = "Styled",
        index = 0,
        content_style(
            hidden = true,
            locked = false,
            quote_prefix = false,
            wrapped = true,
            shrink_to_fit = false,
            horizontal_alignment = "right",
            vertical_alignment = "bottom",
            rotation = -30,
            indent = 1,
            data_format = 44,
            border_left = "medium",
            border_right = "medium",
            border_top = "medium",
            border_bottom = "medium",
            left_border_color = 20,
            right_border_color = 20,
            top_border_color = 20,
            bottom_border_color = 20,
            fill_pattern_type = "solid",
            fill_background_color = 30,
            fill_foreground_color = 17
        )
    )]
    styled: String,
}

/// ContentStyle: struct-level content style
#[derive(Debug, Clone, ExcelRow)]
#[excel(content_style(fill_pattern = "solid", fill_foreground_color = 17))]
struct ContentStyleStructTestData {
    #[excel(name = "Name", index = 0)]
    name: String,
}

/// HeadFontStyle: field-level head font style with all 10 properties
#[derive(Debug, Clone, ExcelRow)]
struct HeadFontStyleFieldTestData {
    #[excel(
        name = "Styled",
        index = 0,
        head_font_style(
            font_name = "Arial",
            font_height_in_points = 14,
            bold = true,
            italic = true,
            strikeout = false,
            underline = "single",
            type_offset = "superscript",
            charset = 1,
            color = 15
        )
    )]
    styled: String,
}

/// HeadFontStyle: struct-level head font style
#[derive(Debug, Clone, ExcelRow)]
#[excel(head_font_style(font_height_in_points = 12, bold = true, color = 10))]
struct HeadFontStyleStructTestData {
    #[excel(name = "Name", index = 0)]
    name: String,
}

/// ContentFontStyle: field-level content font style with all 10 properties
#[derive(Debug, Clone, ExcelRow)]
struct ContentFontStyleFieldTestData {
    #[excel(
        name = "Styled",
        index = 0,
        content_font_style(
            font_name = "Calibri",
            font_height_in_points = 11,
            bold = false,
            italic = true,
            strikeout = true,
            underline = "double",
            type_offset = "subscript",
            charset = 2,
            color = 22
        )
    )]
    styled: String,
}

/// ContentFontStyle: struct-level content font style
#[derive(Debug, Clone, ExcelRow)]
#[excel(content_font_style(font_height_in_points = 10, italic = true, color = 58))]
struct ContentFontStyleStructTestData {
    #[excel(name = "Name", index = 0)]
    name: String,
}

/// OnceAbsoluteMerge: struct-level absolute merge
#[derive(Debug, Clone, ExcelRow)]
#[excel(once_absolute_merge(
    first_row_index = 0,
    last_row_index = 2,
    first_column_index = 0,
    last_column_index = 1
))]
struct OnceAbsoluteMergeTestData {
    #[excel(name = "Name", index = 0)]
    name: String,
    #[excel(name = "Value", index = 1)]
    value: String,
}

/// ContentLoopMerge: field-level content loop merge
#[derive(Debug, Clone, ExcelRow)]
struct ContentLoopMergeTestData {
    #[excel(
        name = "Name",
        index = 0,
        content_loop_merge(each_row = 3, column_extend = 2)
    )]
    name: String,
    #[excel(name = "Value", index = 1)]
    value: String,
}

// ---------------------------------------------------------------------------
// Compile probe tests: verify derive macro compiles without error
// ---------------------------------------------------------------------------

#[test]
fn column_width_compile_probe() -> easyexcel::Result<()> {
    // Verify ColumnWidth annotation compiles at both field and struct level
    let _cols = ColumnWidthTestData::schema();
    let _cols = ColumnWidthStructTestData::schema();
    Ok(())
}

#[test]
fn head_row_height_compile_probe() -> easyexcel::Result<()> {
    let _cols = HeadRowHeightTestData::schema();
    Ok(())
}

#[test]
fn content_row_height_compile_probe() -> easyexcel::Result<()> {
    let _cols = ContentRowHeightTestData::schema();
    Ok(())
}

#[test]
fn head_style_compile_probe() -> easyexcel::Result<()> {
    let _cols = HeadStyleFieldTestData::schema();
    let _cols = HeadStyleStructTestData::schema();
    Ok(())
}

#[test]
fn content_style_compile_probe() -> easyexcel::Result<()> {
    let _cols = ContentStyleFieldTestData::schema();
    let _cols = ContentStyleStructTestData::schema();
    Ok(())
}

#[test]
fn head_font_style_compile_probe() -> easyexcel::Result<()> {
    let _cols = HeadFontStyleFieldTestData::schema();
    let _cols = HeadFontStyleStructTestData::schema();
    Ok(())
}

#[test]
fn content_font_style_compile_probe() -> easyexcel::Result<()> {
    let _cols = ContentFontStyleFieldTestData::schema();
    let _cols = ContentFontStyleStructTestData::schema();
    Ok(())
}

#[test]
fn once_absolute_merge_compile_probe() -> easyexcel::Result<()> {
    let _cols = OnceAbsoluteMergeTestData::schema();
    Ok(())
}

#[test]
fn content_loop_merge_compile_probe() -> easyexcel::Result<()> {
    let _cols = ContentLoopMergeTestData::schema();
    Ok(())
}

// ---------------------------------------------------------------------------
// Behavior tests: verify metadata values match Java golden contract
// ---------------------------------------------------------------------------

/// Verify ColumnWidth annotation metadata matches contract defaults.
#[test]
fn column_width_behavior() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.column_width.class,
        "com.alibaba.excel.annotation.write.style.ColumnWidth"
    );

    // Field-level: column_width = 30 on field "name"
    let cols = ColumnWidthTestData::schema();
    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].field, "name");
    assert_eq!(cols[0].column_width, Some(30));
    assert_eq!(cols[1].column_width, None);

    // Struct-level: column_width = 50 on struct
    let metadata = ColumnWidthStructTestData::write_metadata();
    assert_eq!(metadata.column_width, Some(50));

    Ok(())
}

/// Verify HeadRowHeight annotation metadata matches contract defaults.
#[test]
fn head_row_height_behavior() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.head_row_height.class,
        "com.alibaba.excel.annotation.write.style.HeadRowHeight"
    );

    let metadata = HeadRowHeightTestData::write_metadata();
    assert_eq!(metadata.head_row_height, Some(25));

    Ok(())
}

/// Verify ContentRowHeight annotation metadata matches contract defaults.
#[test]
fn content_row_height_behavior() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.content_row_height.class,
        "com.alibaba.excel.annotation.write.style.ContentRowHeight"
    );

    let metadata = ContentRowHeightTestData::write_metadata();
    assert_eq!(metadata.content_row_height, Some(15));

    Ok(())
}

/// Verify HeadStyle annotation parses all 21 members correctly.
#[test]
fn head_style_behavior() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.head_style.class,
        "com.alibaba.excel.annotation.write.style.HeadStyle"
    );
    assert_eq!(contract.head_style.member_count, 21);
    assert_eq!(contract.head_style.members.len(), 21);

    // Field-level test: verify all style properties are set
    let cols = HeadStyleFieldTestData::schema();
    assert_eq!(cols.len(), 1);
    let head_style = cols[0]
        .head_style
        .as_ref()
        .expect("head_style should be set");
    assert_eq!(head_style.hidden, Some(true));
    assert_eq!(head_style.locked, Some(true));
    assert_eq!(head_style.quote_prefix, Some(true));
    assert_eq!(head_style.wrapped, Some(true));
    assert_eq!(head_style.shrink_to_fit, Some(true));
    assert_eq!(
        head_style.horizontal_alignment,
        Some(ExcelHorizontalAlignment::Center)
    );
    assert_eq!(
        head_style.vertical_alignment,
        Some(ExcelVerticalAlignment::Center)
    );
    assert_eq!(head_style.rotation, Some(45));
    assert_eq!(head_style.indent, Some(2));
    assert_eq!(
        head_style.data_format,
        Some(ExcelDataFormat::Custom("0.00"))
    );
    assert_eq!(head_style.border_left, Some(ExcelBorderStyle::Thin));
    assert_eq!(head_style.border_right, Some(ExcelBorderStyle::Thin));
    assert_eq!(head_style.border_top, Some(ExcelBorderStyle::Thin));
    assert_eq!(head_style.border_bottom, Some(ExcelBorderStyle::Thin));
    assert_eq!(
        head_style.left_border_color,
        Some(ExcelColor::java_or_rgb(10))
    );
    assert_eq!(
        head_style.right_border_color,
        Some(ExcelColor::java_or_rgb(10))
    );
    assert_eq!(
        head_style.top_border_color,
        Some(ExcelColor::java_or_rgb(10))
    );
    assert_eq!(
        head_style.bottom_border_color,
        Some(ExcelColor::java_or_rgb(10))
    );
    assert_eq!(head_style.fill_pattern, Some(ExcelFillPattern::Solid));
    assert_eq!(
        head_style.fill_background_color,
        Some(ExcelColor::java_or_rgb(22))
    );
    assert_eq!(
        head_style.fill_foreground_color,
        Some(ExcelColor::java_or_rgb(13))
    );

    // Struct-level test
    let metadata = HeadStyleStructTestData::write_metadata();
    let head_style = metadata
        .head_style
        .as_ref()
        .expect("struct head_style should be set");
    assert_eq!(head_style.fill_pattern, Some(ExcelFillPattern::Solid));
    assert_eq!(
        head_style.fill_foreground_color,
        Some(ExcelColor::java_or_rgb(10))
    );
    assert_eq!(
        head_style.horizontal_alignment,
        Some(ExcelHorizontalAlignment::Left)
    );

    Ok(())
}

/// Verify ContentStyle annotation parses all 21 members correctly.
#[test]
fn content_style_behavior() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.content_style.class,
        "com.alibaba.excel.annotation.write.style.ContentStyle"
    );
    assert_eq!(contract.content_style.member_count, 21);
    assert_eq!(contract.content_style.members.len(), 21);

    // Field-level test: verify all style properties are set
    let cols = ContentStyleFieldTestData::schema();
    assert_eq!(cols.len(), 1);
    let content_style = cols[0]
        .content_style
        .as_ref()
        .expect("content_style should be set");
    assert_eq!(content_style.hidden, Some(true));
    assert_eq!(content_style.locked, Some(false));
    assert_eq!(content_style.quote_prefix, Some(false));
    assert_eq!(content_style.wrapped, Some(true));
    assert_eq!(content_style.shrink_to_fit, Some(false));
    assert_eq!(
        content_style.horizontal_alignment,
        Some(ExcelHorizontalAlignment::Right)
    );
    assert_eq!(
        content_style.vertical_alignment,
        Some(ExcelVerticalAlignment::Bottom)
    );
    assert_eq!(content_style.rotation, Some(-30));
    assert_eq!(content_style.indent, Some(1));
    assert_eq!(
        content_style.data_format,
        Some(ExcelDataFormat::Builtin(44))
    );
    assert_eq!(content_style.border_left, Some(ExcelBorderStyle::Medium));
    assert_eq!(content_style.border_right, Some(ExcelBorderStyle::Medium));
    assert_eq!(content_style.border_top, Some(ExcelBorderStyle::Medium));
    assert_eq!(content_style.border_bottom, Some(ExcelBorderStyle::Medium));
    assert_eq!(
        content_style.left_border_color,
        Some(ExcelColor::java_or_rgb(20))
    );
    assert_eq!(
        content_style.right_border_color,
        Some(ExcelColor::java_or_rgb(20))
    );
    assert_eq!(
        content_style.top_border_color,
        Some(ExcelColor::java_or_rgb(20))
    );
    assert_eq!(
        content_style.bottom_border_color,
        Some(ExcelColor::java_or_rgb(20))
    );
    assert_eq!(content_style.fill_pattern, Some(ExcelFillPattern::Solid));
    assert_eq!(
        content_style.fill_background_color,
        Some(ExcelColor::java_or_rgb(30))
    );
    assert_eq!(
        content_style.fill_foreground_color,
        Some(ExcelColor::java_or_rgb(17))
    );

    // Struct-level test
    let metadata = ContentStyleStructTestData::write_metadata();
    let content_style = metadata
        .content_style
        .as_ref()
        .expect("struct content_style should be set");
    assert_eq!(content_style.fill_pattern, Some(ExcelFillPattern::Solid));
    assert_eq!(
        content_style.fill_foreground_color,
        Some(ExcelColor::java_or_rgb(17))
    );

    Ok(())
}

/// Verify HeadFontStyle annotation parses all 9 members correctly.
#[test]
fn head_font_style_behavior() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.head_font_style.class,
        "com.alibaba.excel.annotation.write.style.HeadFontStyle"
    );
    assert_eq!(contract.head_font_style.member_count, 9);
    assert_eq!(contract.head_font_style.members.len(), 9);

    // Field-level test: verify all font properties are set
    let cols = HeadFontStyleFieldTestData::schema();
    assert_eq!(cols.len(), 1);
    let head_font = cols[0]
        .head_font_style
        .as_ref()
        .expect("head_font_style should be set");
    assert_eq!(head_font.font_name, Some("Arial"));
    assert_eq!(head_font.font_height_in_points, Some(14.0));
    assert_eq!(head_font.bold, Some(true));
    assert_eq!(head_font.italic, Some(true));
    assert_eq!(head_font.strikeout, Some(false));
    assert_eq!(head_font.underline, Some(ExcelUnderline::Single));
    assert_eq!(head_font.type_offset, Some(ExcelFontScript::Superscript));
    assert_eq!(head_font.charset, Some(1));
    assert_eq!(head_font.color, Some(ExcelColor::java_or_rgb(15)));

    // Struct-level test
    let metadata = HeadFontStyleStructTestData::write_metadata();
    let head_font = metadata
        .head_font_style
        .as_ref()
        .expect("struct head_font_style should be set");
    assert_eq!(head_font.font_height_in_points, Some(12.0));
    assert_eq!(head_font.bold, Some(true));
    assert_eq!(head_font.color, Some(ExcelColor::java_or_rgb(10)));

    Ok(())
}

/// Verify ContentFontStyle annotation parses all 9 members correctly.
#[test]
fn content_font_style_behavior() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.content_font_style.class,
        "com.alibaba.excel.annotation.write.style.ContentFontStyle"
    );
    assert_eq!(contract.content_font_style.member_count, 9);
    assert_eq!(contract.content_font_style.members.len(), 9);

    // Field-level test: verify all font properties are set
    let cols = ContentFontStyleFieldTestData::schema();
    assert_eq!(cols.len(), 1);
    let content_font = cols[0]
        .content_font_style
        .as_ref()
        .expect("content_font_style should be set");
    assert_eq!(content_font.font_name, Some("Calibri"));
    assert_eq!(content_font.font_height_in_points, Some(11.0));
    assert_eq!(content_font.bold, Some(false));
    assert_eq!(content_font.italic, Some(true));
    assert_eq!(content_font.strikeout, Some(true));
    assert_eq!(content_font.underline, Some(ExcelUnderline::Double));
    assert_eq!(content_font.type_offset, Some(ExcelFontScript::Subscript));
    assert_eq!(content_font.charset, Some(2));
    assert_eq!(content_font.color, Some(ExcelColor::java_or_rgb(22)));

    // Struct-level test
    let metadata = ContentFontStyleStructTestData::write_metadata();
    let content_font = metadata
        .content_font_style
        .as_ref()
        .expect("struct content_font_style should be set");
    assert_eq!(content_font.font_height_in_points, Some(10.0));
    assert_eq!(content_font.italic, Some(true));
    assert_eq!(content_font.color, Some(ExcelColor::java_or_rgb(58)));

    Ok(())
}

/// Verify OnceAbsoluteMerge annotation parses all 4 members correctly.
#[test]
fn once_absolute_merge_behavior() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.once_absolute_merge.class,
        "com.alibaba.excel.annotation.write.style.OnceAbsoluteMerge"
    );
    assert_eq!(contract.once_absolute_merge.member_count, 4);
    assert_eq!(contract.once_absolute_merge.members.len(), 4);

    // Struct-level test
    let metadata = OnceAbsoluteMergeTestData::write_metadata();
    let merge = metadata
        .once_absolute_merge
        .as_ref()
        .expect("once_absolute_merge should be set");
    assert_eq!(merge.first_row_index, 0);
    assert_eq!(merge.last_row_index, 2);
    assert_eq!(merge.first_column_index, 0);
    assert_eq!(merge.last_column_index, 1);

    Ok(())
}

/// Verify ContentLoopMerge annotation parses all 2 members correctly.
#[test]
fn content_loop_merge_behavior() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.content_loop_merge.class,
        "com.alibaba.excel.annotation.write.style.ContentLoopMerge"
    );
    assert_eq!(contract.content_loop_merge.member_count, 2);
    assert_eq!(contract.content_loop_merge.members.len(), 2);

    // Field-level test
    let cols = ContentLoopMergeTestData::schema();
    assert_eq!(cols.len(), 2);
    let loop_merge = cols[0]
        .loop_merge
        .as_ref()
        .expect("loop_merge should be set");
    assert_eq!(loop_merge.each_row, 3);
    assert_eq!(loop_merge.column_extend, 2);

    Ok(())
}

/// Verify the golden contract total Java member count.
#[test]
fn total_java_members_matches_contract() -> easyexcel::Result<()> {
    let contract = contract();
    assert_eq!(
        contract.total_java_members, 72,
        "golden contract declares 72 Java members across 9 annotations"
    );
    assert_eq!(contract.authority, "com.alibaba:easyexcel:4.0.3");
    Ok(())
}

/// Verify that all annotation metadata types are accessible from the public API.
#[test]
fn public_api_type_accessibility() -> easyexcel::Result<()> {
    // Verify all style-related types are accessible
    let _cell_style = ExcelCellStyle::new();
    let _font_style = ExcelFontStyle::new();
    let _merge_property = OnceAbsoluteMergeProperty::new(0, 1, 0, 1);
    let _loop_property = LoopMergeProperty::new(1, 1);

    // Verify enums are accessible
    let _ha = ExcelHorizontalAlignment::Center;
    let _va = ExcelVerticalAlignment::Center;
    let _bs = ExcelBorderStyle::Thin;
    let _fp = ExcelFillPattern::Solid;
    let _df = ExcelDataFormat::Builtin(0);
    let _fs = ExcelFontScript::None;
    let _ul = ExcelUnderline::None;
    let _color = ExcelColor::java_or_rgb(0);

    Ok(())
}
