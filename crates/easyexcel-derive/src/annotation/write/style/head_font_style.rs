//! Java `com.alibaba.excel.annotation.write.style.HeadFontStyle` 的属性解析。

use proc_macro2::TokenStream;
use syn::meta::ParseNestedMeta;

use crate::annotation::field_options::FieldOptions;
use crate::annotation::struct_options::StructOptions;
use crate::annotation::style_parser::parse_font_style;

/// 对应 Java：com.alibaba.excel.annotation.write.style.HeadFontStyle。 解析字段级表头字体。
pub(crate) fn parse_field(
    meta: &ParseNestedMeta<'_>,
    options: &mut FieldOptions,
    crate_path: &TokenStream,
) -> syn::Result<bool> {
    if !meta.path.is_ident("head_font_style") {
        return Ok(false);
    }
    options.head_font_style = Some(parse_font_style(meta, crate_path)?);
    Ok(true)
}

/// 对应 Java：com.alibaba.excel.annotation.write.style.HeadFontStyle。 解析类型级表头字体。
pub(crate) fn parse_struct(
    meta: &ParseNestedMeta<'_>,
    options: &mut StructOptions,
    crate_path: &TokenStream,
) -> syn::Result<bool> {
    if !meta.path.is_ident("head_font_style") {
        return Ok(false);
    }
    options.head_font_style = Some(parse_font_style(meta, crate_path)?);
    Ok(true)
}
