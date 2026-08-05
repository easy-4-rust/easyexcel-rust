//! 类型级 `#[excel(...)]` 属性模型与解析。

use proc_macro2::TokenStream;
use syn::Attribute;

use super::SignedInteger;
use super::excel_ignore_unannotated;
use super::write::style::{
    parse_content_row_height, parse_head_row_height, parse_once_absolute_merge,
    parse_struct_column_width, parse_struct_content_font_style, parse_struct_content_style,
    parse_struct_head_font_style, parse_struct_head_style,
};

/// `ExcelRow` 派生宏支持的类型级属性。
#[derive(Default)]
pub(crate) struct StructOptions {
    pub(crate) ignore_unannotated: bool,
    pub(crate) column_width: Option<SignedInteger>,
    pub(crate) head_row_height: Option<SignedInteger>,
    pub(crate) content_row_height: Option<SignedInteger>,
    pub(crate) head_style: Option<TokenStream>,
    pub(crate) content_style: Option<TokenStream>,
    pub(crate) head_font_style: Option<TokenStream>,
    pub(crate) content_font_style: Option<TokenStream>,
    pub(crate) once_absolute_merge: Option<TokenStream>,
}

/// 解析类型级 `#[excel(...)]` 属性。
pub(crate) fn parse_struct_options(
    attrs: &[Attribute],
    crate_path: &TokenStream,
) -> syn::Result<StructOptions> {
    let mut options = StructOptions::default();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("excel")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("ignore_unannotated") {
                excel_ignore_unannotated::apply(&mut options);
                return Ok(());
            }
            if parse_struct_column_width(&meta, &mut options)?
                || parse_head_row_height(&meta, &mut options)?
                || parse_content_row_height(&meta, &mut options)?
                || parse_struct_head_style(&meta, &mut options, crate_path)?
                || parse_struct_content_style(&meta, &mut options, crate_path)?
                || parse_struct_head_font_style(&meta, &mut options, crate_path)?
                || parse_struct_content_font_style(&meta, &mut options, crate_path)?
                || parse_once_absolute_merge(&meta, &mut options, crate_path)?
            {
                return Ok(());
            }
            Err(meta.error("unsupported ExcelRow struct option"))
        })?;
    }
    Ok(options)
}
