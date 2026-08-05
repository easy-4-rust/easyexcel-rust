//! 单字段 `ExcelRow` 读写方法体令牌生成。

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path, Type};

use crate::expand::conversion::{
    field_original_write_conversion, field_read_conversion, field_registered_read_conversion,
    field_registered_write_cell_data_conversion, field_registered_write_conversion,
    field_write_conversion,
};

/// 一个字段参与全部 `ExcelRow` 方法所需的令牌。
pub(super) struct FieldTokens {
    pub(super) reader: TokenStream,
    pub(super) registered_reader: TokenStream,
    pub(super) writer: TokenStream,
    pub(super) original_writer: TokenStream,
    pub(super) selected_original_writer: TokenStream,
    pub(super) registered_writer: TokenStream,
    pub(super) registered_write_cell: TokenStream,
    pub(super) selected_registered_write_cell: TokenStream,
}

/// 生成一个字段的读取、写入及转换器注册表分支。
pub(super) fn build_field_tokens(
    crate_path: &TokenStream,
    ident: &Ident,
    ty: &Type,
    converter: Option<&Path>,
    schema_position: usize,
) -> FieldTokens {
    let position = syn::Index::from(schema_position);
    let read = field_read_conversion(crate_path, ty, converter);
    let registered_read = field_registered_read_conversion(crate_path, ty, converter);
    let write = field_write_conversion(crate_path, ty, ident, converter);
    let original_write = field_original_write_conversion(crate_path, ty, ident, converter);
    let registered_write = field_registered_write_conversion(crate_path, ty, ident, converter);
    let registered_cell =
        field_registered_write_cell_data_conversion(crate_path, ty, ident, converter);
    let reader = quote! {
        #ident: {
            let column = &Self::schema()[#position];
            let context = row.convert_context(column);
            #read
        }
    };
    let registered_reader = quote! {
        #ident: {
            let column = &Self::schema()[#position];
            let context = row.convert_context(column);
            #registered_read
        }
    };
    let cell_value = quote!(#crate_path::CellValue);
    let write_cell_data = quote!(#crate_path::WriteCellData);
    let writer = contextual_value(crate_path, &position, &write, &cell_value);
    let original_writer = contextual_value(crate_path, &position, &original_write, &cell_value);
    let selected_original_writer = quote! {
        if selected_schema_indexes.is_none_or(|selected| selected.contains(&#position)) {
            #original_writer
        } else {
            #crate_path::CellValue::Empty
        }
    };
    let registered_writer = contextual_value(crate_path, &position, &registered_write, &cell_value);
    let registered_write_cell =
        contextual_value(crate_path, &position, &registered_cell, &write_cell_data);
    let selected_registered_write_cell = quote! {
        if selected_schema_indexes.is_none_or(|selected| selected.contains(&#position)) {
            #registered_write_cell
        } else {
            #crate_path::WriteCellData::new(#crate_path::CellValue::Empty)
        }
    };
    FieldTokens {
        reader,
        registered_reader,
        writer,
        original_writer,
        selected_original_writer,
        registered_writer,
        registered_write_cell,
        selected_registered_write_cell,
    }
}

fn contextual_value(
    crate_path: &TokenStream,
    position: &syn::Index,
    conversion: &TokenStream,
    output: &TokenStream,
) -> TokenStream {
    quote! {{
        let column = &Self::schema()[#position];
        let context = #crate_path::ConvertContext {
            sheet_name: ::std::string::String::new(),
            row_index: 0,
            column_index: column.index,
            field: column.field,
            format: column.format,
            date_time_format: column.date_time_format,
            number_format: column.number_format,
            use_1904_windowing: column.use_1904_windowing.unwrap_or(false),
        };
        (|| -> #crate_path::Result<#output> { #conversion })()
            .map_err(|error| context.write_error(error))?
    }}
}
