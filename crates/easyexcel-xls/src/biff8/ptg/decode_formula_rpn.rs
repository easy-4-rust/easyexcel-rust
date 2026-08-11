/// 将 BIFF8 FORMULA 的 RPN token 恢复为可观察的 A1 公式文本。
///
/// 对应 Apache POI：`HSSFFormulaParser.toFormulaString`。当前覆盖写入器会生成的
/// 标量、引用、3D 引用、运算符和内建函数 token；遇到未知 token 返回 `None`，
/// 调用方可保留缓存值而不伪造表达式。
#[must_use]
pub(crate) fn decode_formula_rpn(
    tokens: &[u8],
    sheet_names: &[String],
    extern_sheets: &[(u16, u16)],
) -> Option<String> {
    let mut cursor = 0usize;
    let mut stack = Vec::<String>::new();
    while cursor < tokens.len() {
        let token = tokens[cursor];
        cursor += 1;
        match token {
            0x03..=0x0e => {
                let right = stack.pop()?;
                let left = stack.pop()?;
                let operator = match token {
                    0x03 => "+", 0x04 => "-", 0x05 => "*", 0x06 => "/",
                    0x07 => "^", 0x08 => "&", 0x09 => "<", 0x0a => "<=",
                    0x0b => "=", 0x0c => ">=", 0x0d => ">", 0x0e => "<>",
                    _ => return None,
                };
                stack.push(format!("{left}{operator}{right}"));
            }
            0x12 | 0x13 => {
                let value = stack.pop()?;
                stack.push(format!("{}{value}", if token == 0x12 { "+" } else { "-" }));
            }
            0x14 => {
                let value = stack.pop()?;
                stack.push(format!("{value}%"));
            }
            0x15 => {
                let value = stack.pop()?;
                stack.push(format!("({value})"));
            }
            0x16 => stack.push(String::new()),
            0x17 => {
                let length = usize::from(*tokens.get(cursor)?);
                let flags = *tokens.get(cursor + 1)?;
                cursor += 2;
                let text = if flags & 0x01 == 0 {
                    let bytes = tokens.get(cursor..cursor.checked_add(length)?)?;
                    cursor += length;
                    bytes.iter().map(|byte| char::from(*byte)).collect::<String>()
                } else {
                    let byte_length = length.checked_mul(2)?;
                    let bytes = tokens.get(cursor..cursor.checked_add(byte_length)?)?;
                    cursor += byte_length;
                    let units = bytes
                        .chunks_exact(2)
                        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                        .collect::<Vec<_>>();
                    String::from_utf16_lossy(&units)
                };
                stack.push(format!("\"{}\"", text.replace('"', "\"\"")));
            }
            0x1c => {
                let code = *tokens.get(cursor)?;
                cursor += 1;
                stack.push(error_literal(code).to_owned());
            }
            0x1d => {
                let value = *tokens.get(cursor)? != 0;
                cursor += 1;
                stack.push(if value { "TRUE" } else { "FALSE" }.to_owned());
            }
            0x1e => {
                let bytes: [u8; 2] = tokens.get(cursor..cursor + 2)?.try_into().ok()?;
                cursor += 2;
                stack.push(i16::from_le_bytes(bytes).to_string());
            }
            0x1f => {
                let bytes: [u8; 8] = tokens.get(cursor..cursor + 8)?.try_into().ok()?;
                cursor += 8;
                stack.push(f64::from_le_bytes(bytes).to_string());
            }
            0x21 | 0x41 | 0x61 => {
                let function_index = read_u16(tokens, &mut cursor)?;
                let (_, name, minimum, maximum, _) = BUILTIN_FUNCTIONS
                    .iter()
                    .find(|(index, ..)| *index == function_index)?;
                let argument_count = if minimum == maximum { *minimum } else { *maximum };
                push_function(&mut stack, name, usize::from(argument_count))?;
            }
            0x22 | 0x42 | 0x62 => {
                let argument_count = usize::from(*tokens.get(cursor)?);
                cursor += 1;
                let function_index = read_u16(tokens, &mut cursor)? & 0x7fff;
                let (_, name, ..) = BUILTIN_FUNCTIONS
                    .iter()
                    .find(|(index, ..)| *index == function_index)?;
                push_function(&mut stack, name, argument_count)?;
            }
            0x24 | 0x44 | 0x64 => {
                let row = read_u16(tokens, &mut cursor)?;
                let column = read_u16(tokens, &mut cursor)?;
                stack.push(a1_reference(row, column));
            }
            0x25 | 0x45 | 0x65 => {
                let first_row = read_u16(tokens, &mut cursor)?;
                let last_row = read_u16(tokens, &mut cursor)?;
                let first_column = read_u16(tokens, &mut cursor)?;
                let last_column = read_u16(tokens, &mut cursor)?;
                stack.push(format!(
                    "{}:{}",
                    a1_reference(first_row, first_column),
                    a1_reference(last_row, last_column)
                ));
            }
            0x3a | 0x5a | 0x7a => {
                let ixti = usize::from(read_u16(tokens, &mut cursor)?);
                let row = read_u16(tokens, &mut cursor)?;
                let column = read_u16(tokens, &mut cursor)?;
                stack.push(format!(
                    "{}!{}",
                    sheet_reference(sheet_names, extern_sheets.get(ixti).copied()),
                    a1_reference(row, column)
                ));
            }
            0x3b | 0x5b | 0x7b => {
                let ixti = usize::from(read_u16(tokens, &mut cursor)?);
                let first_row = read_u16(tokens, &mut cursor)?;
                let last_row = read_u16(tokens, &mut cursor)?;
                let first_column = read_u16(tokens, &mut cursor)?;
                let last_column = read_u16(tokens, &mut cursor)?;
                stack.push(format!(
                    "{}!{}:{}",
                    sheet_reference(sheet_names, extern_sheets.get(ixti).copied()),
                    a1_reference(first_row, first_column),
                    a1_reference(last_row, last_column)
                ));
            }
            _ => return None,
        }
    }
    (stack.len() == 1).then(|| format!("={}", stack.pop().unwrap_or_default()))
}

