//! Java `com.alibaba.excel.annotation.format.DateTimeFormat` 的属性解析。

use syn::meta::ParseNestedMeta;

use crate::annotation::field_options::FieldOptions;

/// 解析日期格式与 1904 日期窗口配置。
pub(crate) fn parse(meta: &ParseNestedMeta<'_>, options: &mut FieldOptions) -> syn::Result<bool> {
    if meta.path.is_ident("date_time_format") {
        options.date_time_format = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    if meta.path.is_ident("use_1904_windowing") {
        options.use_1904_windowing = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    Ok(false)
}
