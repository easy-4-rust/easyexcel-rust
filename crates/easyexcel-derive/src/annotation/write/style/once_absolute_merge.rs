//! Java `com.alibaba.excel.annotation.write.style.OnceAbsoluteMerge` 的属性解析。

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::token::Paren;

use crate::annotation::integer::{SignedInteger, parse_signed_i32};
use crate::annotation::struct_options::StructOptions;

/// 解析一次性绝对合并区域。
pub(crate) fn parse(
    meta: &ParseNestedMeta<'_>,
    options: &mut StructOptions,
    crate_path: &TokenStream,
) -> syn::Result<bool> {
    if !meta.path.is_ident("once_absolute_merge") {
        return Ok(false);
    }
    let mut first_row_index: Option<SignedInteger> = None;
    let mut last_row_index: Option<SignedInteger> = None;
    let mut first_column_index: Option<SignedInteger> = None;
    let mut last_column_index: Option<SignedInteger> = None;
    let has_empty_parentheses = if meta.input.peek(Paren) {
        let ahead = meta.input.fork();
        let content;
        syn::parenthesized!(content in ahead);
        content.is_empty()
    } else {
        false
    };
    if has_empty_parentheses {
        let _content;
        syn::parenthesized!(_content in meta.input);
    } else if !meta.input.is_empty() {
        meta.parse_nested_meta(|property| {
            if property.path.is_ident("first_row_index") {
                first_row_index = Some(parse_signed_i32(&property)?);
                return Ok(());
            }
            if property.path.is_ident("last_row_index") {
                last_row_index = Some(parse_signed_i32(&property)?);
                return Ok(());
            }
            if property.path.is_ident("first_column_index") {
                first_column_index = Some(parse_signed_i32(&property)?);
                return Ok(());
            }
            if property.path.is_ident("last_column_index") {
                last_column_index = Some(parse_signed_i32(&property)?);
                return Ok(());
            }
            Err(property.error("unsupported once_absolute_merge property"))
        })?;
    }
    let default = || SignedInteger::new(-1, Span::call_site());
    let first_row_index = first_row_index.unwrap_or_else(default).tokens();
    let last_row_index = last_row_index.unwrap_or_else(default).tokens();
    let first_column_index = first_column_index.unwrap_or_else(default).tokens();
    let last_column_index = last_column_index.unwrap_or_else(default).tokens();
    options.once_absolute_merge = Some(quote!(
        #crate_path::OnceAbsoluteMergeProperty::new(
            #first_row_index,
            #last_row_index,
            #first_column_index,
            #last_column_index,
        )
    ));
    Ok(true)
}