fn read_u16(tokens: &[u8], cursor: &mut usize) -> Option<u16> {
    let bytes: [u8; 2] = tokens
        .get(*cursor..(*cursor).checked_add(2)?)?
        .try_into()
        .ok()?;
    *cursor += 2;
    Some(u16::from_le_bytes(bytes))
}

fn push_function(stack: &mut Vec<String>, name: &str, argument_count: usize) -> Option<()> {
    if argument_count > stack.len() { return None; }
    let start = stack.len() - argument_count;
    let arguments = stack.drain(start..).collect::<Vec<_>>().join(",");
    stack.push(format!("{name}({arguments})"));
    Some(())
}

fn a1_reference(row: u16, encoded_column: u16) -> String {
    let row_absolute = encoded_column & 0x8000 == 0;
    let column_absolute = encoded_column & 0x4000 == 0;
    let mut column = usize::from(encoded_column & 0x00ff);
    let mut letters = String::new();
    loop {
        letters.insert(0, char::from(b'A' + u8::try_from(column % 26).unwrap_or(0)));
        if column < 26 { break; }
        column = column / 26 - 1;
    }
    format!(
        "{}{}{}{}",
        if column_absolute { "$" } else { "" },
        letters,
        if row_absolute { "$" } else { "" },
        u32::from(row) + 1
    )
}

fn sheet_reference(sheet_names: &[String], range: Option<(u16, u16)>) -> String {
    let Some((first, last)) = range else { return "#REF".to_owned(); };
    let quote = |name: &str| {
        if name.chars().all(|character| character.is_ascii_alphanumeric() || character == '_') {
            name.to_owned()
        } else {
            format!("'{}'", name.replace('\'', "''"))
        }
    };
    let first_name = sheet_names.get(usize::from(first)).map_or("#REF", String::as_str);
    if first == last {
        quote(first_name)
    } else {
        let last_name = sheet_names.get(usize::from(last)).map_or("#REF", String::as_str);
        format!("{}:{}", quote(first_name), quote(last_name))
    }
}

