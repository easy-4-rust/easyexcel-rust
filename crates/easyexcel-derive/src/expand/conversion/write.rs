//! 字段写入转换代码生成。

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path, Type};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 生成字段写入单元格值的表达式。
pub(crate) fn field_write_conversion(
    crate_path: &TokenStream,
    ty: &Type,
    ident: &Ident,
    converter: Option<&Path>,
) -> TokenStream {
    converter.map_or_else(
        || quote!(#crate_path::IntoExcelCell::to_excel_cell(&self.#ident, &context)),
        |converter| {
            quote! {
                #crate_path::IntoExcelCell::to_excel_cell(
                    &#crate_path::Converter::<#ty>::convert_to_excel_data(
                        &<#converter as ::core::default::Default>::default(),
                        &#crate_path::WriteConverterContext::new(&self.#ident, column, &context),
                    )?, &context,
                )
            }
        },
    )
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 生成支持运行时转换器注册表的字段写入表达式。
pub(crate) fn field_registered_write_conversion(
    crate_path: &TokenStream,
    ty: &Type,
    ident: &Ident,
    converter: Option<&Path>,
) -> TokenStream {
    let value_is_null = option_null_expression(ty, ident);
    converter.map_or_else(
        || quote! {
            if let ::core::option::Option::Some(value) = converters.convert_to_excel_data_with_null_state::<#ty>(
                &self.#ident, column, &context, #value_is_null,
            )? {
                #crate_path::IntoExcelCell::to_excel_cell(&value, &context)
            } else {
                #crate_path::IntoExcelCell::to_excel_cell(&self.#ident, &context)
            }
        },
        |converter| quote! {
            #crate_path::IntoExcelCell::to_excel_cell(
                &#crate_path::Converter::<#ty>::convert_to_excel_data(
                    &<#converter as ::core::default::Default>::default(),
                    &#crate_path::WriteConverterContext::new(&self.#ident, column, &context),
                )?, &context,
            )
        },
    )
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 生成保留写入元数据的 `WriteCellData` 转换表达式。
pub(crate) fn field_registered_write_cell_data_conversion(
    crate_path: &TokenStream,
    ty: &Type,
    ident: &Ident,
    converter: Option<&Path>,
) -> TokenStream {
    let value_is_null = option_null_expression(ty, ident);
    converter.map_or_else(
        || quote! {
            if let ::core::option::Option::Some(value) = converters.convert_to_excel_data_with_null_state::<#ty>(
                &self.#ident, column, &context, #value_is_null,
            )? {
                Ok(value)
            } else {
                let cell = #crate_path::IntoExcelCell::to_excel_cell(&self.#ident, &context)?;
                Ok(#crate_path::WriteCellData::new(cell))
            }
        },
        |converter| quote! {
            #crate_path::Converter::<#ty>::convert_to_excel_data(
                &<#converter as ::core::default::Default>::default(),
                &#crate_path::WriteConverterContext::new(&self.#ident, column, &context),
            )
        },
    )
}

fn option_null_expression(ty: &Type, ident: &Ident) -> TokenStream {
    let is_option = matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Option"));
    if is_option {
        quote!(self.#ident.is_none())
    } else {
        quote!(false)
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 生成写处理器使用的原始值快照表达式。
pub(crate) fn field_original_write_conversion(
    crate_path: &TokenStream,
    ty: &Type,
    ident: &Ident,
    converter: Option<&Path>,
) -> TokenStream {
    if converter.is_none() || is_side_effect_free_original_type(ty) {
        quote!(#crate_path::IntoExcelCell::to_excel_cell(&self.#ident, &context))
    } else {
        quote!(Ok(#crate_path::CellValue::Empty))
    }
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn is_side_effect_free_original_type(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    matches!(
        segment.ident.to_string().as_str(),
        "String"
            | "bool"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "BigDecimal"
            | "BigInt"
            | "NaiveDate"
            | "NaiveDateTime"
    )
}
