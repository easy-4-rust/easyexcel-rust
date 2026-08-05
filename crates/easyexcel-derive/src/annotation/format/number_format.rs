//! Java `com.alibaba.excel.annotation.format.NumberFormat` 的属性解析。

use syn::meta::ParseNestedMeta;

use crate::annotation::field_options::FieldOptions;

/// 解析数字格式与舍入模式。
pub(crate) fn parse(meta: &ParseNestedMeta<'_>, options: &mut FieldOptions) -> syn::Result<bool> {
    if meta.path.is_ident("number_format") {
        options.number_format = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    if meta.path.is_ident("rounding_mode") {
        options.rounding_mode = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    Ok(false)
}
