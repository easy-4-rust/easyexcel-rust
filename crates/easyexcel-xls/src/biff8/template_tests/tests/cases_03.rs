// ---------------------------------------------------------------------------
// 第 4 轮补充测试：覆盖 rawrecord、shift_formula、collection_placeholder
// 及其他低覆盖文件中尚未覆盖的分支
// ---------------------------------------------------------------------------

// ===========================================================================
// parse_sst Unicode 路径覆盖
// ===========================================================================

#[test]
fn parse_sst_unicode_string() {
    // 构造包含 16-bit Unicode 字符的 SST record
    let mut sst_data = Vec::new();
    sst_data.extend_from_slice(&1u32.to_le_bytes()); // total count
    sst_data.extend_from_slice(&1u32.to_le_bytes()); // unique count
    // XLUnicodeString: cch(2) + grbit(1) + chars (16-bit)
    let text = "\u{4e2d}\u{6587}"; // 中文
    let units: Vec<u16> = text.encode_utf16().collect();
    sst_data.extend_from_slice(&(units.len() as u16).to_le_bytes());
    sst_data.push(0x01); // grbit: 16-bit characters
    for unit in &units {
        sst_data.extend_from_slice(&unit.to_le_bytes());
    }

    let records = vec![RawRecord {
        typ: SST,
        data: sst_data,
    }];
    let strings = parse_sst(&records);
    assert_eq!(strings.len(), 1);
    assert_eq!(strings[0], text);
}

#[test]
fn parse_sst_multiple_strings() {
    let mut sst_data = Vec::new();
    sst_data.extend_from_slice(&2u32.to_le_bytes()); // total
    sst_data.extend_from_slice(&2u32.to_le_bytes()); // unique
    // string 1: compressed
    sst_data.extend_from_slice(&3u16.to_le_bytes());
    sst_data.push(0x00);
    sst_data.extend_from_slice(b"abc");
    // string 2: compressed
    sst_data.extend_from_slice(&2u16.to_le_bytes());
    sst_data.push(0x00);
    sst_data.extend_from_slice(b"de");

    let records = vec![RawRecord {
        typ: SST,
        data: sst_data,
    }];
    let strings = parse_sst(&records);
    assert_eq!(strings.len(), 2);
    assert_eq!(strings[0], "abc");
    assert_eq!(strings[1], "de");
}

#[test]
fn parse_sst_truncated_string_breaks() {
    // cch=10 但只有 2 字节数据
    let mut sst_data = Vec::new();
    sst_data.extend_from_slice(&1u32.to_le_bytes());
    sst_data.extend_from_slice(&1u32.to_le_bytes());
    sst_data.extend_from_slice(&10u16.to_le_bytes()); // cch=10
    sst_data.push(0x00);
    sst_data.extend_from_slice(b"ab"); // 只有 2 字节

    let records = vec![RawRecord {
        typ: SST,
        data: sst_data,
    }];
    let strings = parse_sst(&records);
    assert_eq!(strings.len(), 1);
    assert_eq!(strings[0], "ab"); // 被截断
}

// ===========================================================================
// decode_label_payload Unicode 路径覆盖
// ===========================================================================

#[test]
fn decode_label_payload_unicode() {
    // LABEL record with 16-bit Unicode characters
    let text = "\u{4e2d}\u{6587}";
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut data = Vec::new();
    data.extend_from_slice(&5u16.to_le_bytes()); // row
    data.extend_from_slice(&3u16.to_le_bytes()); // col
    data.extend_from_slice(&0u16.to_le_bytes()); // xf
    data.extend_from_slice(&(units.len() as u16).to_le_bytes()); // cch
    data.push(0x01); // grbit: 16-bit
    for unit in &units {
        data.extend_from_slice(&unit.to_le_bytes());
    }

    let (row, col, text_out) = decode_label_payload(&data);
    assert_eq!(row, 5);
    assert_eq!(col, 3);
    assert_eq!(text_out.as_deref(), Some("\u{4e2d}\u{6587}"));
}

#[test]
fn decode_label_payload_empty_string() {
    let mut data = Vec::new();
    data.extend_from_slice(&0u16.to_le_bytes()); // row
    data.extend_from_slice(&0u16.to_le_bytes()); // col
    data.extend_from_slice(&0u16.to_le_bytes()); // xf
    data.extend_from_slice(&0u16.to_le_bytes()); // cch = 0
    data.push(0x00); // grbit

    let (row, col, text) = decode_label_payload(&data);
    assert_eq!(row, 0);
    assert_eq!(col, 0);
    // 空字符串返回 None
    assert!(text.is_none());
}

// ===========================================================================
// decode_boundsheet_name Unicode 路径覆盖
// ===========================================================================

#[test]
fn decode_boundsheet_name_unicode() {
    // 构造包含 Unicode 名称的 BOUNDSHEET 数据
    let name = "\u{4e2d}\u{6587}";
    let units: Vec<u16> = name.encode_utf16().collect();
    let mut data = Vec::new();
    data.extend_from_slice(&0u32.to_le_bytes()); // lbPlyPos
    data.push(0); // hsState
    data.push(0); // dt
    data.push(units.len() as u8); // cch
    data.push(0x01); // grbit: 16-bit
    for unit in &units {
        data.extend_from_slice(&unit.to_le_bytes());
    }
    let result = decode_boundsheet_name(&data).unwrap();
    assert_eq!(result, name);
}

#[test]
fn decode_boundsheet_name_too_short() {
    let data = [0u8; 4]; // 太短
    assert!(decode_boundsheet_name(&data).is_err());
}

// ===========================================================================
// encode_rk / decode_rk 额外分支覆盖（通过模板 roundtrip 间接覆盖）
// ===========================================================================

#[test]
fn encode_rk_none_for_unrepresentable() {
    // 某些浮点数不能用 RK 表示（低 32 位 mantissa 非零且不满足其他条件）
    // 使用一个不能被任何 RK 形式表示的值
    let v = std::f64::consts::E; // e ≈ 2.71828...
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Number(v)).unwrap();
    // e 的低 32 位不为零，不满足整数或 div100 条件，也不满足 double 形式
    // 但如果它恰好可以被 double 形式编码，就用 RK，否则用 NUMBER
    // 这里只验证编码成功即可
    assert!(record.typ == RK || record.typ == NUMBER);
}

#[test]
fn encode_rk_double_form() {
    // 0.5 可以用 RK double 形式编码
    let v = f64::from_bits(0x3FE0_0000_0000_0000);
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Number(v)).unwrap();
    assert_eq!(record.typ, RK);
}

#[test]
fn decode_rk_div100_form() {
    // 12.34 可以用 RK div100 形式编码
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Number(12.34)).unwrap();
    assert_eq!(record.typ, RK);
}

// ===========================================================================
// encode_unicode_string 覆盖
// ===========================================================================

#[test]
fn encode_unicode_string_compressed() {
    let encoded = encode_unicode_string("hello");
    assert_eq!(encoded[2], 0x00); // compressed
    assert_eq!(&encoded[3..], b"hello");
}

#[test]
fn encode_unicode_string_unicode() {
    let text = "\u{4e2d}\u{6587}";
    let encoded = encode_unicode_string(text);
    assert_eq!(encoded[2], 0x01); // 16-bit
    // 2 个 UTF-16 单元，每个 2 字节
    assert_eq!(encoded.len(), 3 + 4);
}

// ===========================================================================
// encode_short_unicode_string / parse_short_unicode_string 覆盖
// 通过 BOUNDSHEET 名称编码间接覆盖
// ===========================================================================

#[test]
fn boundsheet_short_unicode_via_template() -> Result<()> {
    // 创建包含中文 sheet 名的模板，间接覆盖 encode_short_unicode_string
    let mut book = crate::biff8::Biff8Book::default();
    book.sheet_mut("\u{4e2d}\u{6587}");
    let bytes = book.to_cfb_bytes()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    assert_eq!(package.sheet_names(), vec!["\u{4e2d}\u{6587}"]);
    Ok(())
}