fn error_literal(code: u8) -> &'static str {
    match code {
        0x00 => "#NULL!", 0x07 => "#DIV/0!", 0x0f => "#VALUE!",
        0x17 => "#REF!", 0x1d => "#NAME?", 0x24 => "#NUM!",
        0x2a => "#N/A", 0x2b => "#GETTING_DATA", _ => "#VALUE!",
    }
}

#[cfg(test)]
mod decode_formula_rpn_tests {
    use super::*;

    // ── 辅助函数 ──────────────────────────────────────────────────────

    /// 构造整数 token (0x1e + i16 LE)。
    fn int_token(value: i16) -> Vec<u8> {
        let mut t = vec![0x1e];
        t.extend_from_slice(&value.to_le_bytes());
        t
    }

    /// 构造浮点 token (0x1f + f64 LE)。
    fn float_token(value: f64) -> Vec<u8> {
        let mut t = vec![0x1f];
        t.extend_from_slice(&value.to_le_bytes());
        t
    }

    /// 构造压缩字符串 token (0x17, flags=0)。
    fn string_token_compressed(text: &[u8]) -> Vec<u8> {
        let mut t = vec![0x17, text.len() as u8, 0x00];
        t.extend_from_slice(text);
        t
    }

    /// 构造宽字符串 token (0x17, flags=1)。
    fn string_token_wide(utf16_units: &[u16]) -> Vec<u8> {
        let mut t = vec![0x17, utf16_units.len() as u8, 0x01];
        for unit in utf16_units {
            t.extend_from_slice(&unit.to_le_bytes());
        }
        t
    }

    /// 将多个 token 片段拼接为一个 Vec<u8>。
    fn concat_tokens(parts: &[&[u8]]) -> Vec<u8> {
        let mut t = Vec::new();
        for part in parts {
            t.extend_from_slice(part);
        }
        t
    }

    // ── 辅助函数单元测试 ──────────────────────────────────────────────

    #[test]
    fn a1_reference_absolute_both() {
        // $A$1: row=0, encoded_column=0x0000
        assert_eq!(a1_reference(0, 0x0000), "$A$1");
    }

    #[test]
    fn a1_reference_relative_both() {
        // A1: row=0, encoded_column=0xC000 (bit15=1 row_rel, bit14=1 col_rel)
        assert_eq!(a1_reference(0, 0xC000), "A1");
    }

    #[test]
    fn a1_reference_mixed_abs_row() {
        // $A1: row=0, encoded_column=0x8000 (bit15=1 row_rel, bit14=0 col_abs)
        assert_eq!(a1_reference(0, 0x8000), "$A1");
    }

    #[test]
    fn a1_reference_mixed_abs_col() {
        // A$1: row=0, encoded_column=0x4000 (bit15=0 row_abs, bit14=1 col_rel)
        assert_eq!(a1_reference(0, 0x4000), "A$1");
    }

    #[test]
    fn a1_reference_column_z() {
        // Z26: row=25, col=25
        assert_eq!(a1_reference(25, 0xC000 | 25), "Z26");
    }

    #[test]
    fn a1_reference_column_aa() {
        // AA1: row=0, col=26
        assert_eq!(a1_reference(0, 0xC000 | 26), "AA1");
    }

    #[test]
    fn sheet_reference_single_same_sheet() {
        let names = vec!["Sheet1".to_owned()];
        assert_eq!(sheet_reference(&names, Some((0, 0))), "Sheet1");
    }

    #[test]
    fn sheet_reference_range() {
        let names = vec!["Sheet1".to_owned(), "Sheet2".to_owned()];
        assert_eq!(sheet_reference(&names, Some((0, 1))), "Sheet1:Sheet2");
    }

    #[test]
    fn sheet_reference_special_chars_quoted() {
        let names = vec!["My Sheet".to_owned()];
        assert_eq!(sheet_reference(&names, Some((0, 0))), "'My Sheet'");
    }

    #[test]
    fn sheet_reference_single_quote_escaped() {
        let names = vec!["It's".to_owned()];
        assert_eq!(sheet_reference(&names, Some((0, 0))), "'It''s'");
    }

    #[test]
    fn sheet_reference_missing_range_returns_hash_ref() {
        assert_eq!(sheet_reference(&[], None), "#REF");
    }

