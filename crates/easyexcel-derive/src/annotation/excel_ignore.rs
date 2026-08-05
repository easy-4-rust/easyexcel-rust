//! Java `com.alibaba.excel.annotation.ExcelIgnore` 的字段忽略语义。

use super::field_options::FieldOptions;

/// 应用字段级忽略标记；该标记优先于 `ExcelProperty`。
pub(crate) fn apply(options: &mut FieldOptions) {
    options.ignore = true;
}
