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
