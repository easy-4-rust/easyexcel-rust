//! 列宽和行高属性解析。

use syn::meta::ParseNestedMeta;

use crate::annotation::integer::{SignedInteger, parse_signed_i32};

/// 解析列宽或行高，保留 Java 的 `-1` 默认值哨兵。
pub(crate) fn parse_dimension(meta: &ParseNestedMeta<'_>) -> syn::Result<SignedInteger> {
    let value = parse_signed_i32(meta)?;
    if value.value() < -1 || value.value() > i32::from(u16::MAX) {
        return Err(syn::Error::new(
            value.span(),
            "dimension must be -1 or fit in u16",
        ));
    }
    Ok(value)
}