    #[test]
    fn sheet_reference_missing_name_gets_quoted() {
        // sheet_names 为空 → first_name="#REF"，含 '#' → quote → "'#REF'"
        let names: Vec<String> = vec![];
        assert_eq!(sheet_reference(&names, Some((0, 0))), "'#REF'");
    }

    #[test]
    fn error_literal_known_codes() {
        assert_eq!(error_literal(0x00), "#NULL!");
        assert_eq!(error_literal(0x07), "#DIV/0!");
        assert_eq!(error_literal(0x0f), "#VALUE!");
        assert_eq!(error_literal(0x17), "#REF!");
        assert_eq!(error_literal(0x1d), "#NAME?");
        assert_eq!(error_literal(0x24), "#NUM!");
        assert_eq!(error_literal(0x2a), "#N/A");
        assert_eq!(error_literal(0x2b), "#GETTING_DATA");
    }

    #[test]
    fn error_literal_unknown_code_falls_back_to_value() {
        assert_eq!(error_literal(0xFF), "#VALUE!");
    }

    // ── 二元运算符 (0x03..=0x0e) ─────────────────────────────────────

    #[test]
    fn binary_add() {
        // 1+2 → [0x1e 01 00][0x1e 02 00][0x03]
        let t = concat_tokens(&[&int_token(1), &int_token(2), &[0x03]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=1+2".to_owned()));
    }

    #[test]
    fn binary_sub() {
        let t = concat_tokens(&[&int_token(5), &int_token(3), &[0x04]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=5-3".to_owned()));
    }

    #[test]
    fn binary_mul() {
        let t = concat_tokens(&[&int_token(2), &int_token(3), &[0x05]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=2*3".to_owned()));
    }

    #[test]
    fn binary_div() {
        let t = concat_tokens(&[&int_token(6), &int_token(2), &[0x06]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=6/2".to_owned()));
    }

    #[test]
    fn binary_power() {
        let t = concat_tokens(&[&int_token(2), &int_token(3), &[0x07]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=2^3".to_owned()));
    }

    #[test]
    fn binary_concat() {
        let t = concat_tokens(&[
            &string_token_compressed(b"a"),
            &string_token_compressed(b"b"),
            &[0x08],
        ]);
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=\"a\"&\"b\"".to_owned())
        );
    }

    #[test]
    fn binary_lt() {
        let t = concat_tokens(&[&int_token(1), &int_token(2), &[0x09]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=1<2".to_owned()));
    }

    #[test]
    fn binary_lte() {
        let t = concat_tokens(&[&int_token(1), &int_token(2), &[0x0a]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=1<=2".to_owned()));
    }

    #[test]
    fn binary_eq() {
        let t = concat_tokens(&[&int_token(1), &int_token(2), &[0x0b]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=1=2".to_owned()));
    }

    #[test]
    fn binary_gte() {
        let t = concat_tokens(&[&int_token(1), &int_token(2), &[0x0c]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=1>=2".to_owned()));
    }

    #[test]
    fn binary_gt() {
        let t = concat_tokens(&[&int_token(1), &int_token(2), &[0x0d]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=1>2".to_owned()));
    }

    #[test]
    fn binary_ne() {
        let t = concat_tokens(&[&int_token(1), &int_token(2), &[0x0e]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=1<>2".to_owned()));
    }

    // ── 一元运算符 (0x12/0x13)、百分比 (0x14)、括号 (0x15) ────────────

    #[test]
    fn unary_positive() {
        let t = concat_tokens(&[&int_token(5), &[0x12]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=+5".to_owned()));
    }

    #[test]
    fn unary_negative() {
        let t = concat_tokens(&[&int_token(3), &[0x13]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=-3".to_owned()));
    }

    #[test]
    fn percent() {
        let t = concat_tokens(&[&int_token(50), &[0x14]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=50%".to_owned()));
    }

    #[test]
    fn parentheses() {
        // (1+2) → [INT 1][INT 2][ADD][PAREN]
        let t = concat_tokens(&[&int_token(1), &int_token(2), &[0x03, 0x15]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=(1+2)".to_owned()));
    }

    // ── MissArg (0x16) ────────────────────────────────────────────────

    #[test]
    fn miss_arg_pushes_empty_string() {
        let t = vec![0x16];
        // stack 有 1 个空字符串 → "="
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=".to_owned()));
    }

    // ── 标量 token：字符串 (0x17) ─────────────────────────────────────

    #[test]
    fn string_compressed_ascii() {
        let t = string_token_compressed(b"hello");
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=\"hello\"".to_owned())
        );
    }

    #[test]
    fn string_compressed_with_embedded_quotes() {
        // "say ""hi""" → 内嵌双引号需转义
        let t = string_token_compressed(b"say \"hi\"");
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=\"say \"\"hi\"\"\"".to_owned())
        );
    }

    #[test]
    fn string_wide_unicode() {
        // "你好" → U+4F60 U+597D
        let t = string_token_wide(&[0x4F60, 0x597D]);
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=\"你好\"".to_owned())
        );
    }

    // ── 错误常量 (0x1c) ───────────────────────────────────────────────

    #[test]
    fn error_na() {
        let t = vec![0x1c, 0x2a];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=#N/A".to_owned()));
    }

    #[test]
    fn error_div0() {
        let t = vec![0x1c, 0x07];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=#DIV/0!".to_owned())
        );
    }

    #[test]
    fn error_value() {
        let t = vec![0x1c, 0x0f];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=#VALUE!".to_owned())
        );
    }

    #[test]
    fn error_ref() {
        let t = vec![0x1c, 0x17];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=#REF!".to_owned()));
    }

    #[test]
    fn error_name() {
        let t = vec![0x1c, 0x1d];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=#NAME?".to_owned())
        );
    }

    #[test]
    fn error_num() {
        let t = vec![0x1c, 0x24];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=#NUM!".to_owned()));
    }

    #[test]
    fn error_null() {
        let t = vec![0x1c, 0x00];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=#NULL!".to_owned())
        );
    }

    #[test]
    fn error_getting_data() {
        let t = vec![0x1c, 0x2b];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=#GETTING_DATA".to_owned())
        );
    }

    #[test]
    fn error_unknown_code_defaults_to_value() {
        let t = vec![0x1c, 0xFF];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=#VALUE!".to_owned())
        );
    }

    // ── 布尔 (0x1d) ──────────────────────────────────────────────────

    #[test]
    fn boolean_true() {
        let t = vec![0x1d, 0x01];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=TRUE".to_owned()));
    }

    #[test]
    fn boolean_false() {
        let t = vec![0x1d, 0x00];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=FALSE".to_owned()));
    }

    // ── 整数 (0x1e) ──────────────────────────────────────────────────

    #[test]
    fn integer_positive() {
        let t = int_token(42);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=42".to_owned()));
    }

    #[test]
    fn integer_negative() {
        let t = int_token(-7);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=-7".to_owned()));
    }

    #[test]
    fn integer_zero() {
        let t = int_token(0);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=0".to_owned()));
    }

    // ── 浮点数 (0x1f) ────────────────────────────────────────────────

    #[test]
    fn float_3_14() {
        let t = float_token(3.14);
        let result = decode_formula_rpn(&t, &[], &[]).unwrap();
        assert!(result.starts_with("=3.14"), "got: {result}");
    }

    #[test]
    fn float_negative() {
        let t = float_token(-2.5);
        let result = decode_formula_rpn(&t, &[], &[]).unwrap();
        assert!(result.starts_with("=-2.5"), "got: {result}");
    }

    // ── 内建函数：固定参数 (0x21/0x41/0x61) ──────────────────────────

    #[test]
    fn function_pi_zero_args() {
        // PI() 索引 0x0013, min=max=0
        let t = vec![0x41, 0x13, 0x00];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=PI()".to_owned()));
    }

    #[test]
    fn function_round_fixed_two_args() {
        // ROUND(A1,2) → [A1][INT 2][FUNC ROUND]
        let t = concat_tokens(&[
            &[0x24, 0x00, 0x00, 0x00, 0xC0], // A1
            &int_token(2),
            &[0x41, 0x1b, 0x00], // tFunc ROUND
        ]);
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=ROUND(A1,2)".to_owned())
        );
    }

    // ── 内建函数：可变参数 (0x22/0x42/0x62) ──────────────────────────

    #[test]
    fn function_sum_variable_args() {
        // SUM(1,2) → [INT 1][INT 2][FUNCVAR SUM count=2]
        let t = concat_tokens(&[
            &int_token(1),
            &int_token(2),
            &[0x42, 0x02, 0x04, 0x00], // tFuncVar SUM, count=2
        ]);
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=SUM(1,2)".to_owned())
        );
    }

    #[test]
    fn function_sum_three_args() {
        // SUM(A1,B1,2)
        let t = concat_tokens(&[
            &[0x24, 0x00, 0x00, 0x00, 0xC0], // A1
            &[0x24, 0x00, 0x00, 0x01, 0xC0], // B1
            &int_token(2),
            &[0x42, 0x03, 0x04, 0x00], // tFuncVar SUM, count=3
        ]);
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=SUM(A1,B1,2)".to_owned())
        );
    }

    // ── 单元格引用 (0x24/0x44/0x64) ──────────────────────────────────

    #[test]
    fn cell_ref_relative() {
        // A1: row=0, col encoded with relative flags
        let t = vec![0x24, 0x00, 0x00, 0x00, 0xC0];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=A1".to_owned()));
    }

    #[test]
    fn cell_ref_absolute() {
        // $A$1: row=0, col=0x0000 (both absolute)
        let t = vec![0x24, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=$A$1".to_owned())
        );
    }

    #[test]
    fn cell_ref_mixed_abs_col() {
        // $A1: col absolute, row relative → encoded=0x8000
        let t = vec![0x24, 0x00, 0x00, 0x00, 0x80];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=$A1".to_owned()));
    }

    #[test]
    fn cell_ref_mixed_abs_row() {
        // A$1: col relative, row absolute → encoded=0x4000
        let t = vec![0x24, 0x00, 0x00, 0x00, 0x40];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=A$1".to_owned()));
    }

    #[test]
    fn cell_ref_beyond_z() {
        // AA1: row=0, col=26 (0x1A), relative
        let t = vec![0x24, 0x00, 0x00, 0x1A, 0xC0];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=AA1".to_owned()));
    }

    // ── 区域引用 (0x25/0x45/0x65) ────────────────────────────────────

    #[test]
    fn area_ref_simple() {
        // A1:B2: first_row=0, last_row=1, first_col=0, last_col=1, relative
        let t = vec![0x25, 0x00, 0x00, 0x01, 0x00, 0x00, 0xC0, 0x01, 0xC0];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=A1:B2".to_owned())
        );
    }

    #[test]
    fn area_ref_absolute() {
        // $A$1:$B$2: all absolute
        let t = vec![0x25, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=$A$1:$B$2".to_owned())
        );
    }

    // ──3D 引用 (0x3a/0x5a/0x7a) ─────────────────────────────────────

    #[test]
    fn three_d_ref_single_sheet() {
        // Sheet1!A1: ixti=0, row=0, col=0xC000
        let t = vec![0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0];
        let sheets = vec!["Sheet1".to_owned()];
        let externs = vec![(0u16, 0u16)];
        assert_eq!(
            decode_formula_rpn(&t, &sheets, &externs),
            Some("=Sheet1!A1".to_owned())
        );
    }

    #[test]
    fn three_d_ref_special_char_sheet() {
        // 'My Sheet'!A1: ixti=0
        let t = vec![0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0];
        let sheets = vec!["My Sheet".to_owned()];
        let externs = vec![(0u16, 0u16)];
        assert_eq!(
            decode_formula_rpn(&t, &sheets, &externs),
            Some("='My Sheet'!A1".to_owned())
        );
    }

    #[test]
    fn three_d_ref_sheet_range() {
        // Sheet1:Sheet2!A1: ixti=0, first_sheet=0, last_sheet=1
        let t = vec![0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0];
        let sheets = vec!["Sheet1".to_owned(), "Sheet2".to_owned()];
        let externs = vec![(0u16, 1u16)];
        assert_eq!(
            decode_formula_rpn(&t, &sheets, &externs),
            Some("=Sheet1:Sheet2!A1".to_owned())
        );
    }

    #[test]
    fn three_d_ref_missing_ixti() {
        // ixti=0 但 extern_sheets 为空 → sheet_reference(None) → "#REF"
        let t = vec![0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=#REF!A1".to_owned())
        );
    }

    #[test]
    fn three_d_ref_missing_sheet_name() {
        // ixti=0 → (0,0) 但 sheet_names 为空 → quote("#REF") → "'#REF'"
        let t = vec![0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0];
        let externs = vec![(0u16, 0u16)];
        assert_eq!(
            decode_formula_rpn(&t, &[], &externs),
            Some("='#REF'!A1".to_owned())
        );
    }

    // ──3D 区域引用 (0x3b/0x5b/0x7b) ─────────────────────────────────

    #[test]
    fn three_d_area_ref() {
        // Sheet1!A1:B2: ixti=0, first_row=0, last_row=1, first_col=0, last_col=1
        let t = vec![0x3b, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0xC0, 0x01, 0xC0];
        let sheets = vec!["Sheet1".to_owned()];
        let externs = vec![(0u16, 0u16)];
        assert_eq!(
            decode_formula_rpn(&t, &sheets, &externs),
            Some("=Sheet1!A1:B2".to_owned())
        );
    }

    #[test]
    fn three_d_area_ref_with_range() {
        // Sheet1:Sheet2!A1:B2
        let t = vec![0x3b, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0xC0, 0x01, 0xC0];
        let sheets = vec!["Sheet1".to_owned(), "Sheet2".to_owned()];
        let externs = vec![(0u16, 1u16)];
        assert_eq!(
            decode_formula_rpn(&t, &sheets, &externs),
            Some("=Sheet1:Sheet2!A1:B2".to_owned())
        );
    }

    // ── 边界与错误路径 ───────────────────────────────────────────────

    #[test]
    fn unknown_token_returns_none() {
        let t = vec![0xFF];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn empty_tokens_returns_none() {
        assert_eq!(decode_formula_rpn(&[], &[], &[]), None);
    }

    #[test]
    fn binary_operator_stack_underflow_returns_none() {
        // 只有 1 个整数 + ADD → 栈不够 pop
        let t = concat_tokens(&[&int_token(1), &[0x03]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn binary_operator_empty_stack_returns_none() {
        let t = vec![0x03];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn unary_negative_empty_stack_returns_none() {
        let t = vec![0x13];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn percent_empty_stack_returns_none() {
        let t = vec![0x14];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn parentheses_empty_stack_returns_none() {
        let t = vec![0x15];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn multi_item_stack_returns_none() {
        // 两个整数但没有运算符 → stack.len() == 2 → None
        let t = concat_tokens(&[&int_token(1), &int_token(2)]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn truncated_integer_token_returns_none() {
        // 0x1e 后只有 1 字节（需要 2 字节）
        let t = vec![0x1e, 0x01];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn truncated_float_token_returns_none() {
        // 0x1f 后只有 4 字节（需要 8 字节）
        let t = vec![0x1f, 0x00, 0x00, 0x00];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn truncated_string_token_returns_none() {
        // 声明 5 字节但只有 2 字节
        let t = vec![0x17, 0x05, 0x00, 0x41, 0x42];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn truncated_error_token_returns_none() {
        let t = vec![0x1c];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn truncated_boolean_token_returns_none() {
        let t = vec![0x1d];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn truncated_cell_ref_returns_none() {
        let t = vec![0x24, 0x00, 0x00];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn truncated_area_ref_returns_none() {
        let t = vec![0x25, 0x00, 0x00, 0x01, 0x00, 0x00, 0xC0];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn truncated_3d_ref_returns_none() {
        let t = vec![0x3a, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn truncated_3d_area_ref_returns_none() {
        let t = vec![0x3b, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0xC0];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn function_unknown_index_returns_none() {
        // 索引 0xFFFF 不在 BUILTIN_FUNCTIONS 表中
        let t = concat_tokens(&[&int_token(1), &[0x41, 0xFF, 0xFF]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn function_var_unknown_index_returns_none() {
        let t = concat_tokens(&[&int_token(1), &[0x42, 0x01, 0xFF, 0xFF]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    #[test]
    fn function_insufficient_stack_returns_none() {
        // ROUND 需要 2 个参数 → 栈为空
        let t = vec![0x41, 0x1b, 0x00];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    // ── 复合公式 ─────────────────────────────────────────────────────

    #[test]
    fn complex_formula_1_plus_2_mul_3() {
        // 1+2*3 → [INT 1][INT 2][INT 3][MUL][ADD]
        let t = concat_tokens(&[
            &int_token(1),
            &int_token(2),
            &int_token(3),
            &[0x05, 0x03],
        ]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=1+2*3".to_owned()));
    }

    #[test]
    fn complex_formula_neg_power() {
        // -2^2 → [INT 2][UNARY_NEG][INT 2][POW]
        let t = concat_tokens(&[
            &int_token(2),
            &[0x13],
            &int_token(2),
            &[0x07],
        ]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=-2^2".to_owned()));
    }

    #[test]
    fn complex_formula_paren_times() {
        // (A1+B1)*2 → [A1][B1][ADD][PAREN][INT 2][MUL]
        let t = concat_tokens(&[
            &[0x24, 0x00, 0x00, 0x00, 0xC0], // A1
            &[0x24, 0x00, 0x00, 0x01, 0xC0], // B1
            &[0x03, 0x15],                     // ADD, PAREN
            &int_token(2),
            &[0x05],                           // MUL
        ]);
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=(A1+B1)*2".to_owned())
        );
    }

    // ── 不同 token 变体前缀 (0x21/0x41/0x61 等) ─────────────────────

    #[test]
    fn function_r_class_variant() {
        // NA() 索引 0x000a, min=max=0
        let t = vec![0x21, 0x0a, 0x00];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=NA()".to_owned()));
    }

    #[test]
    fn function_a_class_variant() {
        let t = vec![0x61, 0x13, 0x00];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=PI()".to_owned()));
    }

    #[test]
    fn function_var_r_class_variant() {
        let t = concat_tokens(&[&int_token(5), &[0x22, 0x01, 0x04, 0x00]]);
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=SUM(5)".to_owned())
        );
    }

    #[test]
    fn function_var_a_class_variant() {
        let t = concat_tokens(&[&int_token(10), &[0x62, 0x01, 0x04, 0x00]]);
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=SUM(10)".to_owned())
        );
    }

    #[test]
    fn cell_ref_r_class_variant() {
        let t = vec![0x24, 0x05, 0x00, 0x02, 0xC0]; // C6
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=C6".to_owned()));
    }

    #[test]
    fn cell_ref_a_class_variant() {
        let t = vec![0x64, 0x05, 0x00, 0x02, 0xC0]; // C6
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=C6".to_owned()));
    }

    #[test]
    fn area_ref_a_class_variant() {
        let t = vec![0x65, 0x00, 0x00, 0x01, 0x00, 0x00, 0xC0, 0x01, 0xC0];
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=A1:B2".to_owned())
        );
    }

    // ── 广字符串截断 ─────────────────────────────────────────────────

    #[test]
    fn truncated_wide_string_returns_none() {
        // 声明 2 个 wide 字符（需要 4 字节），但只有 2 字节
        let t = vec![0x17, 0x02, 0x01, 0x60, 0x4F];
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    // ── read_u16 边界 ────────────────────────────────────────────────

    #[test]
    fn read_u16_exact_boundary() {
        let t = int_token(100);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), Some("=100".to_owned()));
    }

    // ── push_function 参数不足 ────────────────────────────────────────

    #[test]
    fn function_var_needs_more_than_available() {
        // SUM 需要 3 参数但只有 1 个
        let t = concat_tokens(&[&int_token(1), &[0x42, 0x03, 0x04, 0x00]]);
        assert_eq!(decode_formula_rpn(&t, &[], &[]), None);
    }

    // ── 空字符串 token 特殊长度 ──────────────────────────────────────

    #[test]
    fn string_zero_length() {
        let t = vec![0x17, 0x00, 0x00]; // 长度 0，压缩
        assert_eq!(
            decode_formula_rpn(&t, &[], &[]),
            Some("=\"\"".to_owned())
        );
    }
}
