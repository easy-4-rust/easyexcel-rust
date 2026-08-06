//! Java `com.alibaba.excel.annotation.write.style.ContentLoopMerge` 的属性解析。

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{LitInt, meta::ParseNestedMeta};

use crate::annotation::field_options::FieldOptions;
use crate::annotation::integer::parse_unsigned_integer;

/// 对应 Java：com.alibaba.excel.annotation.write.style.ContentLoopMerge。 解析循环合并的行周期和横向扩展列数。
pub(crate) fn parse(
    meta: &ParseNestedMeta<'_>,
    options: &mut FieldOptions,
    crate_path: &TokenStream,
) -> syn::Result<bool> {
    if !meta.path.is_ident("content_loop_merge") {
        return Ok(false);
    }
    let mut each_row: Option<LitInt> = None;
    let mut column_extend: Option<LitInt> = None;
    meta.parse_nested_meta(|property| {
        if property.path.is_ident("each_row") {
            each_row = Some(parse_unsigned_integer::<u32>(&property)?);
            return Ok(());
        }
        if property.path.is_ident("column_extend") {
            column_extend = Some(parse_unsigned_integer::<u16>(&property)?);
            return Ok(());
        }
        Err(property.error("unsupported content_loop_merge property"))
    })?;
    let each_row = each_row.unwrap_or_else(|| LitInt::new("1", Span::call_site()));
    let column_extend = column_extend.unwrap_or_else(|| LitInt::new("1", Span::call_site()));
    options.content_loop_merge = Some(quote!(
        #crate_path::LoopMergeProperty::new(#each_row, #column_extend)
    ));
    Ok(true)
}
