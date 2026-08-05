//! 字段读取转换代码生成。

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Path, Type};

/// 生成字段使用显式转换器或默认转换器读取的表达式。
pub(crate) fn field_read_conversion(
    crate_path: &TokenStream,
    ty: &Type,
    converter: Option<&Path>,
) -> TokenStream {
    converter.map_or_else(
        || {
            quote! {
                <#ty as #crate_path::FromExcelCell>::from_excel_cell(row.cell(column), &context)?
            }
        },
        |converter| {
            quote! {
                #crate_path::Converter::<#ty>::convert_to_rust_data(
                    &<#converter as ::core::default::Default>::default(),
                    &#crate_path::ReadConverterContext::with_cell_metadata(
                        row.cell(column), row.formula(column), row.display_value(column),
                        row.decimal_value(column), column, &context,
                    ),
                )?
            }
        },
    )
}

/// 生成支持运行时转换器注册表的字段读取表达式。
pub(crate) fn field_registered_read_conversion(
    crate_path: &TokenStream,
    ty: &Type,
    converter: Option<&Path>,
) -> TokenStream {
    converter.map_or_else(
        || quote! {
            if let ::core::option::Option::Some(value) = converters.convert_to_rust_data::<#ty>(
                &#crate_path::ReadConverterContext::with_cell_metadata(
                    row.cell(column), row.formula(column), row.display_value(column),
                    row.decimal_value(column), column, &context,
                ),
            )? {
                value
            } else {
                <#ty as #crate_path::FromExcelCell>::from_excel_cell(row.cell(column), &context)?
            }
        },
        |converter| quote! {
            #crate_path::Converter::<#ty>::convert_to_rust_data(
                &<#converter as ::core::default::Default>::default(),
                &#crate_path::ReadConverterContext::with_cell_metadata(
                    row.cell(column), row.formula(column), row.display_value(column),
                    row.decimal_value(column), column, &context,
                ),
            )?
        },
    )
}
