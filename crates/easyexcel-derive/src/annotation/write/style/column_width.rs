//! Java `com.alibaba.excel.annotation.write.style.ColumnWidth` 的属性解析。

use syn::meta::ParseNestedMeta;

use crate::annotation::field_options::FieldOptions;
use crate::annotation::struct_options::StructOptions;
use crate::annotation::style_parser::parse_dimension;

/// 对应 Java：com.alibaba.excel.annotation.write.style.ColumnWidth。 解析字段级列宽。
pub(crate) fn parse_field(
    meta: &ParseNestedMeta<'_>,
    options: &mut FieldOptions,
) -> syn::Result<bool> {
    if !meta.path.is_ident("column_width") {
        return Ok(false);
    }
    options.column_width = Some(parse_dimension(meta)?);
    Ok(true)
}

/// 对应 Java：com.alibaba.excel.annotation.write.style.ColumnWidth。 解析类型级默认列宽。
pub(crate) fn parse_struct(
    meta: &ParseNestedMeta<'_>,
    options: &mut StructOptions,
) -> syn::Result<bool> {
    if !meta.path.is_ident("column_width") {
        return Ok(false);
    }
    options.column_width = Some(parse_dimension(meta)?);
    Ok(true)
}
