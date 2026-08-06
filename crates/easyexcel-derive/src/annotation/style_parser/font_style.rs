//! `HeadFontStyle` 与 `ContentFontStyle` 共用的字体样式解析。

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Lit, LitBool, LitStr, meta::ParseNestedMeta};

use crate::annotation::integer::parse_unsigned_integer;

use super::parse_named_variant;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 Java 字体注解的全部九个属性。
pub(crate) fn parse_font_style(
    meta: &ParseNestedMeta<'_>,
    crate_path: &TokenStream,
) -> syn::Result<TokenStream> {
    let mut assignments = Vec::new();
    meta.parse_nested_meta(|property| {
        let name = property.path.get_ident().ok_or_else(|| property.error("font property must be an identifier"))?;
        match name.to_string().as_str() {
            "font_name" => {
                let value: LitStr = property.value()?.parse()?;
                assignments.push(quote!(style.font_name = ::core::option::Option::Some(#value);));
            }
            "font_height_in_points" => {
                let value: Lit = property.value()?.parse()?;
                let numeric = match &value {
                    Lit::Int(value) => value.base10_parse::<f64>(),
                    Lit::Float(value) => value.base10_parse::<f64>(),
                    _ => return Err(syn::Error::new_spanned(value, "font height must be numeric")),
                }.unwrap_or(f64::NAN);
                if !numeric.is_finite() || numeric <= 0.0 {
                    return Err(syn::Error::new_spanned(value, "font height must be positive"));
                }
                assignments.push(quote!(style.font_height_in_points = ::core::option::Option::Some(#numeric);));
            }
            "italic" | "strikeout" | "bold" => {
                let field = format_ident!("{name}");
                let value: LitBool = property.value()?.parse()?;
                assignments.push(quote!(style.#field = ::core::option::Option::Some(#value);));
            }
            "color" => {
                let value = parse_unsigned_integer::<u32>(&property)?;
                assignments.push(quote!(style.color = ::core::option::Option::Some(#crate_path::ExcelColor::java_or_rgb(#value));));
            }
            "charset" => {
                let value = parse_unsigned_integer::<u8>(&property)?;
                assignments.push(quote!(style.charset = ::core::option::Option::Some(#value);));
            }
            "type_offset" => {
                let value = parse_named_variant(&property, crate_path, "ExcelFontScript", &[("none", "None"), ("superscript", "Superscript"), ("subscript", "Subscript")])?;
                assignments.push(quote!(style.type_offset = ::core::option::Option::Some(#value);));
            }
            "underline" => {
                let value = parse_named_variant(&property, crate_path, "ExcelUnderline", &[("none", "None"), ("single", "Single"), ("double", "Double"), ("single_accounting", "SingleAccounting"), ("double_accounting", "DoubleAccounting")])?;
                assignments.push(quote!(style.underline = ::core::option::Option::Some(#value);));
            }
            _ => return Err(property.error("unsupported font style property")),
        }
        Ok(())
    })?;
    Ok(quote!({
        let mut style = #crate_path::ExcelFontStyle::new();
        #(#assignments)*
        style
    }))
}
