//! Java `com.alibaba.excel.annotation.ExcelProperty` 的属性解析。

use syn::meta::ParseNestedMeta;
use syn::{Expr, ExprArray, ExprLit, Lit, LitStr};

use super::field_options::FieldOptions;
use super::integer::parse_signed_i32;

/// 解析 `name`、`head/value`、`index`、`order` 与 `converter`。
pub(crate) fn parse(meta: &ParseNestedMeta<'_>, options: &mut FieldOptions) -> syn::Result<bool> {
    if meta.path.is_ident("property") {
        options.property_annotated = true;
        return Ok(true);
    }
    if meta.path.is_ident("name") {
        options.property_annotated = true;
        options.name = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    if meta.path.is_ident("head") || meta.path.is_ident("value") {
        options.property_annotated = true;
        options.head_names = Some(parse_head_names(meta)?);
        return Ok(true);
    }
    if meta.path.is_ident("index") {
        options.property_annotated = true;
        options.index = Some(parse_signed_i32(meta)?);
        return Ok(true);
    }
    if meta.path.is_ident("order") {
        options.property_annotated = true;
        options.order = Some(parse_signed_i32(meta)?);
        return Ok(true);
    }
    if meta.path.is_ident("converter") {
        options.property_annotated = true;
        options.converter = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    if meta.path.is_ident("format") {
        options.property_annotated = true;
        options.legacy_format = Some(meta.value()?.parse()?);
        return Ok(true);
    }
    Ok(false)
}

/// 解析 Java `ExcelProperty.value()` 对应的单级或多级表头。
fn parse_head_names(meta: &ParseNestedMeta<'_>) -> syn::Result<Vec<LitStr>> {
    let expression: Expr = meta.value()?.parse()?;
    let values = match expression {
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => vec![value],
        Expr::Array(ExprArray { elems, .. }) => elems
            .into_iter()
            .map(|element| match element {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) => Ok(value),
                other => Err(syn::Error::new_spanned(
                    other,
                    "head entries must be string literals",
                )),
            })
            .collect::<syn::Result<Vec<_>>>()?,
        other => {
            return Err(syn::Error::new_spanned(
                other,
                "head must be a string literal or an array of string literals",
            ));
        }
    };
    if values.is_empty() {
        return Err(meta.error("head must contain at least one label"));
    }
    Ok(values)
}
