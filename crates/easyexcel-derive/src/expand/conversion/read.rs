//! 字段读取转换代码生成。
//!
//! T3.2 derive 原语字段直读快路径：对纯原语字段（i64/f64/String/bool/NaiveDate）
//! 无自定义 converter 时直接展开 `CellValue` 模式匹配，绕过
//! `ReadConverterContext` 装配（formula/display_value/decimal_value 查询）。

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Path, Type};

/// 识别可走快路径的原语字段类型。
///
/// 仅当字段类型为以下之一且无自定义 converter 时才返回 `Some`：
/// `i64`、`f64`、`String`、`bool`、`NaiveDate`。
fn classify_primitive(ty: &Type) -> Option<&'static str> {
    let path = match ty {
        Type::Path(type_path) => &type_path.path,
        _ => return None,
    };
    let segment = path.segments.last()?;
    match segment.ident.to_string().as_str() {
        "i64" => Some("i64"),
        "f64" => Some("f64"),
        "String" => Some("String"),
        "bool" => Some("bool"),
        "NaiveDate" => Some("NaiveDate"),
        _ => None,
    }
}

/// 为原语类型生成直接匹配 `CellValue` 的读取表达式，绕过 `FromExcelCell` trait
/// 调用开销和 `ReadConverterContext` 装配。
///
/// 匹配语义与 `from_into_impls.rs` 中的 `FromExcelCell` 实现完全一致。
fn primitive_cell_read(kind: &str, crate_path: &TokenStream) -> TokenStream {
    match kind {
        "i64" => quote! {{
            let cell = row.cell(column).unwrap_or(&#crate_path::CellValue::Empty);
            match cell {
                #crate_path::CellValue::Bool(v) => Ok(i64::from(u8::from(*v))),
                #crate_path::CellValue::Int(v) => Ok(*v),
                #crate_path::CellValue::Float(v) if v.fract() == 0.0 => Ok(*v as i64),
                #crate_path::CellValue::Decimal(v) if v == &v.with_scale(0) => {
                    // 与 parse_integer 一致：Decimal → String → parse，无需 ToPrimitive。
                    ::std::str::FromStr::from_str(
                        ::std::string::ToString::to_string(v).as_str(),
                    )
                    .map_err(|_| context.invalid(cell, "i64"))
                }
                #crate_path::CellValue::String(v) => {
                    ::std::str::FromStr::from_str(v.as_str())
                        .map_err(|_| context.invalid(cell, "i64"))
                }
                other => Err(context.invalid(other, "i64")),
            }?
        }},
        "f64" => quote! {{
            let cell = row.cell(column).unwrap_or(&#crate_path::CellValue::Empty);
            match cell {
                #crate_path::CellValue::Float(v) => Ok(*v),
                #crate_path::CellValue::Int(v) => Ok(*v as f64),
                #crate_path::CellValue::Bool(v) => Ok(if *v { 1.0 } else { 0.0 }),
                #crate_path::CellValue::Decimal(v) => {
                    ::std::str::FromStr::from_str(
                        ::std::string::ToString::to_string(v).as_str(),
                    )
                    .map_err(|_| context.invalid(cell, "f64"))
                }
                #crate_path::CellValue::String(v) => {
                    // Fast path: plain numeric string.
                    if let Ok(result) = ::std::str::FromStr::from_str(v.as_str()) {
                        Ok(result)
                    // Slow path: number_format-aware parsing (e.g. "123.5%" with "#.##%").
                    } else if let Some(pattern) = context.effective_number_format() {
                        #crate_path::util::number_utils::parse_decimal_as_f64(
                            v.as_str(), Some(pattern),
                        )
                        .map_err(|_| context.invalid(cell, "f64"))
                    } else {
                        Err(context.invalid(cell, "f64"))
                    }
                }
                other => Err(context.invalid(other, "f64")),
            }?
        }},
        // String 字段优先使用 display value（对应 Java StringNumberConverter
        // 通过 DataFormatter 保留格式化尾零，如 "24.20" 而非 "24.2"）；
        // 无 display value 时回退到 CellValue::as_text。
        "String" => quote! {
            if let ::core::option::Option::Some(dv) = row.display_value(column) {
                ::std::borrow::ToOwned::to_owned(dv)
            } else {
                row.cell(column).map_or_else(
                    ::std::string::String::new,
                    #crate_path::CellValue::as_text,
                )
            }
        },
        "bool" => quote! {{
            let cell = row.cell(column).unwrap_or(&#crate_path::CellValue::Empty);
            match cell {
                #crate_path::CellValue::Bool(v) => Ok(*v),
                #crate_path::CellValue::Int(v) => Ok(*v != 0),
                #crate_path::CellValue::Float(v) => Ok(*v != 0.0),
                #crate_path::CellValue::Decimal(v) => {
                    Ok(v != &#crate_path::BigDecimal::from(0))
                }
                #crate_path::CellValue::String(v)
                    if v.eq_ignore_ascii_case("true") || v == "1" =>
                {
                    Ok(true)
                }
                #crate_path::CellValue::String(v)
                    if v.eq_ignore_ascii_case("false") || v == "0" =>
                {
                    Ok(false)
                }
                other => Err(context.invalid(other, "bool")),
            }?
        }},
        // NaiveDate 的 FromExcelCell 需要 context（1904 窗口、日期格式），
        // 内联收益有限；仍走 FromExcelCell 但跳过 ReadConverterContext 装配。
        "NaiveDate" => quote! {
            <chrono::NaiveDate as #crate_path::FromExcelCell>::from_excel_cell(
                row.cell(column), &context,
            )?
        },
        _ => unreachable!("classify_primitive returned unknown kind"),
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 生成字段使用显式转换器或默认转换器读取的表达式。
pub(crate) fn field_read_conversion(
    crate_path: &TokenStream,
    ty: &Type,
    converter: Option<&Path>,
) -> TokenStream {
    converter.map_or_else(
        || {
            // T3.2 快路径：纯原语字段直接展开 CellValue 匹配，
            // 跳过 FromExcelCell trait 调用。
            if let Some(kind) = classify_primitive(ty) {
                return primitive_cell_read(kind, crate_path);
            }
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 生成支持运行时转换器注册表的字段读取表达式。
pub(crate) fn field_registered_read_conversion(
    crate_path: &TokenStream,
    ty: &Type,
    converter: Option<&Path>,
) -> TokenStream {
    converter.map_or_else(
        || {
            // T3.2 快路径：纯原语字段 + 无自定义 converter 时，
            // 检查注册表是否为标准只读集合。若是则直接展开 CellValue 匹配，
            // 绕过 ReadConverterContext::with_cell_metadata（formula/display_value/
            // decimal_value 三次 HashMap 查询）和 converter registry 查找。
            if let Some(kind) = classify_primitive(ty) {
                let fast = primitive_cell_read(kind, crate_path);
                return quote! {
                    if converters.is_standard_read_only() {
                        #fast
                    } else if let ::core::option::Option::Some(value) =
                        converters.convert_to_rust_data::<#ty>(
                            &#crate_path::ReadConverterContext::with_cell_metadata(
                                row.cell(column), row.formula(column),
                                row.display_value(column),
                                row.decimal_value(column), column, &context,
                            ),
                        )?
                    {
                        value
                    } else {
                        <#ty as #crate_path::FromExcelCell>::from_excel_cell(
                            row.cell(column), &context,
                        )?
                    }
                };
            }
            quote! {
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
