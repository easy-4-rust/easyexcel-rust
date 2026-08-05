//! Java `com.alibaba.excel.annotation.write.style.HeadStyle` 的属性解析。

use proc_macro2::TokenStream;
use syn::meta::ParseNestedMeta;

use crate::annotation::field_options::FieldOptions;
use crate::annotation::struct_options::StructOptions;
use crate::annotation::style_parser::parse_cell_style;

/// 解析字段级表头样式。
pub(crate) fn parse_field(
    meta: &ParseNestedMeta<'_>,
    options: &mut FieldOptions,
    crate_path: &TokenStream,
) -> syn::Result<bool> {
    if !meta.path.is_ident("head_style") {
        return Ok(false);
    }
    options.head_style = Some(parse_cell_style(meta, crate_path)?);
    Ok(true)
}

/// 解析类型级表头样式。
pub(crate) fn parse_struct(
    meta: &ParseNestedMeta<'_>,
    options: &mut StructOptions,
    crate_path: &TokenStream,
) -> syn::Result<bool> {
    if !meta.path.is_ident("head_style") {
        return Ok(false);
    }
    options.head_style = Some(parse_cell_style(meta, crate_path)?);
    Ok(true)
}
