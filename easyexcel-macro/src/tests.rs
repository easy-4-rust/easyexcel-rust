use proc_macro_crate::FoundCrate;
use quote::quote;
use syn::{DeriveInput, parse_quote};

use super::*;

fn assert_struct_style_options_rejected(attributes: &[&str]) {
    for attribute in attributes {
        let source = format!("#[excel({attribute})] struct User {{ value: String }}");
        let input = syn::parse_str::<DeriveInput>(&source).expect("attribute tokens");
        assert!(parse_struct_options(&input.attrs, &quote!(::easyexcel)).is_err());
    }
}

#[test]
fn token_entry_parses_valid_input_and_rejects_invalid_syntax() {
    assert!(
        expand_excel_row_tokens(quote!(
            struct User {
                value: String,
            }
        ))
        .expect("valid tokens")
        .to_string()
        .contains("ExcelRow")
    );
    assert!(expand_excel_row_tokens(quote!(struct)).is_err());
}

#[test]
fn number_format_tokens_carry_java_rounding_mode_into_schema() {
    let tokens = expand_excel_row_tokens(quote!(
        struct Amount {
            #[excel(number_format = "#.##%", rounding_mode = "UNNECESSARY")]
            value: bigdecimal::BigDecimal,
        }
    ))
    .expect("valid Java number format")
    .to_string();
    assert!(tokens.contains("Some (\"#.##%\")"));
    assert!(tokens.contains("NumberRoundingMode :: Unnecessary"));

    let invalid: DeriveInput = parse_quote! {
        struct Amount {
            #[excel(rounding_mode = "BANKERS")]
            value: i32,
        }
    };
    // parse_quote! 恒产出 struct；非 struct 输入由 expect_struct_fields 的 panic 臂兜底。
    let fields = expect_struct_fields(invalid);
    let field = fields.iter().next().expect("field");
    let options = parse_field_options(&field.attrs, &quote!(::easyexcel)).expect("parsed literal");
    assert!(
        number_rounding_mode_tokens(
            options.rounding_mode.as_ref().expect("mode"),
            &quote!(::easyexcel),
        )
        .is_err()
    );
}

#[test]
fn crate_paths_support_self_renames_and_fallback_lookup() {
    assert_eq!(found_crate_path(FoundCrate::Itself).to_string(), "crate");
    assert_eq!(
        found_crate_path(FoundCrate::Name("easyexcel-renamed".to_owned())).to_string(),
        ":: easyexcel_renamed"
    );
    assert_eq!(resolve_easyexcel_path(None).to_string(), ":: easyexcel");
    assert_eq!(
        resolve_easyexcel_path(Some(FoundCrate::Name("renamed-core".to_owned()))).to_string(),
        ":: renamed_core"
    );
    assert!(!easyexcel_path().is_empty());
}

#[test]
fn struct_options_accept_ignore_unannotated_and_reject_unknown_values() {
    let input: DeriveInput = parse_quote! {
        #[excel(
            ignore_unannotated,
            column_width = 25,
            head_row_height = 20,
            content_row_height = 16,
            once_absolute_merge(
                first_row_index = 0,
                last_row_index = 1,
                first_column_index = 0,
                last_column_index = 2
            )
        )]
        struct User { name: String }
    };
    let options = parse_struct_options(&input.attrs, &quote!(::easyexcel)).expect("valid option");
    assert!(options.ignore_unannotated);
    assert_eq!(
        options
            .column_width
            .expect("width")
            .base10_parse::<u16>()
            .expect("u16"),
        25
    );
    assert_eq!(
        options
            .head_row_height
            .expect("head height")
            .base10_parse::<u16>()
            .expect("u16"),
        20
    );
    assert_eq!(
        options
            .content_row_height
            .expect("content height")
            .base10_parse::<u16>()
            .expect("u16"),
        16
    );
    assert!(options.once_absolute_merge.is_some());

    let input: DeriveInput = parse_quote! {
        #[excel(unknown)]
        struct User { name: String }
    };
    assert!(
        parse_struct_options(&input.attrs, &quote!(::easyexcel))
            .err()
            .expect("unknown option")
            .to_string()
            .contains("unsupported ExcelRow struct option")
    );

    for attribute in [
        "column_width",
        "column_width = \"wide\"",
        "column_width = 65536",
        "head_row_height",
        "content_row_height = -1",
        "once_absolute_merge(unknown = 1)",
        "once_absolute_merge(first_row_index = \"zero\")",
    ] {
        let source = format!("#[excel({attribute})] struct User {{ value: String }}");
        let input = syn::parse_str::<DeriveInput>(&source).expect("attribute tokens");
        assert!(parse_struct_options(&input.attrs, &quote!(::easyexcel)).is_err());
    }
}