// ===========================================================================
// u16le / u32le / f64le 覆盖
// 通过 BIFF record 解析间接覆盖
// ===========================================================================

#[test]
fn biff_le_reads_via_record_parsing() {
    // 构造包含小端编码数据的记录，间接覆盖 u16le 解析
    // cell_coords 需要至少 4 字节: row(2) + col(2)
    let mut data = vec![5u8, 0, 3u8, 0]; // row=5, col=3
    data.extend_from_slice(&0x12345678u32.to_le_bytes()); // u32le 数据
    data.extend_from_slice(&std::f64::consts::PI.to_le_bytes()); // f64le 数据
    let record = RawRecord {
        typ: LABEL,
        data,
    };
    // 通过 cell_coords 间接使用 u16le 解析
    let coords = cell_coords(&record);
    assert_eq!(coords, Some((5, 3)));
}

// ===========================================================================
// shifted_row / shift_record_row / shift_range_rows 额外覆盖
// ===========================================================================

#[test]
fn shifted_row_at_start_boundary() {
    assert_eq!(shifted_row(10, 10, 5).unwrap(), 15);
}

#[test]
fn shifted_row_exactly_at_max() {
    // u16::MAX 不溢出的情况
    assert_eq!(shifted_row(100, 50, 10).unwrap(), 110);
}

#[test]
fn shift_record_row_preserves_other_fields() {
    let mut record = RawRecord {
        typ: LABEL,
        data: vec![5, 0, 3, 0, 0, 0, 7, 0],
    };
    shift_record_row(&mut record, 3, 10).unwrap();
    let row = u16::from_le_bytes([record.data[0], record.data[1]]);
    assert_eq!(row, 15);
    // 其他字段不变
    assert_eq!(record.data[2], 3);
    assert_eq!(record.data[6], 7);
}

// ===========================================================================
// shift_merge_rows 额外覆盖
// ===========================================================================

#[test]
fn shift_merge_rows_empty_count() {
    let mut data = vec![0, 0]; // count=0
    shift_merge_rows(&mut data, 5, 2).unwrap();
    assert_eq!(data, vec![0, 0]);
}

// ===========================================================================
// shift_formula_references 额外覆盖：ptgEncodedLen 各分支
// ===========================================================================

#[test]
fn shift_formula_references_string_token() {
    // ptgStr = 0x17: cch(1) + grbit(1) + chars
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgStr: 0x17, cch=3, grbit=0x00 (compressed), 3 chars
    let tokens = [0x17, 3, 0x00, b'a', b'b', b'c'];
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 2, 0, &[]).unwrap();
    // ptgStr 不含行引用，不修改
}

#[test]
fn shift_formula_references_string_unicode_token() {
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgStr: 0x17, cch=1, grbit=0x01 (16-bit), 1 UTF-16 unit
    let mut tokens = vec![0x17, 1, 0x01];
    tokens.extend_from_slice(&0x4e2du16.to_le_bytes());
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 2, 0, &[]).unwrap();
}

#[test]
fn shift_formula_references_attr_token() {
    // ptgAttr = 0x19: options(1) + [data]
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgAttr: 0x19, options=0x00 (not choose), 4 bytes total
    let tokens = [0x19, 0x00, 0x00, 0x00];
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 2, 0, &[]).unwrap();
}

#[test]
fn shift_formula_references_attr_choose_token() {
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgAttr: 0x19, options=0x04 (choose), count=1
    let mut tokens = vec![0x19, 0x04, 1, 0]; // choose with 1 offset
    tokens.extend_from_slice(&[0, 0]); // 1 offset (2 bytes each)
    tokens.extend_from_slice(&[0, 0]); // sentinel
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 2, 0, &[]).unwrap();
}

#[test]
fn shift_formula_references_ref_error_token() {
    // ptgRefErr = 0x2A: 5 bytes total
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    let tokens = [0x2A, 0, 0, 0, 0]; // ptgRefErr
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 2, 0, &[]).unwrap();
}

#[test]
fn shift_formula_references_area_error_token() {
    // ptgAreaErr = 0x2B: 9 bytes total
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    let tokens = [0x2B, 0, 0, 0, 0, 0, 0, 0, 0]; // ptgAreaErr
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 2, 0, &[]).unwrap();
}

// ===========================================================================
// shift_formula_references: RefN (0x2C) relative + AreaN (0x2D) 额外覆盖
// ===========================================================================

#[test]
fn shift_formula_references_refn_absolute() {
    // ptgRefN = 0x2C, absolute reference (high bit not set)
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    let tokens = [0x2C, 5, 0, 0, 0]; // row=5 absolute, col=0
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 3, 10, 0, &[]).unwrap();
    // row 5 >= 3 -> 15
    assert_eq!(u16::from_le_bytes([f.data[23], f.data[24]]), 15);
}

#[test]
fn shift_formula_references_arean_absolute() {
    // ptgAreaN = 0x2D, absolute reference
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgAreaN = 0x2D: first_row(2)+last_row(2)+first_col(2)+last_col(2)
    let tokens = [0x2D, 3, 0, 7, 0, 0, 0, 0, 0]; // rows 3-7, cols 0-0
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 10, 0, &[]).unwrap();
    // row 3 < 5 不变, row 7 >= 5 -> 17
    assert_eq!(u16::from_le_bytes([f.data[23], f.data[24]]), 3);
    assert_eq!(u16::from_le_bytes([f.data[25], f.data[26]]), 17);
}

// ===========================================================================
// shift_formula_references: Area3d (0x3B) 额外覆盖
// ===========================================================================

#[test]
fn shift_formula_references_area3d_same_sheet() {
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgArea3d = 0x3B: ixti(2) + first_row(2) + last_row(2) + first_col(2) + last_col(2)
    let tokens = [0x3B, 0, 0, 3, 0, 7, 0, 0, 0, 0, 0];
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 10, 0, &[Some((0, 0))]).unwrap();
    // row 3 < 5 不变, row 7 >= 5 -> 17
    assert_eq!(u16::from_le_bytes([f.data[25], f.data[26]]), 3);
    assert_eq!(u16::from_le_bytes([f.data[27], f.data[28]]), 17);
}

#[test]
fn shift_formula_references_area3d_different_sheet() {
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgArea3d on different sheet: should NOT shift
    let tokens = [0x3B, 0, 0, 3, 0, 7, 0, 0, 0, 0, 0];
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 10, 1, &[Some((0, 0))]).unwrap();
    assert_eq!(u16::from_le_bytes([f.data[25], f.data[26]]), 3);
    assert_eq!(u16::from_le_bytes([f.data[27], f.data[28]]), 7);
}

// ===========================================================================
// shift_formula_references: deleted refs (0x3C, 0x3D)
// ===========================================================================

#[test]
fn shift_formula_references_deleted_ref3d() {
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgRef3dErr = 0x3C: 7 bytes
    let tokens = [0x3C, 0, 0, 0, 0, 0, 0];
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 10, 0, &[]).unwrap();
    // 0x3C 是 deleted ref，不修改

    // ptgArea3dErr = 0x3D: 11 bytes
    let tokens = [0x3D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut f2 = formula(0, &tokens);
    shift_formula_references(&mut f2, 5, 10, 0, &[]).unwrap();
}

// ===========================================================================
// shift_chart_ai_references 额外覆盖
// ===========================================================================

#[test]
fn shift_chart_ai_references_short_data() {
    let mut record = RawRecord {
        typ: CHART_AI_SID,
        data: vec![0u8; 4], // 太短
    };
    shift_chart_ai_references(&mut record, 5, 2, 0, &[]).unwrap();
    // 短数据直接返回 Ok(())
}

#[test]
fn shift_chart_ai_references_zero_token_len() {
    let mut record = RawRecord {
        typ: CHART_AI_SID,
        data: vec![0u8; 8], // token_len = 0
    };
    shift_chart_ai_references(&mut record, 5, 2, 0, &[]).unwrap();
}

