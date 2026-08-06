//! Java/Hutool Excel 工具兼容入口。
//!
//! 地址换算、格式识别和数字格式分析由基础 crate 实现；本模块仅保留
//! `EasyExcel` 原有工具路径和函数名称。

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将零基列下标转换为 Excel 列名。
#[must_use]
pub fn index_to_col_name(index: u32) -> String {
    easyexcel_model::addr::col_index_to_letters(index)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将 Excel 列名转换为零基列下标。
#[must_use]
pub fn col_name_to_index(name: &str) -> Option<u32> {
    easyexcel_model::addr::col_letters_to_index(name)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断字节是否为 XLS/CFB 文件头。
#[must_use]
pub fn is_xls_bytes(data: &[u8]) -> bool {
    easyexcel_io::looks_like_cfb(data)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断字节是否为 XLSX/ZIP 文件头。
#[must_use]
pub fn is_xlsx_bytes(data: &[u8]) -> bool {
    easyexcel_io::looks_like_zip(data)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断字节是否可能是 CSV 或其他分隔文本。
#[must_use]
pub fn is_csv_bytes(data: &[u8]) -> bool {
    easyexcel_io::looks_like_delimited_text(data)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断 Excel 数字格式代码是否表达日期或时间。
#[must_use]
pub fn is_date_format(format_str: &str) -> bool {
    easyexcel_model::numfmt::is_date_format(format_str)
}
