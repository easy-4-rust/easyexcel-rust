//! 对应 Java：`com.alibaba.excel.write.style.column.*`.

pub mod abstract_column_width_style_strategy;
pub mod abstract_head_column_width_style_strategy;
pub mod longest_match_column_width_style_strategy;
pub mod simple_column_width_style_strategy;

pub use abstract_column_width_style_strategy::AbstractColumnWidthStyleStrategy;
pub use abstract_head_column_width_style_strategy::AbstractHeadColumnWidthStyleStrategy;
pub use longest_match_column_width_style_strategy::LongestMatchColumnWidthStyleStrategy;
pub use simple_column_width_style_strategy::SimpleColumnWidthStyleStrategy;
