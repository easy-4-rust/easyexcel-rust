//! 汇总字段令牌并生成最终 `ExcelRow` trait 实现。

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Ident};

use crate::annotation::StructOptions;

use super::field_expansion::FieldExpansion;
use super::metadata::write_metadata_tokens;

/// 结构体全部有效字段的展开片段。
#[derive(Default)]
pub(super) struct ExpansionParts {
    pub(super) columns: Vec<TokenStream>,
    readers: Vec<TokenStream>,
    registered_readers: Vec<TokenStream>,
    writers: Vec<TokenStream>,
    original_writers: Vec<TokenStream>,
    selected_original_writers: Vec<TokenStream>,
    registered_writers: Vec<TokenStream>,
    registered_write_cell_data: Vec<TokenStream>,
    selected_registered_write_cell_data: Vec<TokenStream>,
}

impl ExpansionParts {
    /// 追加单字段展开结果；忽略字段只参与读取默认值初始化。
    pub(super) fn push(&mut self, field: FieldExpansion) {
        self.readers.push(field.readers);
        self.registered_readers.push(field.registered_readers);
        if let Some(value) = field.column {
            self.columns.push(value);
        }
        if let Some(value) = field.writers {
            self.writers.push(value);
        }
        if let Some(value) = field.original_writers {
            self.original_writers.push(value);
        }
        if let Some(value) = field.selected_original_writers {
            self.selected_original_writers.push(value);
        }
        if let Some(value) = field.registered_writers {
            self.registered_writers.push(value);
        }
        if let Some(value) = field.registered_write_cell_data {
            self.registered_write_cell_data.push(value);
        }
        if let Some(value) = field.selected_registered_write_cell_data {
            self.selected_registered_write_cell_data.push(value);
        }
    }
}

/// 生成静态 schema、类型级元数据与双向行转换实现。
pub(super) fn render_trait_impl(
    name: &Ident,
    generics: &Generics,
    crate_path: &TokenStream,
    options: &StructOptions,
    parts: ExpansionParts,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let write_metadata = write_metadata_tokens(crate_path, options);
    let ExpansionParts {
        columns,
        readers,
        registered_readers,
        writers,
        original_writers,
        selected_original_writers,
        registered_writers,
        registered_write_cell_data,
        selected_registered_write_cell_data,
    } = parts;
    quote! {
        impl #impl_generics #crate_path::ExcelRow for #name #ty_generics #where_clause {
            fn schema() -> &'static [#crate_path::ExcelColumn] {
                const COLUMNS: &[#crate_path::ExcelColumn] = &[#(#columns),*];
                COLUMNS
            }
            fn write_metadata() -> &'static #crate_path::ExcelWriteMetadata {
                const METADATA: #crate_path::ExcelWriteMetadata = #write_metadata;
                &METADATA
            }
            fn from_row(row: &#crate_path::RowData) -> #crate_path::Result<Self> {
                Ok(Self { #(#readers),* })
            }
            fn from_row_with_converters(row: &#crate_path::RowData, converters: &#crate_path::ConverterRegistry) -> #crate_path::Result<Self> {
                Ok(Self { #(#registered_readers),* })
            }
            fn to_row(&self) -> #crate_path::Result<::std::vec::Vec<#crate_path::CellValue>> {
                Ok(::std::vec![#(#writers),*])
            }
            fn to_row_with_converters(&self, converters: &#crate_path::ConverterRegistry) -> #crate_path::Result<::std::vec::Vec<#crate_path::CellValue>> {
                Ok(::std::vec![#(#registered_writers),*])
            }
            fn to_excel_write_row(&self, converters: &#crate_path::ConverterRegistry) -> #crate_path::Result<(::std::vec::Vec<#crate_path::CellValue>, ::std::vec::Vec<#crate_path::WriteCellData>)> {
                Ok((::std::vec![#(#original_writers),*], ::std::vec![#(#registered_write_cell_data),*]))
            }
            fn to_excel_write_row_selected(&self, converters: &#crate_path::ConverterRegistry, selected_schema_indexes: ::core::option::Option<&[usize]>) -> #crate_path::Result<(::std::vec::Vec<#crate_path::CellValue>, ::std::vec::Vec<#crate_path::WriteCellData>)> {
                Ok((::std::vec![#(#selected_original_writers),*], ::std::vec![#(#selected_registered_write_cell_data),*]))
            }
        }
    }
}
