//! Java `com.alibaba.excel.annotation.ExcelIgnoreUnannotated` 的类型级过滤语义。

use super::struct_options::StructOptions;

/// 对应 Java：com.alibaba.excel.annotation.ExcelIgnoreUnannotated。 仅保留显式声明了 `ExcelProperty` 等价属性的字段。
pub(crate) fn apply(options: &mut StructOptions) {
    options.ignore_unannotated = true;
}