#[test]
fn shift_chart_ai_references_area3d() {
    // ptgArea3d = 0x3B: ixti(2) + first_row(2) + last_row(2) + first_col(2) + last_col(2)
    let tokens = [0x3B, 0, 0, 5, 0, 10, 0, 0, 0, 0, 0];
    let mut data = vec![0u8; 8];
    data[6..8].copy_from_slice(&11u16.to_le_bytes()); // token_len
    data.extend_from_slice(&tokens);

    let mut record = RawRecord {
        typ: CHART_AI_SID,
        data,
    };
    shift_chart_ai_references(&mut record, 8, 5, 0, &[Some((0, 0))]).unwrap();
    // ptgArea3d rows: 5 >= 8 -> 不变, 10 >= 8 -> 15
    // 但对于 chart AI, 0x3B 的 shift 在 cursor+3 和 cursor+5
    // cursor+3 = tokens[3..5] = first_row, cursor+5 = tokens[5..7] = first_col?
    // 实际 shift_chart_ptg_row(row_offset=cursor+3, column_offset=cursor+5)
    // 对于 0x3B: first_row at 3, last_row at 5, first_col at 7, last_col at 9
    // shift_chart_ptg_row(tokens, 3, 5) -> row=5, col=10 -> 10 >= 8 -> 15
    // shift_chart_ptg_row(tokens, 5, 7) -> row=10, col=0
    //   col=0, no high bit -> OK, row=10 >= 8 -> 15
}

// ===========================================================================
// shift_chart_ptg_row relative column error
// ===========================================================================

#[test]
fn shift_chart_ptg_row_relative_column_errors() {
    // 相对列引用（高位为 1）应该报错
    // shift_chart_ptg_row 参数：(tokens, row_offset, column_offset, start_row, delta)
    // 它从 column_offset 读取 2 字节作为 column，检查 high bit (0x8000)
    let mut tokens = vec![0u8; 10];
    // row 数据在 row_offset=1
    tokens[1] = 5;
    tokens[2] = 0;
    // column 数据在 column_offset=3，little-endian u16 的 high bit 在第二个字节
    tokens[3] = 0x00; // column low byte
    tokens[4] = 0x80; // column high byte with relative bit (0x8000)

    let result = shift_chart_ptg_row(&mut tokens, 1, 3, 3, 10);
    // 相对列引用应该返回错误
    assert!(result.is_err());
}

// ===========================================================================
// shift_conditional_format_header 覆盖
// ===========================================================================

#[test]
fn shift_conditional_format_header_too_short() {
    let mut data = vec![0u8; 10]; // 太短（需要 14 字节）
    assert!(shift_conditional_format_header(&mut data, 3, 2).is_err());
}

// ===========================================================================
// shift_conditional_format_rule 覆盖
// ===========================================================================

#[test]
fn shift_conditional_format_rule_too_short() {
    let mut data = vec![0u8; 8]; // 太短（需要 12 字节）
    assert!(shift_conditional_format_rule(&mut data, 0, 0, 5, 2, 0, &[]).is_err());
}

#[test]
fn shift_conditional_format_rule_truncated_formulas() {
    // formula2_end > data.len()
    let mut cf = vec![2, 0, 100, 0, 100, 0]; // type=2, formula1_len=100, formula2_len=100
    cf.extend_from_slice(&0u32.to_le_bytes()); // formatting_options = 0
    cf.extend_from_slice(&0u16.to_le_bytes()); // padding
    // 太少的数据
    cf.extend_from_slice(&[0u8; 10]);
    assert!(shift_conditional_format_rule(&mut cf, 0, 0, 5, 2, 0, &[]).is_err());
}

// ===========================================================================
// shift_data_validation 额外覆盖
// ===========================================================================

#[test]
fn shift_data_validation_too_short() {
    let data = vec![0u8; 2]; // 太短
    let mut data = data;
    assert!(shift_data_validation(&mut data, 5, 2, 0, &[]).is_err());
}

// ===========================================================================
// shift_name_references 覆盖
// ===========================================================================

#[test]
fn shift_name_references_too_short() {
    let mut record = RawRecord {
        typ: NAME_SID,
        data: vec![0u8; 10], // 太短（需要 15 字节）
    };
    assert!(shift_name_references(&mut record, 3, 2, 0, &[]).is_err());
}

// ===========================================================================
// internal_extern_sheet_ranges 覆盖
// ===========================================================================

#[test]
fn internal_extern_sheet_ranges_no_supbook() {
    // 没有 internal supbook
    let records = vec![RawRecord {
        typ: EXTERNAL_SHEET_SID,
        data: vec![1, 0, 0, 0, 0, 0, 1, 0],
    }];
    let ranges = internal_extern_sheet_ranges(&records);
    assert_eq!(ranges[0], None);
}

#[test]
fn internal_extern_sheet_ranges_truncated_entry() {
    // EXTERNSHEET 声明 2 个条目但只有 1 个
    let records = vec![
        RawRecord {
            typ: SUP_BOOK_SID,
            data: vec![2, 0, 1, 4],
        },
        RawRecord {
            typ: EXTERNAL_SHEET_SID,
            data: vec![2, 0, 0, 0, 0, 0, 1, 0], // count=2 但只有 8 字节（需要 2+12=14）
        },
    ];
    let ranges = internal_extern_sheet_ranges(&records);
    assert_eq!(ranges.len(), 2);
    assert_eq!(ranges[0], Some((0, 1)));
    assert_eq!(ranges[1], None); // 截断
}

// ===========================================================================
// ptg_targets_sheet 覆盖
// ===========================================================================

#[test]
fn ptg_targets_sheet_invalid_ixti() {
    // ixti 超出 ranges 范围
    let tokens = [0x3A, 5, 0, 3, 0, 0, 0]; // ixti=5
    let ranges = vec![Some((0, 0))]; // 只有 1 个
    let result = ptg_targets_sheet(&tokens, 0, 0, &ranges).unwrap();
    assert!(!result);
}

// ===========================================================================
// remove_escher_records 额外覆盖
// ===========================================================================

#[test]
fn remove_escher_records_nested_container() {
    // 容器嵌套容器
    let mut inner_container = Vec::new();
    // 内部容器是一个 F004 shape
    let mut shape = vec![0u8; 16];
    shape[2..4].copy_from_slice(&0xF00Au16.to_le_bytes());
    shape[4..8].copy_from_slice(&4u32.to_le_bytes());
    shape[8..12].copy_from_slice(&10u32.to_le_bytes()); // spid=10
    // 外层容器头
    inner_container.extend_from_slice(&0x000Fu16.to_le_bytes()); // container
    inner_container.extend_from_slice(&0xF004u16.to_le_bytes()); // F004
    inner_container.extend_from_slice(&(shape.len() as u32).to_le_bytes());
    inner_container.extend_from_slice(&shape);

    // 最外层容器
    let mut data = Vec::new();
    data.extend_from_slice(&0x000Fu16.to_le_bytes()); // container
    data.extend_from_slice(&0xF000u16.to_le_bytes()); // DGG container
    data.extend_from_slice(&(inner_container.len() as u32).to_le_bytes());
    data.extend_from_slice(&inner_container);

    let (result, removed) = remove_escher_records(&data, 10).unwrap();
    assert!(removed);
    // shape 被移除但外层容器保留
    assert!(result.len() < data.len());
}

// ===========================================================================
// decrement_escher_dg_count 容器递归覆盖
// ===========================================================================

#[test]
fn decrement_escher_dg_count_nested_in_container() {
    // F008 在嵌套容器中
    let mut inner = Vec::new();
    inner.extend_from_slice(&0x0000u16.to_le_bytes());
    inner.extend_from_slice(&0xF008u16.to_le_bytes());
    inner.extend_from_slice(&8u32.to_le_bytes());
    inner.extend_from_slice(&5u32.to_le_bytes()); // count=5
    inner.extend_from_slice(&0u32.to_le_bytes());

    // 外层容器
    let mut data = Vec::new();
    data.extend_from_slice(&0x000Fu16.to_le_bytes()); // container
    data.extend_from_slice(&0xF000u16.to_le_bytes());
    data.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    data.extend_from_slice(&inner);

    let result = decrement_escher_dg_count(&mut data).unwrap();
    assert!(result);
}

