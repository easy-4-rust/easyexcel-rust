//! `ExcelRow` 派生宏的结构体级编排。

use std::collections::BTreeMap;

use syn::{Data, DeriveInput, Fields};

use crate::annotation::parse_struct_options;
use crate::crate_path::easyexcel_path;

use super::field_expansion::expand_field;
use super::trait_impl::{ExpansionParts, render_trait_impl};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析调用方令牌并生成 `ExcelRow` 实现。
pub(crate) fn expand_excel_row_tokens(
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    expand_excel_row(syn::parse2(input)?)
}

/// 根据已经解析的结构体生成完整的 `ExcelRow` 实现。
fn expand_excel_row(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let crate_path = easyexcel_path();
    let name = input.ident;
    let struct_options = parse_struct_options(&input.attrs, &crate_path)?;
    let fields = named_fields(&name, input.data)?.named;
    let mut parts = ExpansionParts::default();
    let mut forced_index_fields = BTreeMap::<usize, String>::new();

    for field in fields {
        let expanded = expand_field(field, &crate_path, &struct_options, parts.columns.len())?;
        if let Some((index, ref field_name, span)) = expanded.forced_index
            && let Some(previous_field) = forced_index_fields.insert(index, field_name.clone())
        {
            return Err(syn::Error::new(
                span,
                format!("the index of `{previous_field}` and `{field_name}` must be different"),
            ));
        }
        parts.push(expanded);
    }

    Ok(render_trait_impl(
        &name,
        &input.generics,
        &crate_path,
        &struct_options,
        parts,
    ))
}

fn named_fields(name: &syn::Ident, data: Data) -> syn::Result<syn::FieldsNamed> {
    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => Ok(fields),
            _ => Err(syn::Error::new_spanned(
                name,
                "ExcelRow requires a struct with named fields",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            name,
            "ExcelRow can only be derived for structs",
        )),
    }
}