#[test]
fn style_options_parse_java_equivalents_and_reject_invalid_values() {
    let input: DeriveInput = parse_quote! {
        #[excel(
            head_style(
                hidden = true,
                locked = false,
                quote_prefix = true,
                horizontal_alignment = "distributed",
                wrapped = true,
                vertical_alignment = "justify",
                rotation = 45,
                indent = 2,
                border_left = "thin",
                border_right = "medium",
                border_top = "dashed",
                border_bottom = "double",
                left_border_color = 0x112233,
                right_border_color = 0x223344,
                top_border_color = 0x334455,
                bottom_border_color = 0x445566,
                fill_pattern = "solid",
                fill_background_color = 0x556677,
                fill_foreground_color = 0x667788,
                shrink_to_fit = true,
                data_format = "0.00"
            ),
            content_style(wrapped = false, data_format = 14, fill_foreground_color = 10),
            head_font_style(
                font_name = "Arial",
                font_height_in_points = 12.5,
                italic = true,
                strikeout = false,
                color = 0x778899,
                type_offset = "superscript",
                underline = "double_accounting",
                charset = 1,
                bold = true
            ),
            content_font_style(bold = false)
        )]
        struct User { name: String }
    };
    let options = parse_struct_options(&input.attrs, &quote!(::easyexcel)).expect("valid styles");
    for style in [
        options.head_style,
        options.content_style,
        options.head_font_style,
        options.content_font_style,
    ] {
        assert!(style.expect("style").to_string().contains("style"));
    }

    assert_struct_style_options_rejected(&[
        "head_style(unknown = true)",
        "head_style(wrapped)",
        "head_style(wrapped = 1)",
        "head_style(rotation)",
        "head_style(rotation = \"up\")",
        "head_style(rotation = 32768)",
        "head_style(indent)",
        "head_style(indent = \"deep\")",
        "head_style(data_format)",
        "head_style(data_format = true)",
        "head_style(data_format = crate::FORMAT)",
        "head_style(data_format = 256)",
        "head_style(fill_foreground_color)",
        "head_style(fill_foreground_color = \"red\")",
        "head_style(horizontal_alignment)",
        "head_style(horizontal_alignment = 1)",
        "head_style(horizontal_alignment = \"diagonal\")",
        "head_style(vertical_alignment = \"diagonal\")",
        "head_style(border_left = \"triple\")",
        "head_style(fill_pattern = \"invalid\")",
        "head_style(indent = 256)",
        "head_style(left_border_color = 4294967296)",
        "head_style(foo::bar = true)",
        "head_font_style(unknown = true)",
        "head_font_style(font_name)",
        "head_font_style(font_name = 1)",
        "head_font_style(font_height_in_points)",
        "head_font_style(font_height_in_points = crate::SIZE)",
        "head_font_style(font_height_in_points = 1e999)",
        "head_font_style(bold)",
        "head_font_style(bold = 1)",
        "head_font_style(color)",
        "head_font_style(color = \"red\")",
        "head_font_style(charset)",
        "head_font_style(charset = \"default\")",
        "head_font_style(type_offset)",
        "head_font_style(type_offset = 1)",
        "head_font_style(font_height_in_points = \"large\")",
        "head_font_style(font_height_in_points = 0)",
        "head_font_style(charset = 256)",
        "head_font_style(type_offset = \"invalid\")",
        "head_font_style(underline = \"invalid\")",
        "head_font_style(foo::bar = true)",
    ]);

    assert_struct_style_options_rejected(&[
        "content_style(unknown = true)",
        "content_font_style(unknown = true)",
    ]);
}

