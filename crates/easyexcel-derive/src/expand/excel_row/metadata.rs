//! 类型级写入注解到 `ExcelWriteMetadata` 的生成。

use proc_macro2::TokenStream;
use quote::quote;

use crate::annotation::StructOptions;

/// 生成类型级默认列宽、行高、样式、字体和绝对合并元数据。
pub(super) fn write_metadata_tokens(
    crate_path: &TokenStream,
    options: &StructOptions,
) -> TokenStream {
    let mut metadata = quote!(#crate_path::ExcelWriteMetadata::new());
    if let Some(value) = options
        .column_width
        .as_ref()
        .filter(|value| value.value() >= 0)
        .map(crate::annotation::SignedInteger::tokens)
    {
        metadata = quote!(#metadata.column_width(#value));
    }
    if let Some(value) = options
        .head_row_height
        .as_ref()
        .filter(|value| value.value() >= 0)
        .map(crate::annotation::SignedInteger::tokens)
    {
        metadata = quote!(#metadata.head_row_height(#value));
    }
    if let Some(value) = options
        .content_row_height
        .as_ref()
        .filter(|value| value.value() >= 0)
        .map(crate::annotation::SignedInteger::tokens)
    {
        metadata = quote!(#metadata.content_row_height(#value));
    }
    if let Some(value) = &options.head_style {
        metadata = quote!(#metadata.head_style(#value));
    }
    if let Some(value) = &options.content_style {
        metadata = quote!(#metadata.content_style(#value));
    }
    if let Some(value) = &options.head_font_style {
        metadata = quote!(#metadata.head_font_style(#value));
    }
    if let Some(value) = &options.content_font_style {
        metadata = quote!(#metadata.content_font_style(#value));
    }
    if let Some(value) = &options.once_absolute_merge {
        metadata = quote!(#metadata.once_absolute_merge(#value));
    }
    metadata
}