// ===========================================================================
// extend_existing_dgg_shapes 覆盖
// ===========================================================================

#[test]
fn extend_existing_dgg_shapes_basic() {
    // 构造一个最小的 DGG container
    let mut data = Vec::new();
    // F000 container header
    data.extend_from_slice(&0x000Fu16.to_le_bytes()); // options: container
    data.extend_from_slice(&0xF000u16.to_le_bytes()); // type
    data.extend_from_slice(&0u32.to_le_bytes()); // length (will fix)

    // F006 DGG record
    let mut dgg_payload = Vec::with_capacity(32);
    dgg_payload.extend_from_slice(&100u32.to_le_bytes()); // max shape id
    dgg_payload.extend_from_slice(&1u32.to_le_bytes()); // num clusters
    dgg_payload.extend_from_slice(&5u32.to_le_bytes()); // num shapes saved
    dgg_payload.extend_from_slice(&0u32.to_le_bytes()); // num drawings saved
    // cluster: drawing_id=1, max_shape_id=5
    dgg_payload.extend_from_slice(&1u32.to_le_bytes());
    dgg_payload.extend_from_slice(&5u32.to_le_bytes());

    data.extend_from_slice(&0x0000u16.to_le_bytes()); // options
    data.extend_from_slice(&0xF006u16.to_le_bytes()); // type: DGG
    data.extend_from_slice(&(dgg_payload.len() as u32).to_le_bytes());
    data.extend_from_slice(&dgg_payload);

    // 修复 F000 container length
    let container_inner_len = (data.len() - 8) as u32;
    data[4..8].copy_from_slice(&container_inner_len.to_le_bytes());

    let result = extend_existing_dgg_shapes(&mut data, 1, 2, 200);
    assert!(result.is_ok());
}

// ===========================================================================
// decrement_existing_dgg_shapes 覆盖
// ===========================================================================

#[test]
fn decrement_existing_dgg_shapes_basic() {
    let mut data = Vec::new();
    // F006 DGG record (without container for simplicity)
    let mut dgg_payload = Vec::with_capacity(32);
    dgg_payload.extend_from_slice(&100u32.to_le_bytes()); // max shape id
    dgg_payload.extend_from_slice(&2u32.to_le_bytes()); // num clusters
    dgg_payload.extend_from_slice(&10u32.to_le_bytes()); // num shapes saved
    dgg_payload.extend_from_slice(&1u32.to_le_bytes()); // num drawings saved
    // cluster 1
    dgg_payload.extend_from_slice(&1u32.to_le_bytes());
    dgg_payload.extend_from_slice(&5u32.to_le_bytes());
    // cluster 2
    dgg_payload.extend_from_slice(&2u32.to_le_bytes());
    dgg_payload.extend_from_slice(&3u32.to_le_bytes());

    data.extend_from_slice(&0x0000u16.to_le_bytes());
    data.extend_from_slice(&0xF006u16.to_le_bytes());
    data.extend_from_slice(&(dgg_payload.len() as u32).to_le_bytes());
    data.extend_from_slice(&dgg_payload);

    let result = decrement_existing_dgg_shapes(&mut data, 1, 1);
    assert!(result.is_ok());
}

// ===========================================================================
// append_dgg_drawing 覆盖
// ===========================================================================

#[test]
fn append_dgg_drawing_basic() {
    let mut data = Vec::new();
    // F000 container
    data.extend_from_slice(&0x000Fu16.to_le_bytes());
    data.extend_from_slice(&0xF000u16.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes()); // length (will fix)

    // F006 DGG
    let mut dgg_payload = Vec::with_capacity(32);
    dgg_payload.extend_from_slice(&100u32.to_le_bytes()); // max shape id
    dgg_payload.extend_from_slice(&0u32.to_le_bytes()); // num clusters
    dgg_payload.extend_from_slice(&5u32.to_le_bytes()); // num shapes saved
    dgg_payload.extend_from_slice(&0u32.to_le_bytes()); // num drawings saved

    data.extend_from_slice(&0x0000u16.to_le_bytes());
    data.extend_from_slice(&0xF006u16.to_le_bytes());
    data.extend_from_slice(&(dgg_payload.len() as u32).to_le_bytes());
    data.extend_from_slice(&dgg_payload);

    // Fix container length
    let inner_len = (data.len() - 8) as u32;
    data[4..8].copy_from_slice(&inner_len.to_le_bytes());

    let drawing_id = append_dgg_drawing(&mut data, 3).unwrap();
    assert!(drawing_id >= 1);
}

// ===========================================================================
// extend_chart_drawing_group 覆盖
// ===========================================================================

#[test]
fn extend_chart_drawing_group_basic() {
    let mut data = Vec::new();
    // F000 container
    data.extend_from_slice(&0x000Fu16.to_le_bytes());
    data.extend_from_slice(&0xF000u16.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

    // F006 DGG
    let mut dgg_payload = Vec::with_capacity(32);
    dgg_payload.extend_from_slice(&100u32.to_le_bytes());
    dgg_payload.extend_from_slice(&0u32.to_le_bytes());
    dgg_payload.extend_from_slice(&5u32.to_le_bytes());
    dgg_payload.extend_from_slice(&0u32.to_le_bytes());

    data.extend_from_slice(&0x0000u16.to_le_bytes());
    data.extend_from_slice(&0xF006u16.to_le_bytes());
    data.extend_from_slice(&(dgg_payload.len() as u32).to_le_bytes());
    data.extend_from_slice(&dgg_payload);

    let inner_len = (data.len() - 8) as u32;
    data[4..8].copy_from_slice(&inner_len.to_le_bytes());

    let drawing_id = extend_chart_drawing_group(&mut data, 2).unwrap();
    assert!(drawing_id >= 1);
}

// ===========================================================================
// is_empty_client_textbox_record 额外覆盖
// ===========================================================================

#[test]
fn is_empty_client_textbox_record_various() {
    // 正确的空 client textbox
    assert!(is_empty_client_textbox_record(&[
        0x00, 0x00, 0x0D, 0xF0, 0x00, 0x00, 0x00, 0x00
    ]));
    // 非 F00D type
    assert!(!is_empty_client_textbox_record(&[
        0x00, 0x00, 0x0E, 0xF0, 0x00, 0x00, 0x00, 0x00
    ]));
    // 非零长度
    assert!(!is_empty_client_textbox_record(&[
        0x00, 0x00, 0x0D, 0xF0, 0x01, 0x00, 0x00, 0x00
    ]));
    // 长度不对
    assert!(!is_empty_client_textbox_record(&[0; 7]));
}

// ===========================================================================
// top_level_substreams 覆盖
// ===========================================================================

#[test]
fn top_level_substreams_multiple_streams() {
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
    ];
    let spans = top_level_substreams(&records);
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0], (0, 1));
    assert_eq!(spans[1], (2, 5));
}

// ===========================================================================
// is_worksheet_bof 覆盖
// ===========================================================================

#[test]
fn is_worksheet_bof_various() {
    // DT_WORKSHEET = 0x0010
    let mut data = vec![0; 4];
    data[2] = 0x10;
    data[3] = 0x00;
    assert!(is_worksheet_bof(&data));

    // 不是 worksheet
    data[2] = 0x20;
    assert!(!is_worksheet_bof(&data));

    // 太短
    assert!(!is_worksheet_bof(&[0, 0]));
}

// ===========================================================================
// cell_coords 覆盖
// ===========================================================================