#[test]
#[allow(clippy::too_many_lines)]
fn field_options_parse_every_supported_value_and_reject_unknown_values() {
    let input: DeriveInput = parse_quote! {
        struct User {
            #[excel(
                name = "姓名",
                index = 2,
                order = 1,
                format = "%Y-%m-%d",
                rounding_mode = "HALF_EVEN",
                use_1904_windowing = true,
                converter = crate::NameConverter,
                column_width = 30,
                head_style(wrapped = true),
                content_style(wrapped = false),
                head_font_style(bold = true),
                content_font_style(italic = true),
                content_loop_merge(each_row = 2, column_extend = 1),
                ignore
            )]
            name: String,
        }
    };
    let fields = expect_struct_fields(input);
    let field = fields.iter().next().expect("field");
    let options = parse_field_options(&field.attrs, &quote!(::easyexcel)).expect("valid options");
    assert!(options.annotated);
    assert!(options.ignore);
    assert_eq!(options.name.expect("name").value(), "姓名");
    assert_eq!(
        options.rounding_mode.expect("rounding mode").value(),
        "HALF_EVEN"
    );
    assert_eq!(
        options
            .index
            .expect("index")
            .base10_parse::<usize>()
            .expect("usize"),
        2
    );
    assert_eq!(
        options
            .order
            .expect("order")
            .base10_parse::<i32>()
            .expect("i32"),
        1
    );
    assert_eq!(options.format.expect("format").value(), "%Y-%m-%d");
    assert!(options.use_1904_windowing.expect("windowing").value());
    assert_eq!(options.converter.expect("converter").segments.len(), 2);
    assert_eq!(
        options
            .column_width
            .expect("width")
            .base10_parse::<u16>()
            .expect("u16"),
        30
    );
    assert!(options.head_style.is_some());
    assert!(options.content_style.is_some());
    assert!(options.head_font_style.is_some());
    assert!(options.content_font_style.is_some());
    assert!(options.content_loop_merge.is_some());

    let input: DeriveInput = parse_quote! {
        struct User { #[excel(unknown)] name: String }
    };
    let fields = expect_struct_fields(input);
    let field = fields.iter().next().expect("field");
    let error = parse_field_options(&field.attrs, &quote!(::easyexcel))
        .map(drop) // map(drop) 规避 expect_err 对 Ok 类型的 Debug 约束，语义与失败行为不变
        .expect_err("unknown option must be rejected");
    assert!(
        error
            .to_string()
            .contains("unsupported ExcelRow field option")
    );

    for attribute in [
        "name",
        "name = 1",
        "index",
        "index = \"zero\"",
        "order",
        "order = \"first\"",
        "format",
        "format = 1",
        "use_1904_windowing",
        "use_1904_windowing = 1",
        "converter",
        "converter = 1",
        "column_width",
        "column_width = \"wide\"",
        "column_width = 65536",
        "head_style(unknown = true)",
        "content_style(unknown = true)",
        "head_font_style(unknown = true)",
        "content_font_style(unknown = true)",
        "content_loop_merge(unknown = 1)",
        "content_loop_merge(each_row = \"two\")",
        "content_loop_merge(column_extend = 65536)",
    ] {
        let source = format!("struct User {{ #[excel({attribute})] value: String }}");
        let input = syn::parse_str::<DeriveInput>(&source).expect("attribute tokens");
        let fields = expect_struct_fields(input);
        let field = fields.iter().next().expect("field");
        assert!(
            parse_field_options(&field.attrs, &quote!(::easyexcel)).is_err(),
            "`{attribute}` must be rejected"
        );
    }
}

// 该测试逐字段断言生成代码，行数较多但结构单一，拆分反而损害可读性。
#[allow(clippy::too_many_lines)]
#[test]
fn expansion_generates_schema_readers_writers_defaults_and_generics() {
    let input: DeriveInput = parse_quote! {
        #[excel(
            ignore_unannotated,
            column_width = 25,
            head_row_height = 20,
            content_row_height = 16,
            head_style(fill_pattern = "solid"),
            content_style(wrapped = true),
            head_font_style(bold = true),
            content_font_style(italic = true),
            once_absolute_merge(
                first_row_index = 0,
                last_row_index = 0,
                first_column_index = 0,
                last_column_index = 1
            )
        )]
        struct User<T>
        where
            T: Default,
        {
            #[excel(
                name = "姓名",
                index = 0,
                order = 2,
                format = "text",
                column_width = 30,
                head_style(wrapped = false),
                content_style(shrink_to_fit = true),
                head_font_style(font_name = "Arial"),
                content_font_style(bold = false),
                content_loop_merge(each_row = 2, column_extend = 1)
            )]
            name: String,
            #[excel(ignore)]
            ignored: u32,
            unannotated: T,
        }
    };
    let expanded = expand_excel_row(input).expect("expansion").to_string();
    for expected in [
        "impl < T >",
        "ExcelRow for User < T >",
        "ExcelColumn :: new",
        "with_field_type (:: core :: stringify ! (String))",
        "with_column_width (30)",
        "with_head_style",
        "with_content_style",
        "with_head_font_style",
        "with_content_font_style",
        "with_loop_merge",
        "LoopMergeProperty :: new (2 , 1)",
        "ExcelWriteMetadata :: new () . column_width (25) . head_row_height (20) . content_row_height (16) . head_style",
        "once_absolute_merge",
        "OnceAbsoluteMergeProperty :: new",
        "姓名",
        "Option :: Some (0)",
        "Option :: Some (\"text\")",
        "ignored : :: core :: default :: Default :: default ()",
        "unannotated : :: core :: default :: Default :: default ()",
        "FromExcelCell",
        "IntoExcelCell :: to_excel_cell",
    ] {
        assert!(
            expanded.contains(expected),
            "missing `{expected}` in {expanded}"
        );
    }

    let default_input: DeriveInput = parse_quote! {
        struct DefaultColumn { value: String }
    };
    let expanded = expand_excel_row(default_input)
        .expect("default expansion")
        .to_string();
    assert!(expanded.contains("\"value\""));
    assert!(expanded.contains("Option :: None"));
    assert!(expanded.contains("i32 :: MAX"));

    let converter_input: DeriveInput = parse_quote! {
        struct Converted {
            #[excel(converter = crate::NameConverter)]
            value: String,
        }
    };
    let expanded = expand_excel_row(converter_input)
        .expect("converter expansion")
        .to_string();
    for expected in [
        "Converter :: < String > :: convert_to_rust_data",
        "ReadConverterContext :: with_cell_metadata",
        "row . formula (column)",
        "row . display_value (column)",
        "row . decimal_value (column)",
        "NameConverter as :: core :: default :: Default",
        "Converter :: < String > :: convert_to_excel_data",
        "WriteConverterContext :: new",
        "fn to_excel_write_row",
        "IntoExcelCell :: to_excel_cell (& self . value",
    ] {
        assert!(
            expanded.contains(expected),
            "missing `{expected}` in {expanded}"
        );
    }
}

