//! `data_validation(...)` 属性解析。

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{LitStr, meta::ParseNestedMeta};

/// 将数据校验属性转换为运行时元数据构造表达式。
pub(super) fn parse_data_validation(
    meta: &ParseNestedMeta<'_>,
    crate_path: &TokenStream,
) -> syn::Result<TokenStream> {
    let mut data_type: Option<LitStr> = None;
    let mut operator: Option<LitStr> = None;
    let mut formula1: Option<LitStr> = None;
    let mut formula2: Option<LitStr> = None;
    meta.parse_nested_meta(|property| {
        if property.path.is_ident("type") {
            data_type = Some(property.value()?.parse()?);
            return Ok(());
        }
        if property.path.is_ident("operator") {
            operator = Some(property.value()?.parse()?);
            return Ok(());
        }
        if property.path.is_ident("formula1") {
            formula1 = Some(property.value()?.parse()?);
            return Ok(());
        }
        if property.path.is_ident("formula2") {
            formula2 = Some(property.value()?.parse()?);
            return Ok(());
        }
        Err(property.error("unsupported data_validation property"))
    })?;
    let default = |value: &str| LitStr::new(value, Span::call_site());
    let data_type = data_type.unwrap_or_else(|| default("list"));
    let operator = operator.unwrap_or_else(|| default("between"));
    let formula1 = formula1.unwrap_or_else(|| default(""));
    let formula2 = formula2.unwrap_or_else(|| default(""));
    Ok(quote!(#crate_path::ExcelDataValidationMeta::new(
        #data_type,
        #operator,
        #formula1,
        #formula2,
    )))
}
