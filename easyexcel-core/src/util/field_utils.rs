#![allow(dead_code)]

//! Mirrors Java com.alibaba.excel.util.FieldUtils.
//!
//! Java uses Spring's `ReflectionUtils` / Apache Commons `FieldUtils` to
//! resolve fields (and to strip CGLIB `$$EnhancerByCGLIB$$` synthetic
//! suffixes). Rust has no runtime reflection, so both helpers are
//! returned as no-op anchors.

/// Mirrors `com.alibaba.excel.util.FieldUtils#resolveCglibFieldName`.
///
/// Java strips the `$$EnhancerByCGLIB$$<hash>` suffix added by the CGLIB
/// proxy. Rust has no equivalent bytecode rewriting, so the input is
/// returned verbatim.
#[must_use]
pub fn resolve_cglib_field_name(name: &str) -> &str {
    name
}

/// Mirrors `com.alibaba.excel.util.FieldUtils#getField`.
///
/// Returns `None` because Rust field access is resolved at compile time
/// via `derive(ExcelRow)` instead of runtime reflection.
#[must_use]
pub fn get_field(_class_name: &str, _field_name: &str) -> Option<()> {
    None
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn resolve_cglib_field_name_returns_verbatim() {
        // 对应 Java：Rust 无 CGLIB 代理，原样返回
        assert_eq!(resolve_cglib_field_name("name"), "name");
        assert_eq!(
            resolve_cglib_field_name("name$$EnhancerByCGLIB$$abc"),
            "name$$EnhancerByCGLIB$$abc"
        );
    }

    #[test]
    fn get_field_returns_none() {
        // 对应 Java：字段由 derive(ExcelRow) 编译期解析
        assert_eq!(get_field("Model", "name"), None);
    }
}