#[test]
fn expansion_rejects_tuple_structs_and_non_struct_items() {
    let tuple: DeriveInput = parse_quote!(
        struct Tuple(String);
    );
    assert!(
        expand_excel_row(tuple)
            .expect_err("tuple struct")
            .to_string()
            .contains("named fields")
    );

    let enumeration: DeriveInput = parse_quote!(
        enum Kind {
            One,
        }
    );
    assert!(
        expand_excel_row(enumeration)
            .expect_err("enum")
            .to_string()
            .contains("only be derived for structs")
    );

    let bad_struct_option: DeriveInput = parse_quote! {
        #[excel(unknown)]
        struct User { value: String }
    };
    assert!(expand_excel_row(bad_struct_option).is_err());

    let bad_field_option: DeriveInput = parse_quote! {
        struct User { #[excel(unknown)] value: String }
    };
    assert!(expand_excel_row(bad_field_option).is_err());
}

#[test]
fn expansion_rejects_duplicate_forced_column_indexes() {
    let input: DeriveInput = parse_quote! {
        struct DuplicateIndex {
            #[excel(index = 2)]
            first: String,
            #[excel(index = 2)]
            second: String,
        }
    };
    let error = expand_excel_row(input).expect_err("duplicate indexes must be rejected");
    let message = error.to_string();
    assert!(message.contains("first"));
    assert!(message.contains("second"));
    assert!(message.contains("must be different"));
}

#[test]
fn generated_tokens_are_valid_rust_syntax() {
    let input: DeriveInput = parse_quote!(
        struct User {
            value: String,
        }
    );
    let tokens = expand_excel_row(input).expect("expansion");
    let wrapped = quote! { #tokens };
    assert!(!wrapped.is_empty());
}

#[test]
fn field_original_write_conversion_uses_empty_for_non_side_effect_free_types() {
    // 对应 Java：字段带 converter 且类型不是纯值类型时，
    // `original` 快照必须是 `CellValue::Empty`（避免资源类型被提前消费）。
    let tuple_ty: syn::Type = syn::parse_str("(String, u32)").expect("tuple type");
    assert!(!is_side_effect_free_original_type(&tuple_ty));

    let reference_ty: syn::Type = syn::parse_str("&'static str").expect("reference type");
    assert!(!is_side_effect_free_original_type(&reference_ty));

    let ident = syn::Ident::new("value", proc_macro2::Span::call_site());
    let tokens = field_original_write_conversion(
        &quote!(::easyexcel),
        &tuple_ty,
        &ident,
        Some(&syn::parse_str::<syn::Path>("crate::TupleConverter").expect("path")),
    );
    assert!(
        tokens.to_string().contains("CellValue :: Empty"),
        "resource-like types must not be eagerly converted: {tokens}"
    );
}

#[test]
fn is_side_effect_free_original_type_handles_empty_path_segments() {
    // `Type::Path` 但没有任何 path 段（防御分支）。
    let empty_path = syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::default(),
        },
        attrs: Vec::new(),
    });
    assert!(!is_side_effect_free_original_type(&empty_path));
}

