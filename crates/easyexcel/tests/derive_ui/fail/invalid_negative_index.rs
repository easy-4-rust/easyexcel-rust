use easyexcel::ExcelRow;

#[derive(ExcelRow)]
struct InvalidNegativeIndex {
    #[excel(index = -2)]
    value: String,
}

fn main() {}