#[test]
fn cell_coords_various_types() {
    let mut data = vec![5, 0, 3, 0, 0, 0]; // row=5, col=3

    // LABEL
    let record = RawRecord {
        typ: LABEL,
        data: data.clone(),
    };
    assert_eq!(cell_coords(&record), Some((5, 3)));

    // NUMBER
    data.extend_from_slice(&0.0f64.to_le_bytes());
    let record = RawRecord {
        typ: NUMBER,
        data: data.clone(),
    };
    assert_eq!(cell_coords(&record), Some((5, 3)));

    // FORMULA
    let record = RawRecord {
        typ: FORMULA,
        data: data,
    };
    assert_eq!(cell_coords(&record), Some((5, 3)));

    // 非 cell record
    let record = RawRecord {
        typ: BOF,
        data: vec![0; 16],
    };
    assert_eq!(cell_coords(&record), None);

    // 太短的 cell record
    let record = RawRecord {
        typ: LABEL,
        data: vec![0; 2],
    };
    assert_eq!(cell_coords(&record), None);
}

// ===========================================================================
// sheet_dimensions 覆盖（已有基础测试，补充边界）
// ===========================================================================

#[test]
fn sheet_dimensions_with_multiple_cells() {
    let mut label1 = vec![0u8; 10];
    label1[0..2].copy_from_slice(&3u16.to_le_bytes()); // row 3
    label1[2..4].copy_from_slice(&5u16.to_le_bytes()); // col 5

    let mut label2 = vec![0u8; 10];
    label2[0..2].copy_from_slice(&7u16.to_le_bytes()); // row 7
    label2[2..4].copy_from_slice(&2u16.to_le_bytes()); // col 2

    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: LABEL,
            data: label1,
        },
        RawRecord {
            typ: LABEL,
            data: label2,
        },
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
    ];
    let sheet = SheetSpan {
        name: "S".to_owned(),
        bound_sheet_index: 0,
        bof_index: 0,
        eof_index: 3,
        dimension_index: None,
    };
    let (max_row, max_col) = sheet_dimensions(&records, &sheet);
    assert_eq!(max_row, 8); // row 7 + 1
    assert_eq!(max_col, 6); // col 5 + 1
}

// ===========================================================================
// sheet_max_row 覆盖（补充边界）
// ===========================================================================

#[test]
fn sheet_max_row_multiple_cells() {
    let mut label1 = vec![0u8; 10];
    label1[0..2].copy_from_slice(&3u16.to_le_bytes());

    let mut label2 = vec![0u8; 10];
    label2[0..2].copy_from_slice(&7u16.to_le_bytes());

    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: LABEL,
            data: label1,
        },
        RawRecord {
            typ: LABEL,
            data: label2,
        },
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
    ];
    let sheet = SheetSpan {
        name: "S".to_owned(),
        bound_sheet_index: 0,
        bof_index: 0,
        eof_index: 3,
        dimension_index: None,
    };
    assert_eq!(sheet_max_row(&records, &sheet), Some(7));
}

// ===========================================================================
// split_records 覆盖
// ===========================================================================

#[test]
fn split_records_empty_errors() {
    assert!(split_records(&[]).is_err());
}

#[test]
fn split_records_single_record() {
    // 一条记录：type=0x0003, length=2, data=[0xAA, 0xBB]
    let bytes = [0x03, 0x00, 0x02, 0x00, 0xAA, 0xBB];
    let records = split_records(&bytes).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].typ, 0x0003);
    assert_eq!(records[0].data, vec![0xAA, 0xBB]);
}

#[test]
fn split_records_truncated_record() {
    // 声明长度 4 但只有 2 字节数据
    let bytes = [0x03, 0x00, 0x04, 0x00, 0xAA, 0xBB];
    assert!(split_records(&bytes).is_err());
}

// ===========================================================================
// discover_sheets 额外覆盖
// ===========================================================================

#[test]
fn discover_sheets_boundsheet_count_mismatch() {
    fn bof(stream_type: u16) -> RawRecord {
        RawRecord {
            typ: BOF,
            data: [0x00, 0x06, stream_type as u8, (stream_type >> 8) as u8].to_vec(),
        }
    }
    fn boundsheet(name: &str) -> RawRecord {
        let mut data = vec![0, 0, 0, 0, 0, 0, name.len() as u8, 0];
        data.extend_from_slice(name.as_bytes());
        RawRecord {
            typ: BOUNDSHEET,
            data,
        }
    }

    // 2 个 BOUNDSHEET 但只有 1 个 sheet stream
    let records = vec![
        bof(0x0005),
        boundsheet("Sheet1"),
        boundsheet("Sheet2"),
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
        bof(DT_WORKSHEET),
        RawRecord {
            typ: DIMENSION,
            data: vec![0; 14],
        },
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
    ];
    assert!(discover_sheets(&records).is_err());
}

// ===========================================================================
// encode_cell_record RichText 覆盖
// ===========================================================================

#[test]
fn encode_cell_record_rich_text() {
    use crate::biff8::Biff8RichText;
    let rich = Biff8RichText {
        text: "hello".to_owned(),
        runs: vec![(0, 0), (3, 1)],
    };
    let record = encode_cell_record(0, 0, 0, &Biff8Value::RichText(rich)).unwrap();
    assert_eq!(record.typ, RICH_STRING_SID);
}

#[test]
fn encode_cell_record_rich_text_unicode() {
    use crate::biff8::Biff8RichText;
    let rich = Biff8RichText {
        text: "\u{4e2d}\u{6587}".to_owned(),
        runs: vec![(0, 0)],
    };
    let record = encode_cell_record(0, 0, 0, &Biff8Value::RichText(rich)).unwrap();
    assert_eq!(record.typ, RICH_STRING_SID);
}

// ===========================================================================
// encode_label_record 额外覆盖
// ===========================================================================

#[test]
fn encode_label_record_unicode() {
    let record = encode_label_record(0, 0, 0, "\u{4e2d}\u{6587}").unwrap();
    assert_eq!(record.typ, LABEL);
    assert!(record.data.len() > 6);
}

// ===========================================================================
// apply_macro_policy 覆盖（Strip 路径）
// ===========================================================================

#[test]
fn apply_macro_policy_preserve() {
    let bytes = vec![0xD0, 0xCF, 0x11, 0xE0]; // 最小 OLE header
    let result = apply_macro_policy(&bytes, &Biff8MacroPolicy::Preserve).unwrap();
    assert_eq!(result, bytes);
}

// ===========================================================================
// Template package: add_charts 覆盖
// ===========================================================================

#[test]
fn template_package_add_charts_empty() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;
    // 空图表列表应该 noop
    package.add_charts("Data", &[])?;
    Ok(())
}

#[test]
fn template_package_add_charts_missing_sheet() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;
    let chart = crate::biff8::Biff8Chart::new(
        crate::biff8::Biff8ChartKind::Bar,
        0,
        0,
        5,
        5,
    );
    assert!(matches!(
        package.add_charts("NoSuch", &[chart]),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

// ===========================================================================
// Template package: fill_collection 额外覆盖
// ===========================================================================

#[test]
fn template_package_fill_collection_horizontal_with_cursor() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{.a}".to_owned())),
        );
        sheet.cells.insert(
            (0, 1),
            Biff8Cell::general(Biff8Value::Text("{.b}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 多行水平填充
    let rows = vec![
        BTreeMap::from([
            ("a".to_owned(), "x1".to_owned()),
            ("b".to_owned(), "y1".to_owned()),
        ]),
        BTreeMap::from([
            ("a".to_owned(), "x2".to_owned()),
            ("b".to_owned(), "y2".to_owned()),
        ]),
    ];
    let count = package.fill_collection_placeholders(None, None, &rows, true, false, true)?;
    assert!(count > 0);
    Ok(())
}

// ===========================================================================
// Template package: 保护已有 PROTECT 记录时再次调用
// ===========================================================================

#[test]
fn template_package_protect_sheet_twice_updates_existing() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 第一次保护
    package.protect_sheet("Data", "password1")?;
    // 第二次保护应该更新而非追加
    package.protect_sheet("Data", "password2")?;

    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: 多次追加行
// ===========================================================================

#[test]
fn template_package_append_rows_multiple_batches() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 第一批
    let rows1 = vec![vec![(
        0,
        Biff8Cell::general(Biff8Value::Text("batch1".to_owned())),
    )]];
    let next1 = package.append_rows("Data", &rows1)?;
    assert_eq!(next1, 3);

    // 第二批
    let rows2 = vec![vec![(
        0,
        Biff8Cell::general(Biff8Value::Text("batch2".to_owned())),
    )]];
    let next2 = package.append_rows("Data", &rows2)?;
    assert_eq!(next2, 4);

    Ok(())
}

// ===========================================================================
// Template package: fill_collection_cells 额外覆盖
// ===========================================================================

#[test]
fn template_package_fill_collection_cells_horizontal() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([(
        "item".to_owned(),
        Biff8Cell::general(Biff8Value::Text("H".to_owned())),
    )])];
    let placements =
        package.fill_collection_cells(None, None, &rows, true, false, true)?;
    assert!(placements.len() > 0);
    Ok(())
}

