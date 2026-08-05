use easyexcel::ExcelRow;

struct NoDefault(&'static str);

#[derive(ExcelRow)]
#[excel(
    column_width = -1,
    head_row_height = -1,
    content_row_height = -1,
    once_absolute_merge()
)]
struct SignedDefaultsAndFormats {
    #[excel(
        property,
        order = -10,
        date_time_format = "%Y-%m-%d",
        number_format = "0.00",
        column_width = -1
    )]
    value: String,
    #[excel(ignore, default = NoDefault("ignored"))]
    ignored: NoDefault,
}

fn main() {
    let column = &SignedDefaultsAndFormats::schema()[0];
    assert_eq!(column.order, -10);
    assert_eq!(column.date_time_format, Some("%Y-%m-%d"));
    assert_eq!(column.number_format, Some("0.00"));
    assert_eq!(column.column_width, None);
    assert_eq!(SignedDefaultsAndFormats::write_metadata().column_width, None);
    assert_eq!(SignedDefaultsAndFormats::write_metadata()
        .once_absolute_merge
        .expect("merge defaults")
        .first_row_index, -1);

    let value = SignedDefaultsAndFormats {
        value: String::new(),
        ignored: NoDefault("ignored"),
    };
    let _ = value.ignored.0;
}
