//! 解析生成代码中使用的 `easyexcel` crate 路径。

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

/// 返回支持依赖重命名的 `EasyExcel` 公共门面路径。
pub(crate) fn easyexcel_path() -> TokenStream {
    let found = ["easyexcel", "easyexcel-core"]
        .into_iter()
        .find_map(|package| crate_name(package).ok());
    resolve_easyexcel_path(found)
}

/// 将 Cargo 包解析结果转换为可生成的 Rust 路径。
pub(crate) fn resolve_easyexcel_path(found: Option<FoundCrate>) -> TokenStream {
    found.map_or_else(
        || {
            let fallback: Path = syn::parse_quote!(::easyexcel);
            quote!(#fallback)
        },
        found_crate_path,
    )
}

pub(crate) fn found_crate_path(found: FoundCrate) -> TokenStream {
    match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name.replace('-', "_"));
            quote!(::#ident)
        }
    }
}