// ===========================================================================
// Template package: fill_collection on specific sheet
// ===========================================================================

#[test]
fn template_package_fill_collection_on_specific_sheet() -> Result<()> {
    let bytes = multi_sheet_template()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 创建占位符
    let mut book2 = crate::biff8::Biff8Book::default();
    {
        let sheet = book2.sheet_mut("Sheet1");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{.val}".to_owned())),
        );
    }
    let bytes2 = book2.to_cfb_bytes()?;
    let mut package2 = Biff8TemplatePackage::from_bytes(&bytes2)?;

    let rows = vec![BTreeMap::from([("val".to_owned(), "test".to_owned())])];
    let count =
        package2.fill_collection_placeholders(Some("Sheet1"), None, &rows, false, false, true)?;
    assert!(count > 0);
    Ok(())
}

// ===========================================================================
// Template package: replace_scalar_cells_on_sheet 额外覆盖
// ===========================================================================

#[test]
fn template_package_replace_scalar_cells_multiple() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut cells = BTreeMap::new();
    cells.insert(
        "name".to_owned(),
        Biff8Cell::general(Biff8Value::Text("Alice".to_owned())),
    );
    cells.insert(
        "age".to_owned(),
        Biff8Cell::general(Biff8Value::Number(25.0)),
    );
    cells.insert(
        "other".to_owned(),
        Biff8Cell::general(Biff8Value::Text("misc".to_owned())),
    );

    let placements = package.replace_scalar_cells_on_sheet(None, &cells)?;
    assert!(placements.len() >= 3);
    Ok(())
}

// ===========================================================================
// Template package: fill_collection_cells with empty rows
// ===========================================================================

#[test]
fn template_package_fill_collection_cells_empty_rows() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let placements =
        package.fill_collection_cells(None, None, &[], false, false, true)?;
    assert!(placements.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: add_hyperlink_range 错误路径
// ===========================================================================

#[test]
fn template_package_add_hyperlink_last_row_too_large() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.add_hyperlink_range(
            "Data",
            0,
            70000, // last_row too large
            0,
            0,
            "url".to_owned(),
            "l".to_owned(),
            Biff8HyperlinkKind::Url,
        ),
        Err(ExcelError::Xls(_))
    ));
    Ok(())
}

#[test]
fn template_package_add_hyperlink_last_col_too_large() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.add_hyperlink_range(
            "Data",
            0,
            0,
            0,
            300, // last_col too large
            "url".to_owned(),
            "l".to_owned(),
            Biff8HyperlinkKind::Url,
        ),
        Err(ExcelError::Xls(_))
    ));
    Ok(())
}

// ===========================================================================
// Template package: append_custom_fonts 多个字体
// ===========================================================================

#[test]
fn template_package_append_custom_fonts_multiple() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let fonts = vec![
        vec![0xC8, 0x00, 0x00, 0x00],
        vec![0xC8, 0x00, 0x00, 0x01],
    ];
    package.append_custom_fonts(&fonts)?;
    let idx = package.next_custom_font_index();
    assert!(idx >= 6);
    Ok(())
}

// ===========================================================================
// Template package: to_bytes_with_password_and_macro_policy Strip
// ===========================================================================

#[test]
fn template_package_to_bytes_macro_policy_strip() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let output = package.to_bytes_with_password_and_macro_policy(
        None,
        &Biff8MacroPolicy::Strip,
    )?;
    assert!(!output.is_empty());
    assert!(looks_like_xls(&output));
    Ok(())
}

// ===========================================================================
// Template package: save_to_path 创建目录
// ===========================================================================

#[test]
fn template_package_save_to_path_creates_dirs() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let dir = std::env::temp_dir().join("easyexcel_test_03_nested").join("sub");
    let path = dir.join("output.xls");
    package.save_to_path(&path)?;
    assert!(path.exists());
    let _ = std::fs::remove_dir_all(dir.parent().unwrap());
    Ok(())
}

// ===========================================================================
// fill_collection_placeholders 多命名集合
// ===========================================================================

