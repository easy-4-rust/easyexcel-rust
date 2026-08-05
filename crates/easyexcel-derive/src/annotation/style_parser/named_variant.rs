//! Java 枚举名称到 Rust 样式枚举的共享映射。

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{LitStr, meta::ParseNestedMeta};

/// 解析命名枚举变量并生成目标 `EasyExcel` 枚举表达式。
pub(crate) fn parse_named_variant(
    meta: &ParseNestedMeta<'_>,
    crate_path: &TokenStream,
    enum_name: &str,
    variants: &[(&str, &str)],
) -> syn::Result<TokenStream> {
    let value: LitStr = meta.value()?.parse()?;
    let variant = variants
        .iter()
        .find_map(|(name, variant)| (*name == value.value()).then_some(*variant))
        .ok_or_else(|| syn::Error::new_spanned(&value, format!("unsupported {enum_name} value")))?;
    let enum_name = format_ident!("{enum_name}");
    let variant = format_ident!("{variant}");
    Ok(quote!(#crate_path::#enum_name::#variant))
}
