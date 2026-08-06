/// Direction used when expanding a collection placeholder.
/// 对应 Java：`com.alibaba.excel.enums.WriteDirectionEnum`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FillDirection {
    /// Repeats the template row downward.
    #[default]
    Vertical,
    /// Repeats the template cell to the right.
    Horizontal,
}