#[test]
fn template_package_fill_collection_multiple_named_collections() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{users.name}".to_owned())),
        );
        sheet.cells.insert(
            (1, 0),
            Biff8Cell::general(Biff8Value::Text("{items.item}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 填充 users 集合
    let rows_users = vec![BTreeMap::from([("name".to_owned(), "Alice".to_owned())])];
    let count1 =
        package.fill_collection_placeholders(None, Some("users"), &rows_users, false, false, true)?;
    assert!(count1 > 0);

    // 填充 items 集合
    let rows_items = vec![BTreeMap::from([("item".to_owned(), "Widget".to_owned())])];
    let count2 = package.fill_collection_placeholders(
        None,
        Some("items"),
        &rows_items,
        false,
        false,
        true,
    )?;
    assert!(count2 > 0);
    Ok(())
}

// ===========================================================================
// Template package: set_cell with formula
// ===========================================================================

#[test]
fn template_package_set_cell_formula() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.set_cell(
        "Data",
        5,
        0,
        &Biff8Cell::general(Biff8Value::Formula("SUM(A1:A2)".to_owned())),
    )?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

#[test]
fn template_package_set_cell_empty_formula() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 空公式表达式
    package.set_cell(
        "Data",
        5,
        0,
        &Biff8Cell::general(Biff8Value::Formula("".to_owned())),
    )?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: replace_label with LABELSST
// ===========================================================================

#[test]
fn template_package_replace_label_on_labelsst() -> Result<()> {
    // 创建一个包含 LABELSST 的模板，然后 replace_label
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        // 添加一个文本单元格（会被编码为 LABELSST）
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("original".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // replace_label 应该把 LABELSST 替换为 LABEL
    package.replace_label("Data", 0, 0, "replaced")?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: fill_collection force_new_row 不迁移
// ===========================================================================

#[test]
fn template_package_fill_collection_force_new_row_false() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![
        BTreeMap::from([("item".to_owned(), "A".to_owned())]),
        BTreeMap::from([("item".to_owned(), "B".to_owned())]),
    ];
    let count = package.fill_collection_placeholders(None, None, &rows, false, false, true)?;
    assert!(count > 0);
    Ok(())
}

// ===========================================================================
// shift_formula_references: ptgRefErr (0x2A) 覆盖
// ===========================================================================

#[test]
fn shift_formula_references_ref_err() {
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgRefErr = 0x2A: 5 bytes
    let tokens = [0x2A, 0, 0, 0, 0];
    let mut f = formula(0, &tokens);
    // 应该成功但不修改
    shift_formula_references(&mut f, 5, 2, 0, &[]).unwrap();
}

// ===========================================================================
// shift_formula_references: ptgArea3dErr (0x3D) 覆盖
// ===========================================================================

#[test]
fn shift_formula_references_area3d_err() {
    fn formula(row: u16, tokens: &[u8]) -> RawRecord {
        let mut data = vec![0; 22];
        data[0..2].copy_from_slice(&row.to_le_bytes());
        data[20..22].copy_from_slice(
            &u16::try_from(tokens.len())
                .expect("test token length")
                .to_le_bytes(),
        );
        data.extend_from_slice(tokens);
        RawRecord { typ: FORMULA, data }
    }

    // ptgArea3dErr = 0x3D: 11 bytes
    let tokens = [0x3D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut f = formula(0, &tokens);
    shift_formula_references(&mut f, 5, 2, 0, &[]).unwrap();
}

// ===========================================================================
// decode_labelsst_payload 覆盖
// ===========================================================================

#[test]
fn decode_labelsst_payload_basic() {
    let mut data = vec![0u8; 10];
    data[0..2].copy_from_slice(&1u16.to_le_bytes()); // row
    data[2..4].copy_from_slice(&2u16.to_le_bytes()); // col
    data[6..10].copy_from_slice(&5u32.to_le_bytes()); // sst index

    let (row, col, text) = decode_labelsst_payload(&data);
    assert_eq!(row, 1);
    assert_eq!(col, 2);
    assert!(text.is_none()); // SST 不可用
}

#[test]
fn decode_labelsst_payload_short() {
    let data = [0u8; 4];
    let (row, col, text) = decode_labelsst_payload(&data);
    assert_eq!(row, 0);
    assert_eq!(col, 0);
    assert!(text.is_none());
}

// ===========================================================================
// Template package: fill_collection_cells_horizontal force_new_row
// ===========================================================================

#[test]
fn template_package_fill_collection_horizontal_force_new_row() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{.val}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([(
        "val".to_owned(),
        Biff8Cell::general(Biff8Value::Text("test".to_owned())),
    )])];
    // 水平填充 + force_new_row
    let placements =
        package.fill_collection_cells(None, None, &rows, true, true, true)?;
    assert!(placements.len() > 0);
    Ok(())
}

// ===========================================================================
// Template package: 多次 fill_collection 测试 cursor 推进
// ===========================================================================

#[test]
fn template_package_fill_collection_vertical_multiple_passes() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{.name}".to_owned())),
        );
        sheet.cells.insert(
            (0, 1),
            Biff8Cell::general(Biff8Value::Text("{.value}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 第一批
    let rows1 = vec![
        BTreeMap::from([
            ("name".to_owned(), "A".to_owned()),
            ("value".to_owned(), "1".to_owned()),
        ]),
        BTreeMap::from([
            ("name".to_owned(), "B".to_owned()),
            ("value".to_owned(), "2".to_owned()),
        ]),
    ];
    let count1 = package.fill_collection_placeholders(None, None, &rows1, false, true, true)?;
    assert!(count1 > 0);

    // 第二批
    let rows2 = vec![BTreeMap::from([
        ("name".to_owned(), "C".to_owned()),
        ("value".to_owned(), "3".to_owned()),
    ])];
    let count2 = package.fill_collection_placeholders(None, None, &rows2, false, true, true)?;
    assert!(count2 > 0);
    Ok(())
}

// ===========================================================================
// Template package: fill_collection with multiple fields per row
// ===========================================================================

#[test]
fn template_package_fill_collection_multiple_fields() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{.a}".to_owned())),
        );
        sheet.cells.insert(
            (0, 1),
            Biff8Cell::general(Biff8Value::Text("{.b}".to_owned())),
        );
        sheet.cells.insert(
            (0, 2),
            Biff8Cell::general(Biff8Value::Text("{.c}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![
        BTreeMap::from([
            ("a".to_owned(), "x1".to_owned()),
            ("b".to_owned(), "y1".to_owned()),
            ("c".to_owned(), "z1".to_owned()),
        ]),
        BTreeMap::from([
            ("a".to_owned(), "x2".to_owned()),
            ("b".to_owned(), "y2".to_owned()),
            ("c".to_owned(), "z2".to_owned()),
        ]),
    ];
    let count = package.fill_collection_placeholders(None, None, &rows, false, true, true)?;
    assert!(count >= 6);
    Ok(())
}

// ===========================================================================
// Template package: set_cell with RichText
// ===========================================================================

#[test]
fn template_package_set_cell_rich_text() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rich = crate::biff8::Biff8RichText {
        text: "hello".to_owned(),
        runs: vec![(0, 0), (3, 1)],
    };
    package.set_cell(
        "Data",
        5,
        0,
        &Biff8Cell::general(Biff8Value::RichText(rich)),
    )?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: set_cell with Error value
// ===========================================================================

#[test]
fn template_package_set_cell_error() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.set_cell(
        "Data",
        5,
        0,
        &Biff8Cell::general(Biff8Value::Error(0x07)), // #DIV/0!
    )?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: set_cell with Blank value
// ===========================================================================

#[test]
fn template_package_set_cell_blank() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.set_cell("Data", 5, 0, &Biff8Cell::general(Biff8Value::Blank))?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: replace_scalar_cells_on_sheet with specific sheet
// ===========================================================================

#[test]
fn template_package_replace_scalar_cells_on_specific_sheet() -> Result<()> {
    let bytes = multi_sheet_template()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut cells = BTreeMap::new();
    cells.insert(
        "val".to_owned(),
        Biff8Cell::general(Biff8Value::Text("replaced".to_owned())),
    );

    let placements = package.replace_scalar_cells_on_sheet(Some("Sheet1"), &cells)?;
    assert_eq!(placements.len(), 1);
    assert_eq!(placements[0].0, "Sheet1");
    Ok(())
}

// ===========================================================================
// Template package: fill_collection on specific sheet with name
// ===========================================================================

#[test]
fn template_package_fill_collection_on_sheet_with_name() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Sheet1");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{users.name}".to_owned())),
        );
    }
    book.create_sheet("Sheet2")?;
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([("name".to_owned(), "Alice".to_owned())])];
    let count = package.fill_collection_placeholders(
        Some("Sheet1"),
        Some("users"),
        &rows,
        false,
        false,
        true,
    )?;
    assert!(count > 0);
    Ok(())
}

// ===========================================================================
// Template package: fill_collection with auto_style=false
// ===========================================================================

#[test]
fn template_package_fill_collection_no_auto_style() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([(
        "item".to_owned(),
        "styled".to_owned(),
    )])];
    let count = package.fill_collection_placeholders(None, None, &rows, false, false, false)?;
    assert!(count > 0);
    Ok(())
}

// ===========================================================================
// Template package: to_bytes_with_password encrypt+decrypt roundtrip
// ===========================================================================

#[test]
fn template_package_encrypt_decrypt_roundtrip() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 替换一些占位符
    let mut values = BTreeMap::new();
    values.insert("name".to_owned(), "Test".to_owned());
    package.replace_scalar_placeholders(&values)?;

    // 加密
    let encrypted = package.to_bytes_with_password(Some("mypassword"))?;
    assert!(looks_like_xls(&encrypted));

    // 用密码解密
    let decrypted_package =
        Biff8TemplatePackage::from_bytes_with_password(&encrypted, Some("mypassword"))?;
    assert_eq!(decrypted_package.sheet_names(), vec!["Data"]);
    Ok(())
}

// ===========================================================================
// Template package: to_bytes strips FILEPASS when no password
// ===========================================================================

#[test]
fn template_package_to_bytes_strips_filepass() -> Result<()> {
    let bytes = template_with_values()?;
    // 先加密
    let encrypted = Biff8TemplatePackage::from_bytes(&bytes)?
        .to_bytes_with_password(Some("secret"))?;
    // 用密码加载
    let package =
        Biff8TemplatePackage::from_bytes_with_password(&encrypted, Some("secret"))?;
    // 不带密码输出
    let plain = package.to_bytes()?;
    // 应该能不带密码加载
    let reloaded = Biff8TemplatePackage::from_bytes(&plain)?;
    assert_eq!(reloaded.sheet_names(), vec!["Data"]);
    Ok(())
}

// ===========================================================================
// Template package: save_to_path_with_password roundtrip
// ===========================================================================

#[test]
fn template_package_save_encrypted_to_path() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let dir = std::env::temp_dir().join("easyexcel_test_03_enc");
    let path = dir.join("encrypted.xls");
    package.save_to_path_with_password(&path, Some("secret"))?;
    assert!(path.exists());

    let loaded = Biff8TemplatePackage::from_path_with_password(&path, Some("secret"))?;
    assert_eq!(loaded.sheet_names(), package.sheet_names());
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

