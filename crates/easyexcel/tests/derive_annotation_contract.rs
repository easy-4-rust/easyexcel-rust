//! Java annotation contracts exposed through the `easyexcel` facade.

use easyexcel::{ExcelRow, NumberRoundingMode};

#[derive(ExcelRow)]
#[excel(
    ignore_unannotated,
    column_width = 20,
    head_row_height = 24,
    content_row_height = 18,
    head_style(horizontal_alignment = "center", wrapped = true),
    content_style(vertical_alignment = "bottom", shrink_to_fit = true),
    head_font_style(font_name = "Arial", bold = true),
    content_font_style(font_name = "Calibri", italic = true),
    once_absolute_merge(
        first_row_index = 0,
        last_row_index = 0,
        first_column_index = 0,
        last_column_index = 1
    )
)]
struct CompleteAnnotationRow {
    #[excel(
        value = ["订单", "编号"],
        index = 0,
        order = 9,
        column_width = 16,
        head_style(fill_pattern_type = "solid", fill_foreground_color = 42),
        content_style(locked = true, data_format = "0"),
        head_font_style(font_height_in_points = 12, underline = "single"),
        content_font_style(color = 10, type_offset = "superscript"),
        content_loop_merge(each_row = 2, column_extend = 1)
    )]
    id: String,
    #[excel(name = "金额", number_format = "0.00", rounding_mode = "HALF_DOWN")]
    amount: f64,
    #[excel(
        name = "日期",
        date_time_format = "%Y-%m-%d",
        use_1904_windowing = true
    )]
    date: String,
    #[excel(number_format = "0.0")]
    _format_only_is_not_property: f64,
    #[excel(ignore, name = "忽略")]
    _ignored: String,
}

#[test]
fn exposes_all_java_annotation_metadata() {
    let schema = CompleteAnnotationRow::schema();
    assert_eq!(schema.len(), 3);
    assert_eq!(schema[0].head_names, Some(&["订单", "编号"][..]));
    assert_eq!(schema[0].leaf_name(), "编号");
    assert_eq!(schema[0].index, Some(0));
    assert_eq!(schema[0].order, 9);
    assert_eq!(schema[0].column_width, Some(16));
    assert_eq!(schema[0].loop_merge.expect("loop merge").each_row, 2);
    assert!(schema[0].head_style.is_some());
    assert!(schema[0].content_style.is_some());
    assert!(schema[0].head_font_style.is_some());
    assert!(schema[0].content_font_style.is_some());
    assert_eq!(schema[1].number_format, Some("0.00"));
    assert_eq!(schema[1].date_time_format, None);
    assert_eq!(
        schema[1].number_rounding_mode,
        Some(NumberRoundingMode::HalfDown)
    );
    assert_eq!(schema[2].date_time_format, Some("%Y-%m-%d"));
    assert_eq!(schema[2].number_format, None);
    assert_eq!(schema[2].use_1904_windowing, Some(true));

    let metadata = CompleteAnnotationRow::write_metadata();
    assert_eq!(metadata.column_width, Some(20));
    assert_eq!(metadata.head_row_height, Some(24));
    assert_eq!(metadata.content_row_height, Some(18));
    assert!(metadata.head_style.is_some());
    assert!(metadata.content_style.is_some());
    assert!(metadata.head_font_style.is_some());
    assert!(metadata.content_font_style.is_some());
    assert_eq!(
        metadata
            .once_absolute_merge
            .expect("absolute merge")
            .last_column_index,
        1
    );
}
