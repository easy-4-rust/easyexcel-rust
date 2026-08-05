//! EasyExcel-Rust 派生宏扩展属性。
//!
//! 本模块中的属性不属于 Java `com.alibaba.excel.annotation`，单独建模以避免
//! 将 Rust 产品能力误报为 Java 注解成员。

use proc_macro2::TokenStream;
use syn::{Expr, LitStr, meta::ParseNestedMeta};

use super::conditional_parser::parse_conditional;
use super::data_validation_parser::parse_data_validation;

/// EasyExcel-Rust 在 Java 注解语义之外提供的字段能力。
#[derive(Default)]
pub(crate) struct ExtensionOptions {
    pub(crate) image: Option<LitStr>,
    pub(crate) comment: Option<LitStr>,
    pub(crate) hyperlink: Option<LitStr>,
    pub(crate) formula: Option<LitStr>,
    pub(crate) data_validation: Option<TokenStream>,
    pub(crate) conditional: Option<TokenStream>,
    pub(crate) filter: bool,
    pub(crate) default: Option<Expr>,
}

/// 尝试解析一个 EasyExcel-Rust 字段扩展属性。
pub(crate) fn parse(
    meta: &ParseNestedMeta<'_>,
    options: &mut ExtensionOptions,
    crate_path: &TokenStream,
) -> syn::Result<bool> {
    if meta.path.is_ident("default") {
        options.default = Some(if meta.input.is_empty() {
            syn::parse_quote!(::core::default::Default::default())
        } else {
            meta.value()?.parse()?
        });
        return Ok(true);
    }
    if meta.path.is_ident("image") {
        options.image = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    if meta.path.is_ident("comment") {
        options.comment = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    if meta.path.is_ident("hyperlink") {
        options.hyperlink = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    if meta.path.is_ident("formula") {
        options.formula = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    if meta.path.is_ident("data_validation") {
        options.data_validation = Some(parse_data_validation(meta, crate_path)?);
        return Ok(true);
    }
    if meta.path.is_ident("conditional") {
        options.conditional = Some(parse_conditional(meta)?);
        return Ok(true);
    }
    if meta.path.is_ident("filter") {
        options.filter = true;
        return Ok(true);
    }
    Ok(false)
}
