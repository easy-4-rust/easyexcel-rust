//! `HeadStyle` 与 `ContentStyle` 共用的单元格样式解析。

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Lit, LitBool, meta::ParseNestedMeta};

use crate::annotation::integer::{parse_signed_i32, parse_unsigned_integer};

use super::{
    BORDER_STYLE_VARIANTS, FILL_PATTERN_VARIANTS, HORIZONTAL_ALIGNMENT_VARIANTS,
    VERTICAL_ALIGNMENT_VARIANTS, parse_named_variant,
};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 Java 单元格样式的全部可配置字段。
pub(crate) fn parse_cell_style(
    meta: &ParseNestedMeta<'_>,
    crate_path: &TokenStream,
) -> syn::Result<TokenStream> {
    let mut assignments = Vec::new();
    meta.parse_nested_meta(|property| {
        let name = property
            .path
            .get_ident()
            .ok_or_else(|| property.error("style property must be an identifier"))?;
        if let Some(assignment) = parse_scalar(&property, name, crate_path)? {
            assignments.push(assignment);
            return Ok(());
        }
        let field = format_ident!("{name}");
        match name.to_string().as_str() {
            "horizontal_alignment" => {
                let value = parse_named_variant(
                    &property,
                    crate_path,
                    "ExcelHorizontalAlignment",
                    HORIZONTAL_ALIGNMENT_VARIANTS,
                )?;
                assignments.push(
                    quote!(style.horizontal_alignment = ::core::option::Option::Some(#value);),
                );
            }
            "vertical_alignment" => {
                let value = parse_named_variant(
                    &property,
                    crate_path,
                    "ExcelVerticalAlignment",
                    VERTICAL_ALIGNMENT_VARIANTS,
                )?;
                assignments
                    .push(quote!(style.vertical_alignment = ::core::option::Option::Some(#value);));
            }
            "border_left" | "border_right" | "border_top" | "border_bottom" => {
                let value = parse_named_variant(
                    &property,
                    crate_path,
                    "ExcelBorderStyle",
                    BORDER_STYLE_VARIANTS,
                )?;
                assignments.push(quote!(style.#field = ::core::option::Option::Some(#value);));
            }
            "fill_pattern" | "fill_pattern_type" => {
                let value = parse_named_variant(
                    &property,
                    crate_path,
                    "ExcelFillPattern",
                    FILL_PATTERN_VARIANTS,
                )?;
                assignments
                    .push(quote!(style.fill_pattern = ::core::option::Option::Some(#value);));
            }
            _ => return Err(property.error("unsupported cell style property")),
        }
        Ok(())
    })?;
    Ok(quote!({
        let mut style = #crate_path::ExcelCellStyle::new();
        #(#assignments)*
        style
    }))
}

fn parse_scalar(
    property: &ParseNestedMeta<'_>,
    name: &syn::Ident,
    crate_path: &TokenStream,
) -> syn::Result<Option<TokenStream>> {
    let field = format_ident!("{name}");
    let assignment = match name.to_string().as_str() {
        "hidden" | "locked" | "quote_prefix" | "wrapped" | "shrink_to_fit" => {
            let value: LitBool = property.value()?.parse()?;
            quote!(style.#field = ::core::option::Option::Some(#value);)
        }
        "left_border_color"
        | "right_border_color"
        | "top_border_color"
        | "bottom_border_color"
        | "fill_background_color"
        | "fill_foreground_color" => {
            let value = parse_unsigned_integer::<u32>(property)?;
            quote!(style.#field = ::core::option::Option::Some(#crate_path::ExcelColor::java_or_rgb(#value));)
        }
        "rotation" => {
            let value = parse_signed_i32(property)?;
            i16::try_from(value.value()).map_err(|error| syn::Error::new(value.span(), error))?;
            let value = value.tokens();
            quote!(style.rotation = ::core::option::Option::Some(#value);)
        }
        "indent" => {
            let value = parse_unsigned_integer::<u8>(property)?;
            quote!(style.indent = ::core::option::Option::Some(#value);)
        }
        "data_format" => {
            let value: Lit = property.value()?.parse()?;
            match value {
                Lit::Str(value) => {
                    quote!(style.data_format = ::core::option::Option::Some(#crate_path::ExcelDataFormat::Custom(#value));)
                }
                Lit::Int(value) => {
                    value
                        .base10_parse::<u8>()
                        .map_err(|error| syn::Error::new_spanned(&value, error))?;
                    quote!(style.data_format = ::core::option::Option::Some(#crate_path::ExcelDataFormat::Builtin(#value));)
                }
                value => {
                    return Err(syn::Error::new_spanned(
                        value,
                        "data format must be a built-in index or custom format string",
                    ));
                }
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(assignment))
}
