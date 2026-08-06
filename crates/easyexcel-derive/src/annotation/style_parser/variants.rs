//! Java 样式枚举名称与 Rust 枚举变体的映射。

use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) const HORIZONTAL_ALIGNMENT_VARIANTS: &[(&str, &str)] = &[
    ("general", "General"),
    ("left", "Left"),
    ("center", "Center"),
    ("right", "Right"),
    ("fill", "Fill"),
    ("justify", "Justify"),
    ("center_across", "CenterAcross"),
    ("distributed", "Distributed"),
];

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) const VERTICAL_ALIGNMENT_VARIANTS: &[(&str, &str)] = &[
    ("top", "Top"),
    ("center", "Center"),
    ("bottom", "Bottom"),
    ("justify", "Justify"),
    ("distributed", "Distributed"),
];

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) const BORDER_STYLE_VARIANTS: &[(&str, &str)] = &[
    ("none", "None"),
    ("thin", "Thin"),
    ("medium", "Medium"),
    ("dashed", "Dashed"),
    ("dotted", "Dotted"),
    ("thick", "Thick"),
    ("double", "Double"),
    ("hair", "Hair"),
    ("medium_dashed", "MediumDashed"),
    ("dash_dot", "DashDot"),
    ("medium_dash_dot", "MediumDashDot"),
    ("dash_dot_dot", "DashDotDot"),
    ("medium_dash_dot_dot", "MediumDashDotDot"),
    ("slant_dash_dot", "SlantDashDot"),
];

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) const FILL_PATTERN_VARIANTS: &[(&str, &str)] = &[
    ("none", "None"),
    ("solid", "Solid"),
    ("medium_gray", "MediumGray"),
    ("dark_gray", "DarkGray"),
    ("light_gray", "LightGray"),
    ("dark_horizontal", "DarkHorizontal"),
    ("dark_vertical", "DarkVertical"),
    ("dark_down", "DarkDown"),
    ("dark_up", "DarkUp"),
    ("dark_grid", "DarkGrid"),
    ("dark_trellis", "DarkTrellis"),
    ("light_horizontal", "LightHorizontal"),
    ("light_vertical", "LightVertical"),
    ("light_down", "LightDown"),
    ("light_up", "LightUp"),
    ("light_grid", "LightGrid"),
    ("light_trellis", "LightTrellis"),
    ("gray_125", "Gray125"),
    ("gray_0625", "Gray0625"),
];

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将 Java 风格舍入模式名称转换为 `EasyExcel` 枚举表达式。
pub(crate) fn number_rounding_mode_tokens(
    value: &LitStr,
    crate_path: &TokenStream,
) -> syn::Result<TokenStream> {
    let variant = match value
        .value()
        .replace(['_', '-'], "")
        .to_ascii_lowercase()
        .as_str()
    {
        "up" => quote!(Up),
        "down" => quote!(Down),
        "ceiling" => quote!(Ceiling),
        "floor" => quote!(Floor),
        "halfup" => quote!(HalfUp),
        "halfdown" => quote!(HalfDown),
        "halfeven" => quote!(HalfEven),
        "unnecessary" => quote!(Unnecessary),
        _ => {
            return Err(syn::Error::new_spanned(
                value,
                "rounding_mode must be UP, DOWN, CEILING, FLOOR, HALF_UP, HALF_DOWN, HALF_EVEN, or UNNECESSARY",
            ));
        }
    };
    Ok(quote!(#crate_path::NumberRoundingMode::#variant))
}
