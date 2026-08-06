//! 字段级 `#[excel(...)]` 属性模型与解析。

use proc_macro2::TokenStream;
use syn::{Attribute, LitBool, LitStr, Path};

use super::SignedInteger;
use super::excel_ignore;
use super::excel_property;
use super::extension_options::{ExtensionOptions, parse as parse_extension};
use super::format::{parse_date_time_format, parse_number_format};
use super::write::style::{
    parse_content_loop_merge, parse_field_column_width, parse_field_content_font_style,
    parse_field_content_style, parse_field_head_font_style, parse_field_head_style,
};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 `ExcelRow` 派生宏支持的字段级属性。
#[derive(Default)]
pub(crate) struct FieldOptions {
    /// 是否显式声明了 Java `@ExcelProperty` 等价配置。
    pub(crate) property_annotated: bool,
    pub(crate) ignore: bool,
    pub(crate) name: Option<LitStr>,
    pub(crate) head_names: Option<Vec<LitStr>>,
    pub(crate) index: Option<SignedInteger>,
    pub(crate) order: Option<SignedInteger>,
    /// 已废弃的 Java `ExcelProperty.format` 兼容值。
    pub(crate) legacy_format: Option<LitStr>,
    pub(crate) date_time_format: Option<LitStr>,
    pub(crate) number_format: Option<LitStr>,
    pub(crate) rounding_mode: Option<LitStr>,
    pub(crate) use_1904_windowing: Option<LitBool>,
    pub(crate) converter: Option<Path>,
    pub(crate) column_width: Option<SignedInteger>,
    pub(crate) head_style: Option<TokenStream>,
    pub(crate) content_style: Option<TokenStream>,
    pub(crate) head_font_style: Option<TokenStream>,
    pub(crate) content_font_style: Option<TokenStream>,
    pub(crate) content_loop_merge: Option<TokenStream>,
    /// EasyExcel-Rust 产品扩展，不属于 Java 注解成员。
    pub(crate) extensions: ExtensionOptions,
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析字段级 `#[excel(...)]` 属性。
pub(crate) fn parse_field_options(
    attrs: &[Attribute],
    crate_path: &TokenStream,
) -> syn::Result<FieldOptions> {
    let mut options = FieldOptions::default();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("excel")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("ignore") {
                excel_ignore::apply(&mut options);
                return Ok(());
            }
            if excel_property::parse(&meta, &mut options)?
                || parse_date_time_format(&meta, &mut options)?
                || parse_number_format(&meta, &mut options)?
                || parse_field_column_width(&meta, &mut options)?
                || parse_field_head_style(&meta, &mut options, crate_path)?
                || parse_field_content_style(&meta, &mut options, crate_path)?
                || parse_field_head_font_style(&meta, &mut options, crate_path)?
                || parse_field_content_font_style(&meta, &mut options, crate_path)?
                || parse_content_loop_merge(&meta, &mut options, crate_path)?
            {
                return Ok(());
            }
            if parse_extension(&meta, &mut options.extensions, crate_path)? {
                return Ok(());
            }
            Err(meta.error("unsupported ExcelRow field option"))
        })?;
    }
    Ok(options)
}
