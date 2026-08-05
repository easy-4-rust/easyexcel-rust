//! `conditional(...)` 属性解析。

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{LitStr, meta::ParseNestedMeta};

/// 将条件格式属性转换为运行时元数据元组。
pub(super) fn parse_conditional(meta: &ParseNestedMeta<'_>) -> syn::Result<TokenStream> {
    let mut condition: Option<LitStr> = None;
    let mut font_color: Option<LitStr> = None;
    let mut background_color: Option<LitStr> = None;
    meta.parse_nested_meta(|property| {
        if property.path.is_ident("condition") {
            condition = Some(property.value()?.parse()?);
            return Ok(());
        }
        if property.path.is_ident("font_color") {
            font_color = Some(property.value()?.parse()?);
            return Ok(());
        }
        if property.path.is_ident("background_color") {
            background_color = Some(property.value()?.parse()?);
            return Ok(());
        }
        Err(property.error("unsupported conditional property"))
    })?;
    let default = |value: &str| LitStr::new(value, Span::call_site());
    let condition = condition.unwrap_or_else(|| default(""));
    let font_color = font_color.unwrap_or_else(|| default(""));
    let background_color = background_color.unwrap_or_else(|| default(""));
    Ok(quote!((#condition, #font_color, #background_color)))
}