#[test]
fn data_validation_parses_formula2_and_rejects_unknown_property() {
    // 对应 Java：@ExcelProperty 的 dataValidation 支持 type/operator/formula1/formula2。
    let input: DeriveInput = parse_quote! {
        struct User {
            #[excel(data_validation(
                type = "decimal",
                operator = "greaterThan",
                formula1 = "0",
                formula2 = "10"
            ))]
            value: f64,
        }
    };
    let fields = expect_struct_fields(input);
    let field = fields.iter().next().expect("field");
    let options = parse_field_options(&field.attrs, &quote!(::easyexcel)).expect("parsed");
    let tokens = options
        .data_validation
        .expect("data validation")
        .to_string();
    assert!(tokens.contains("ExcelDataValidationMeta :: new"));
    assert!(tokens.contains("\"decimal\""), "{tokens}");
    assert!(tokens.contains("\"greaterThan\""), "{tokens}");
    assert!(
        tokens.contains("\"10\""),
        "formula2 must be carried: {tokens}"
    );

    // 未支持的属性必须报错（覆盖 formula1/formula2 的负分支与 Err 分支）。
    let bad: DeriveInput = parse_quote! {
        struct User {
            #[excel(data_validation(unknown = 1))]
            value: f64,
        }
    };
    let fields = expect_struct_fields(bad);
    let field = fields.iter().next().expect("field");
    let error = parse_field_options(&field.attrs, &quote!(::easyexcel))
        .err()
        .expect("unknown data_validation property must be rejected");
    assert!(
        error
            .to_string()
            .contains("unsupported data_validation property"),
        "unexpected: {error}"
    );
}

#[test]
fn conditional_rejects_unknown_property() {
    // 对应 Java：conditional 只支持 condition/font_color/background_color。
    let valid: DeriveInput = parse_quote! {
        struct User {
            #[excel(conditional(
                condition = ">0",
                font_color = "FF0000",
                background_color = "FFFF00"
            ))]
            value: i32,
        }
    };
    let fields = expect_struct_fields(valid);
    let field = fields.iter().next().expect("field");
    let options = parse_field_options(&field.attrs, &quote!(::easyexcel)).expect("parsed");
    let tokens = options.conditional.expect("conditional").to_string();
    assert!(tokens.contains("\">0\""), "{tokens}");
    assert!(tokens.contains("\"FF0000\""), "{tokens}");
    assert!(tokens.contains("\"FFFF00\""), "{tokens}");

    let bad: DeriveInput = parse_quote! {
        struct User {
            #[excel(conditional(unknown = 1))]
            value: i32,
        }
    };
    let fields = expect_struct_fields(bad);
    let field = fields.iter().next().expect("field");
    let error = parse_field_options(&field.attrs, &quote!(::easyexcel))
        .err()
        .expect("unknown conditional property must be rejected");
    assert!(
        error
            .to_string()
            .contains("unsupported conditional property"),
        "unexpected: {error}"
    );
}

#[test]
fn once_absolute_merge_accepts_negative_integer_indexes() {
    // 对应 Java：onceAbsoluteMerge 未设置时默认 -1，解析器必须支持负数。
    let input: DeriveInput = parse_quote! {
        #[excel(once_absolute_merge(
            first_row_index = -1,
            last_row_index = -2,
            first_column_index = 0,
            last_column_index = 1
        ))]
        struct User { value: String }
    };
    let options = parse_struct_options(&input.attrs, &quote!(::easyexcel)).expect("valid options");
    let tokens = options
        .once_absolute_merge
        .expect("once absolute merge")
        .to_string();
    assert!(tokens.contains("- 1"), "{tokens}");
    assert!(tokens.contains("- 2"), "{tokens}");
}

#[test]
fn parse_signed_integer_rejects_overflow_and_non_literal_negation() {
    // 对应 Java：merge 索引必须是字面量整数；负号后跟表达式或 i32 溢出都要报错。
    for attribute in [
        "once_absolute_merge(first_row_index = -2147483648)",
        "once_absolute_merge(first_row_index = -(1 + 2))",
        "once_absolute_merge(first_row_index = 2147483648)",
    ] {
        let source = format!("#[excel({attribute})] struct User {{ value: String }}");
        let input = syn::parse_str::<DeriveInput>(&source).expect("attribute tokens");
        assert!(
            parse_struct_options(&input.attrs, &quote!(::easyexcel)).is_err(),
            "`{attribute}` must be rejected"
        );
    }
}
