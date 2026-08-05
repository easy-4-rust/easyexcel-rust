use easyexcel::ExcelRow;

#[derive(ExcelRow)]
struct DefaultOnMappedField {
    #[excel(property, default = String::new())]
    value: String,
}

fn main() {}
