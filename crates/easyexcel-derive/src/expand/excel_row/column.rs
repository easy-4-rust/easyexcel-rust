//! 字段注解到静态 `ExcelColumn` 元数据的生成。

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Type};

use crate::annotation::{FieldOptions, SignedInteger, number_rounding_mode_tokens};

/// 构造并装饰单个静态列定义。
pub(super) fn build_column(
    crate_path: &TokenStream,
    ident: &Ident,
    ty: &Type,
    options: FieldOptions,
) -> syn::Result<TokenStream> {
    if options.name.is_some() && options.head_names.is_some() {
        return Err(syn::Error::new_spanned(
            ident,
            "name cannot be combined with head or value",
        ));
    }
    let field_name = ident.to_string();
    let header_name = options
        .head_names
        .as_ref()
        .and_then(|values| values.last().cloned())
        .or(options.name)
        .unwrap_or_else(|| LitStr::new(&field_name, ident.span()));
    let index = match options.index {
        Some(value) if value.value() == -1 => quote!(::core::option::Option::None),
        Some(value) if value.value() >= 0 => {
            let value = usize::try_from(value.value())
                .map_err(|error| syn::Error::new(value.span(), error))?;
            quote!(::core::option::Option::Some(#value))
        }
        Some(value) => {
            return Err(syn::Error::new(
                value.span(),
                "index must be -1 or a non-negative integer",
            ));
        }
        None => quote!(::core::option::Option::None),
    };
    let order = options
        .order
        .map_or_else(|| quote!(i32::MAX), |value| value.tokens());
    let legacy_format = options.legacy_format.as_ref().map_or_else(
        || quote!(::core::option::Option::None),
        |value| quote!(::core::option::Option::Some(#value)),
    );
    let column = quote!(
        #crate_path::ExcelColumn::new(#field_name, #header_name, #index, #order, #legacy_format)
            .with_field_type(::core::stringify!(#ty))
    );
    let mut column = column;
    if let Some(value) = options.date_time_format {
        column = quote!(#column.with_date_time_format(#value));
    }
    if let Some(value) = options.number_format {
        column = quote!(#column.with_number_format(#value));
    }
    if let Some(value) = options.legacy_format {
        column = quote!(#column.with_legacy_format(#value));
    }
    let mut column = decorate_column(
        column,
        options.head_names,
        options.column_width,
        options.head_style,
        options.content_style,
        options.head_font_style,
        options.content_font_style,
        options.content_loop_merge,
        options.extensions.image,
        options.extensions.comment,
        options.extensions.hyperlink,
        options.extensions.formula,
        options.extensions.data_validation,
        options.extensions.conditional,
        options.extensions.filter,
    );
    if let Some(use_1904_windowing) = options.use_1904_windowing {
        column = quote!(#column.with_use_1904_windowing(#use_1904_windowing));
    }
    if let Some(rounding_mode) = options.rounding_mode {
        let rounding_mode = number_rounding_mode_tokens(&rounding_mode, crate_path)?;
        column = quote!(#column.with_number_rounding_mode(#rounding_mode));
    }
    Ok(column)
}

#[allow(clippy::too_many_arguments)]
fn decorate_column(
    mut column: TokenStream,
    head_names: Option<Vec<LitStr>>,
    width: Option<SignedInteger>,
    head_style: Option<TokenStream>,
    content_style: Option<TokenStream>,
    head_font_style: Option<TokenStream>,
    content_font_style: Option<TokenStream>,
    content_loop_merge: Option<TokenStream>,
    image: Option<LitStr>,
    comment: Option<LitStr>,
    hyperlink: Option<LitStr>,
    formula: Option<LitStr>,
    data_validation: Option<TokenStream>,
    conditional: Option<TokenStream>,
    filter: bool,
) -> TokenStream {
    if let Some(value) = head_names {
        column = quote!(#column.with_head_names(&[#(#value),*]));
    }
    if let Some(value) = width.filter(|value| value.value() >= 0) {
        let value = value.tokens();
        column = quote!(#column.with_column_width(#value));
    }
    if let Some(value) = head_style {
        column = quote!(#column.with_head_style(#value));
    }
    if let Some(value) = content_style {
        column = quote!(#column.with_content_style(#value));
    }
    if let Some(value) = head_font_style {
        column = quote!(#column.with_head_font_style(#value));
    }
    if let Some(value) = content_font_style {
        column = quote!(#column.with_content_font_style(#value));
    }
    if let Some(value) = content_loop_merge {
        column = quote!(#column.with_loop_merge(#value));
    }
    if let Some(value) = image {
        column = quote!(#column.with_image_path(#value));
    }
    if let Some(value) = comment {
        column = quote!(#column.with_comment(#value));
    }
    if let Some(value) = hyperlink {
        column = quote!(#column.with_hyperlink(#value));
    }
    if let Some(value) = formula {
        column = quote!(#column.with_formula(#value));
    }
    if let Some(value) = data_validation {
        column = quote!(#column.with_data_validation(#value));
    }
    if let Some(value) = conditional {
        column = quote!(#column.with_conditional_format(#value));
    }
    if filter {
        column = quote!(#column.with_auto_filter());
    }
    column
}
