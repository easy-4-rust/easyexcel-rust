use easyexcel_model::{CellValue, DateSystem};

/// 流式读取产生的一个非空单元格。
#[derive(Debug, Clone, PartialEq)]
pub struct StreamCell {
    /// 从零开始的列索引。
    pub col: u32,
    /// 标量值；公式单元格使用缓存值。
    pub value: CellValue,
    /// 数字格式代码；空字符串表示 General。
    pub number_format: String,
}

impl StreamCell {
    /// 按工作簿日期系统和数字格式渲染显示值。
    #[must_use]
    pub fn display(&self, date_system: DateSystem) -> String {
        match &self.value {
            CellValue::Number(number)
                if !self.number_format.is_empty()
                    && !self.number_format.eq_ignore_ascii_case("general") =>
            {
                easyexcel_model::numfmt::format_value(*number, &self.number_format, date_system)
            }
            CellValue::Number(number) => easyexcel_model::value::format_number_general(*number),
            value => value.to_display_string(),
        }
    }
}
