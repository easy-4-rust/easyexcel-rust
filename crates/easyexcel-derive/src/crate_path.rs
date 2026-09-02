//! 解析生成代码中使用的 `easyexcel` crate 路径。

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回支持依赖重命名的 `EasyExcel` 公共门面路径。
pub(crate) fn easyexcel_path() -> TokenStream {
    let found = ["easyexcel", "easyexcel-core"]
        .into_iter()
        .find_map(|package| crate_name(package).ok());
    resolve_easyexcel_path(found)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将 Cargo 包解析结果转换为可生成的 Rust 路径。
pub(crate) fn resolve_easyexcel_path(found: Option<FoundCrate>) -> TokenStream {
    found.map_or_else(
        || {
            let fallback: Path = syn::parse_quote!(::easyexcel);
            quote!(#fallback)
        },
        found_crate_path,
    )
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn found_crate_path(found: FoundCrate) -> TokenStream {
    match found {
        // 包内使用（lib 单元测试 / examples / 集成测试同属 easyexcel
        // package）也统一生成 `::easyexcel`：easyexcel lib.rs 顶部声明了
        // `extern crate self as easyexcel;`，而 examples 的 `crate::` 根是
        // example 自身（没有 util 模块），不能生成 `crate`。
        FoundCrate::Itself => {
            let ident = format_ident!("easyexcel");
            quote!(::#ident)
        }
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name.replace('-', "_"));
            quote!(::#ident)
        }
    }
}