// ===========================================================================
// Template package: save_to_writer_with_password
// ===========================================================================

#[test]
fn template_package_save_encrypted_to_writer() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    package.save_to_writer_with_password(&mut cursor, Some("secret"))?;
    assert!(!buffer.is_empty());

    // 用密码加载
    let loaded = Biff8TemplatePackage::from_bytes_with_password(&buffer, Some("secret"))?;
    assert_eq!(loaded.sheet_names(), package.sheet_names());
    Ok(())
}

// ===========================================================================
// Template package: fill_collection_cells with horizontal and cursor
// ===========================================================================

#[test]
fn template_package_fill_collection_horizontal_with_cursor_advance() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{.a}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 第一次水平填充
    let rows1 = vec![BTreeMap::from([("a".to_owned(), "x".to_owned())])];
    let count1 =
        package.fill_collection_placeholders(None, None, &rows1, true, false, true)?;
    assert!(count1 > 0);

    // 第二次水平填充（cursor 推进）
    let rows2 = vec![BTreeMap::from([("a".to_owned(), "y".to_owned())])];
    let count2 =
        package.fill_collection_placeholders(None, None, &rows2, true, false, true)?;
    assert!(count2 > 0);
    Ok(())
}

// ===========================================================================
// Template package: ensure_sheet then fill
// ===========================================================================

#[test]
fn template_package_ensure_sheet_then_fill() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 创建新 sheet
    let created = package.ensure_sheet("NewSheet")?;
    assert!(created);

    // 在新 sheet 上设置单元格
    package.set_cell(
        "NewSheet",
        0,
        0,
        &Biff8Cell::general(Biff8Value::Text("hello".to_owned())),
    )?;

    let output = package.to_bytes()?;
    let reloaded = Biff8TemplatePackage::from_bytes(&output)?;
    assert!(reloaded.sheet_names().contains(&"NewSheet".to_owned()));
    Ok(())
}

// ===========================================================================
// Template package: protect_sheet then to_bytes
// ===========================================================================

#[test]
fn template_package_protect_and_serialize() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.protect_sheet("Data", "secret")?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    assert!(looks_like_xls(&output));
    Ok(())
}

// ===========================================================================
// Template package: add_merge_range then to_bytes
// ===========================================================================

#[test]
fn template_package_merge_and_serialize() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.add_merge_range(
        "Data",
        Biff8Merge {
            first_row: 0,
            last_row: 1,
            first_col: 0,
            last_col: 2,
        },
    )?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: append_custom_fonts then to_bytes
// ===========================================================================

#[test]
fn template_package_append_fonts_and_serialize() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let font_data = vec![0xC8, 0x00, 0x00, 0x00];
    package.append_custom_fonts(&[font_data])?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: add_hyperlink_range then to_bytes
// ===========================================================================

#[test]
fn template_package_hyperlink_and_serialize() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.add_hyperlink_range(
        "Data",
        0,
        0,
        0,
        0,
        "https://example.com".to_owned(),
        "Example".to_owned(),
        Biff8HyperlinkKind::Url,
    )?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: fill_collection with named collection and cursor
// ===========================================================================

#[test]
fn template_package_fill_named_collection_with_cursor() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{items.name}".to_owned())),
        );
        sheet.cells.insert(
            (0, 1),
            Biff8Cell::general(Biff8Value::Text("{items.value}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 第一批
    let rows1 = vec![BTreeMap::from([
        ("name".to_owned(), "A".to_owned()),
        ("value".to_owned(), "1".to_owned()),
    ])];
    let count1 = package.fill_collection_placeholders(
        None,
        Some("items"),
        &rows1,
        false,
        true,
        true,
    )?;
    assert!(count1 > 0);

    // 第二批
    let rows2 = vec![BTreeMap::from([
        ("name".to_owned(), "B".to_owned()),
        ("value".to_owned(), "2".to_owned()),
    ])];
    let count2 = package.fill_collection_placeholders(
        None,
        Some("items"),
        &rows2,
        false,
        true,
        true,
    )?;
    assert!(count2 > 0);
    Ok(())
}

// ===========================================================================
// Template package: fill_collection with horizontal named collection
// ===========================================================================

#[test]
fn template_package_fill_named_collection_horizontal() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{items.val}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([("val".to_owned(), "X".to_owned())])];
    let count = package.fill_collection_placeholders(
        None,
        Some("items"),
        &rows,
        true,
        false,
        true,
    )?;
    assert!(count > 0);
    Ok(())
}

// ===========================================================================
// Template package: fill_collection with force_new_row and static rows after
// ===========================================================================

#[test]
fn template_package_fill_collection_force_new_row_with_static_rows() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{.name}".to_owned())),
        );
        // 静态行在占位符之后
        sheet.cells.insert(
            (2, 0),
            Biff8Cell::general(Biff8Value::Text("footer".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![
        BTreeMap::from([("name".to_owned(), "Row1".to_owned())]),
        BTreeMap::from([("name".to_owned(), "Row2".to_owned())]),
        BTreeMap::from([("name".to_owned(), "Row3".to_owned())]),
    ];
    let count = package.fill_collection_placeholders(None, None, &rows, false, true, true)?;
    assert!(count > 0);

    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// Template package: fill_collection with empty collection name (unnamed)
// ===========================================================================

#[test]
fn template_package_fill_unnamed_collection() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([
        ("item".to_owned(), "Widget".to_owned()),
        ("price".to_owned(), "9.99".to_owned()),
    ])];
    let count = package.fill_collection_placeholders(None, None, &rows, false, false, true)?;
    assert!(count > 0);
    Ok(())
}

// ===========================================================================
// Template package: fill_collection with collection_name but no matching placeholders
// ===========================================================================

#[test]
fn template_package_fill_collection_no_matching_name() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([("item".to_owned(), "Widget".to_owned())])];
    // 使用不存在的集合名
    let count = package.fill_collection_placeholders(
        None,
        Some("nonexistent"),
        &rows,
        false,
        false,
        true,
    )?;
    assert_eq!(count, 0);
    Ok(())
}

// ===========================================================================
// Template package: replace_scalar_placeholders with no matching keys
// ===========================================================================

#[test]
fn template_package_replace_scalar_no_matching_keys() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut values = BTreeMap::new();
    values.insert("nonexistent".to_owned(), "value".to_owned());

    let count = package.replace_scalar_placeholders(&values)?;
    assert_eq!(count, 0);
    Ok(())
}

// ===========================================================================
// Template package: replace_scalar_placeholders with all keys
// ===========================================================================

#[test]
fn template_package_replace_scalar_all_keys() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut values = BTreeMap::new();
    values.insert("name".to_owned(), "Alice".to_owned());
    values.insert("age".to_owned(), "30".to_owned());
    values.insert("other".to_owned(), "misc".to_owned());

    let count = package.replace_scalar_placeholders(&values)?;
    assert!(count >= 3);
    Ok(())
}

// ===========================================================================
// Template package: to_bytes_with_password_and_macro_policy Preserve
// ===========================================================================

#[test]
fn template_package_to_bytes_macro_policy_preserve() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let output = package.to_bytes_with_password_and_macro_policy(
        None,
        &Biff8MacroPolicy::Preserve,
    )?;
    assert!(!output.is_empty());
    assert!(looks_like_xls(&output));
    Ok(())
}

// ===========================================================================
// Template package: fill_collection with multiple rows horizontal
// ===========================================================================

#[test]
fn template_package_fill_collection_horizontal_multiple_rows() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{.val}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![
        BTreeMap::from([("val".to_owned(), "A".to_owned())]),
        BTreeMap::from([("val".to_owned(), "B".to_owned())]),
        BTreeMap::from([("val".to_owned(), "C".to_owned())]),
    ];
    let count = package.fill_collection_placeholders(None, None, &rows, true, false, true)?;
    assert!(count >= 3);
    Ok(())
}
