//! 单个 Rust 字段到列元数据与行转换令牌的展开。

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Field;

use crate::annotation::{StructOptions, parse_field_options};

use super::column::build_column;
use super::field_tokens::build_field_tokens;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 单字段展开结果。
pub(super) struct FieldExpansion {
    pub(super) column: Option<TokenStream>,
    pub(super) readers: TokenStream,
    pub(super) registered_readers: TokenStream,
    pub(super) writers: Option<TokenStream>,
    pub(super) original_writers: Option<TokenStream>,
    pub(super) selected_original_writers: Option<TokenStream>,
    pub(super) registered_writers: Option<TokenStream>,
    pub(super) registered_write_cell_data: Option<TokenStream>,
    pub(super) selected_registered_write_cell_data: Option<TokenStream>,
    pub(super) forced_index: Option<(usize, String, Span)>,
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 展开一个具名字段，并保持 `ExcelIgnore` 的最高优先级。
pub(super) fn expand_field(
    field: Field,
    crate_path: &TokenStream,
    struct_options: &StructOptions,
    schema_position: usize,
) -> syn::Result<FieldExpansion> {
    let options = parse_field_options(&field.attrs, crate_path)?;
    let ident = field
        .ident
        .ok_or_else(|| syn::Error::new_spanned(&field.ty, "ExcelRow requires named fields"))?;
    let ty = field.ty;
    if options.ignore || (struct_options.ignore_unannotated && !options.property_annotated) {
        let default = options
            .extensions
            .default
            .unwrap_or_else(|| syn::parse_quote!(::core::default::Default::default()));
        return Ok(FieldExpansion {
            column: None,
            readers: quote!(#ident: #default),
            registered_readers: quote!(#ident: #default),
            writers: None,
            original_writers: None,
            selected_original_writers: None,
            registered_writers: None,
            registered_write_cell_data: None,
            selected_registered_write_cell_data: None,
            forced_index: None,
        });
    }
    if let Some(default) = options.extensions.default.as_ref() {
        return Err(syn::Error::new_spanned(
            default,
            "default is only valid for ignored fields",
        ));
    }

    let forced_index = if let Some(index) = options.index.as_ref() {
        if index.value() == -1 {
            None
        } else if index.value() < 0 {
            return Err(syn::Error::new(
                index.span(),
                "index must be -1 or a non-negative integer",
            ));
        } else {
            Some((
                usize::try_from(index.value())
                    .map_err(|error| syn::Error::new(index.span(), error))?,
                ident.to_string(),
                index.span(),
            ))
        }
    } else {
        None
    };
    let converter = options.converter.clone();
    let column = build_column(crate_path, &ident, &ty, options)?;
    let tokens = build_field_tokens(crate_path, &ident, &ty, converter.as_ref(), schema_position);
    Ok(FieldExpansion {
        column: Some(column),
        readers: tokens.reader,
        registered_readers: tokens.registered_reader,
        writers: Some(tokens.writer),
        original_writers: Some(tokens.original_writer),
        selected_original_writers: Some(tokens.selected_original_writer),
        registered_writers: Some(tokens.registered_writer),
        registered_write_cell_data: Some(tokens.registered_write_cell),
        selected_registered_write_cell_data: Some(tokens.selected_registered_write_cell),
        forced_index,
    })
}
