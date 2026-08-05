//! Java `com.alibaba.excel.annotation.write.style.ContentRowHeight` 的属性解析。

use syn::meta::ParseNestedMeta;

use crate::annotation::struct_options::StructOptions;
use crate::annotation::style_parser::parse_dimension;

/// 解析类型级内容行高。
pub(crate) fn parse(meta: &ParseNestedMeta<'_>, options: &mut StructOptions) -> syn::Result<bool> {
    if !meta.path.is_ident("content_row_height") {
        return Ok(false);
    }
    options.content_row_height = Some(parse_dimension(meta)?);
    Ok(true)
}
