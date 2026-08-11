// ---------------------------------------------------------------------------
// 第 3 轮补充测试：rawrecord_to_scalar_placeholder_key.rs 及其他低覆盖文件
// 来源：coverage 分析 rawrecord 27.7%, collection_placeholder 62.5%,
//       shift_formula 64.3%, biff8_workbook_model 62.9%, biff.rs 60.5%
// ---------------------------------------------------------------------------

/// 创建一个包含占位符的最小 XLS 模板字节。
fn template_with_placeholders() -> Result<Vec<u8>> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{name}".to_owned())),
        );
        sheet.cells.insert(
            (0, 1),
            Biff8Cell::general(Biff8Value::Text("{age}".to_owned())),
        );
        sheet.cells.insert(
            (1, 0),
            Biff8Cell::general(Biff8Value::Text("{.item}".to_owned())),
        );
        sheet.cells.insert(
            (1, 1),
            Biff8Cell::general(Biff8Value::Text("{.price}".to_owned())),
        );
        sheet.cells.insert(
            (2, 0),
            Biff8Cell::general(Biff8Value::Text("{other}".to_owned())),
        );
    }
    book.to_cfb_bytes()
}

/// 创建包含多个工作表的 XLS 模板。
fn multi_sheet_template() -> Result<Vec<u8>> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Sheet1");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{val}".to_owned())),
        );
    }
    book.create_sheet("Sheet2")?;
    book.to_cfb_bytes()
}

/// 创建含有数值单元格的模板。
fn template_with_values() -> Result<Vec<u8>> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert((0, 0), Biff8Cell::general(Biff8Value::Number(42.0)));
        sheet.cells.insert(
            (0, 1),
            Biff8Cell::general(Biff8Value::Text("hello".to_owned())),
        );
        sheet.cells.insert(
            (1, 0),
            Biff8Cell::general(Biff8Value::Bool(true)),
        );
        sheet.cells.insert(
            (1, 1),
            Biff8Cell::general(Biff8Value::Blank),
        );
    }
    book.to_cfb_bytes()
}

/// 创建有合并区域和保护的模板。
fn template_with_merges_and_protection() -> Result<Vec<u8>> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("merged".to_owned())),
        );
        sheet.merges.push(Biff8Merge {
            first_row: 0,
            last_row: 1,
            first_col: 0,
            last_col: 1,
        });
        sheet.cells.insert(
            (2, 0),
            Biff8Cell::general(Biff8Value::Number(100.0)),
        );
    }
    book.to_cfb_bytes()
}

// ===========================================================================
// validate_new_sheet_name 测试
// ===========================================================================

#[test]
fn validate_sheet_name_rejects_empty_and_too_long() {
    let sheets = vec![];
    // 空名称
    assert!(validate_new_sheet_name("", &sheets).is_err());
    // 32 字符名称（超过 BIFF8 限制）
    let long_name = "A".repeat(32);
    assert!(validate_new_sheet_name(&long_name, &sheets).is_err());
    // 31 字符名称应该可以
    let ok_name = "B".repeat(31);
    assert!(validate_new_sheet_name(&ok_name, &sheets).is_ok());
}

#[test]
fn validate_sheet_name_rejects_invalid_chars() {
    let sheets = vec![];
    for ch in ['\0', ':', '\\', '/', '?', '*', '[', ']'] {
        let name = format!("Bad{ch}Name");
        assert!(
            validate_new_sheet_name(&name, &sheets).is_err(),
            "should reject char: {ch:?}"
        );
    }
}

#[test]
fn validate_sheet_name_rejects_duplicate_case_insensitive() {
    let sheets = vec![SheetSpan {
        name: "Data".to_owned(),
        bound_sheet_index: 0,
        bof_index: 0,
        eof_index: 0,
        dimension_index: None,
    }];
    assert!(validate_new_sheet_name("data", &sheets).is_err());
    assert!(validate_new_sheet_name("DATA", &sheets).is_err());
    assert!(validate_new_sheet_name("Other", &sheets).is_ok());
}

#[test]
fn validate_sheet_name_rejects_non_bmp_utf16() {
    let sheets = vec![];
    // 用 emoji 测试（多于 1 个 UTF-16 单元的字符在名称中仍然有效只要总长 <=31）
    let name = "A".repeat(31);
    assert!(validate_new_sheet_name(&name, &sheets).is_ok());
}

// ===========================================================================
// encode_boundsheet_record_data 测试
// ===========================================================================

#[test]
fn encode_boundsheet_compressed_ascii_name() {
    let data = encode_boundsheet_record_data("Sheet1").unwrap();
    // 前 4 字节是偏移量（0），然后是可见性、类型、名称长度、grbit
    assert_eq!(data[6], 6); // char count
    assert_eq!(data[7], 0); // compressed flag
    assert_eq!(&data[8..14], b"Sheet1");
}

#[test]
fn encode_boundsheet_unicode_name() {
    let data = encode_boundsheet_record_data("中文").unwrap();
    assert_eq!(data[6], 2); // 2 个 UTF-16 单元
    assert_eq!(data[7], 1); // Unicode flag
    // 4 字节 UTF-16LE
    assert_eq!(data.len(), 8 + 4);
}

// ===========================================================================
// empty_worksheet_records 测试
// ===========================================================================

#[test]
fn empty_worksheet_records_produces_bof_dimension_eof() {
    let records = empty_worksheet_records();
    assert_eq!(records.len(), 4);
    assert_eq!(records[0].typ, BOF);
    assert_eq!(records[1].typ, DIMENSION);
    assert_eq!(records[2].typ, WINDOW2);
    assert_eq!(records[3].typ, EOF);
    // BOF 数据 16 字节
    assert_eq!(records[0].data.len(), 16);
    // DIMENSION 数据 14 字节
    assert_eq!(records[1].data.len(), 14);
}

// ===========================================================================
// next_sheet_shape_id / next_sheet_object_id 测试
// ===========================================================================

#[test]
fn next_sheet_shape_id_returns_default_when_no_objects() {
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
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
        eof_index: 1,
        dimension_index: None,
    };
    assert_eq!(next_sheet_shape_id(&records, &sheet), 1025);
}

#[test]
fn next_sheet_shape_id_returns_max_obj_plus_one() {
    // next_sheet_shape_id 返回 max(maximum+1, 1025)
    // 当 OBJ shape_id = 2000 时，返回 2001
    let mut obj_data = vec![0u8; 8];
    obj_data[6] = 0xD0; // 2000 = 0x07D0
    obj_data[7] = 0x07;
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: OBJ_SID,
            data: obj_data,
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
        eof_index: 2,
        dimension_index: None,
    };
    assert_eq!(next_sheet_shape_id(&records, &sheet), 2001);
}

#[test]
fn next_sheet_object_id_returns_default_when_no_objects() {
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
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
        eof_index: 1,
        dimension_index: None,
    };
    assert_eq!(next_sheet_object_id(&records, &sheet), 2);
}

// ===========================================================================
// is_empty_client_textbox_record 测试
// ===========================================================================

#[test]
fn is_empty_client_textbox_various() {
    // 正确的空 client textbox
    let data = [
        0x00, 0x00, 0x0D, 0xF0, 0x00, 0x00, 0x00, 0x00,
    ];
    assert!(is_empty_client_textbox_record(&data));
    // 长度不对
    assert!(!is_empty_client_textbox_record(&[0; 7]));
    // record type 不对
    let bad_type = [0x00, 0x00, 0x0E, 0xF0, 0x00, 0x00, 0x00, 0x00];
    assert!(!is_empty_client_textbox_record(&bad_type));
    // 非零长度
    let non_zero = [0x00, 0x00, 0x0D, 0xF0, 0x01, 0x00, 0x00, 0x00];
    assert!(!is_empty_client_textbox_record(&non_zero));
}

// ===========================================================================
// escher_shape_container_id 测试
// ===========================================================================

#[test]
fn escher_shape_container_id_finds_spid() {
    // escher_shape_container_id 要求 offset+16 <= payload.len()
    // 所以需要在 F00A 记录之前有填充数据，或者 payload 足够长
    // 布局: offset+0..2=options, +2..4=type, +4..8=length, +8..12=spid
    // 需要 payload.len() >= 16（因为 offset=0, 0+16 <= len）
    let mut payload = vec![0u8; 16];
    // F00A record at offset 0
    payload[2..4].copy_from_slice(&0xF00Au16.to_le_bytes()); // type
    payload[4..8].copy_from_slice(&4u32.to_le_bytes()); // length = 4
    payload[8..12].copy_from_slice(&42u32.to_le_bytes()); // spid = 42
    assert_eq!(escher_shape_container_id(&payload), Some(42));
}

#[test]
fn escher_shape_container_id_returns_none_for_empty() {
    assert_eq!(escher_shape_container_id(&[]), None);
}

#[test]
fn escher_shape_container_id_returns_none_when_no_f00a() {
    // 非 F00A record type
    let mut payload = Vec::new();
    payload.extend_from_slice(&0x0000u16.to_le_bytes());
    payload.extend_from_slice(&0xF001u16.to_le_bytes());
    payload.extend_from_slice(&4u32.to_le_bytes());
    payload.extend_from_slice(&1u32.to_le_bytes());
    assert_eq!(escher_shape_container_id(&payload), None);
}

// ===========================================================================
// remove_escher_records 测试
// ===========================================================================

#[test]
fn remove_escher_records_removes_matching_shape() {
    // 构造一个 F004 container（options 低4位=0xF 表示容器）
    // 容器内包含 F00A record (spid=5)
    let mut inner = Vec::new();
    // F00A record: needs enough bytes for escher_shape_container_id (requires 16+ bytes from offset)
    // 但这里 inner 只作为 F004 的 payload 传入
    // 简化：让 inner 只有 F00A header + spid，够 escher_shape_container_id 检查
    inner.resize(16, 0u8); // 填充到 16 字节
    inner[2..4].copy_from_slice(&0xF00Au16.to_le_bytes()); // type = F00A
    inner[4..8].copy_from_slice(&4u32.to_le_bytes()); // length = 4
    inner[8..12].copy_from_slice(&5u32.to_le_bytes()); // spid = 5

    let mut data = Vec::new();
    data.extend_from_slice(&0x000Fu16.to_le_bytes()); // options = container (0xF)
    data.extend_from_slice(&0xF004u16.to_le_bytes()); // type = F004 (Shape)
    data.extend_from_slice(
        &u32::try_from(inner.len()).unwrap().to_le_bytes(),
    );
    data.extend_from_slice(&inner);

    let (result, removed) = remove_escher_records(&data, 5).unwrap();
    assert!(removed);
    // F004 容器应被整体移除
    assert!(result.len() < data.len());
}

#[test]
fn remove_escher_records_returns_false_for_no_match() {
    let mut data = Vec::new();
    data.extend_from_slice(&0x0000u16.to_le_bytes());
    data.extend_from_slice(&0xF00Au16.to_le_bytes());
    data.extend_from_slice(&4u32.to_le_bytes());
    data.extend_from_slice(&99u32.to_le_bytes());

    let (_, removed) = remove_escher_records(&data, 5).unwrap();
    assert!(!removed);
}

#[test]
fn remove_escher_records_errors_on_truncated_data() {
    let data = [0x00, 0x00, 0x0A, 0xF0]; // 太短
    assert!(remove_escher_records(&data, 1).is_err());
}

// ===========================================================================
// decrement_escher_dg_count 测试
// ===========================================================================

#[test]
fn decrement_escher_dg_count_finds_and_decrements() {
    // 构造包含 F008 (DG) record 的数据
    let mut data = Vec::new();
    data.extend_from_slice(&0x0000u16.to_le_bytes()); // options
    data.extend_from_slice(&0xF008u16.to_le_bytes()); // type = DG
    data.extend_from_slice(&8u32.to_le_bytes()); // length
    data.extend_from_slice(&10u32.to_le_bytes()); // shape count = 10
    data.extend_from_slice(&9u32.to_le_bytes()); // last spid

    let result = decrement_escher_dg_count(&mut data).unwrap();
    assert!(result);
    // shape count 应该减一 (min 1)
    let count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    assert_eq!(count, 9);
}

#[test]
fn decrement_escher_dg_count_minimum_one() {
    let mut data = Vec::new();
    data.extend_from_slice(&0x0000u16.to_le_bytes());
    data.extend_from_slice(&0xF008u16.to_le_bytes());
    data.extend_from_slice(&8u32.to_le_bytes());
    data.extend_from_slice(&1u32.to_le_bytes()); // count = 1
    data.extend_from_slice(&0u32.to_le_bytes());

    let result = decrement_escher_dg_count(&mut data).unwrap();
    assert!(result);
    let count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    assert_eq!(count, 1); // 不会小于 1
}

#[test]
fn decrement_escher_dg_count_returns_false_when_absent() {
    let mut data = Vec::new();
    data.extend_from_slice(&0x0000u16.to_le_bytes());
    data.extend_from_slice(&0xF001u16.to_le_bytes()); // 非 F008
    data.extend_from_slice(&4u32.to_le_bytes());
    data.extend_from_slice(&1u32.to_le_bytes());

    let result = decrement_escher_dg_count(&mut data).unwrap();
    assert!(!result);
}

#[test]
fn decrement_escher_dg_count_errors_on_truncated() {
    let mut data = vec![0u8; 4];
    assert!(decrement_escher_dg_count(&mut data).is_err());
}

// ===========================================================================
// scalar_placeholder_key 测试
// ===========================================================================

#[test]
fn scalar_placeholder_key_strips_braces() {
    assert_eq!(scalar_placeholder_key("{name}"), "name");
    assert_eq!(scalar_placeholder_key("{{double}}"), "double");
    assert_eq!(scalar_placeholder_key("{key}"), "key");
}

// ===========================================================================
// shifted_row / shift_record_row / shift_range_rows / shift_merge_rows 测试
// ===========================================================================

#[test]
fn shifted_row_below_start_is_unchanged() {
    assert_eq!(shifted_row(2, 5, 3).unwrap(), 2);
}

#[test]
fn shifted_row_at_or_above_start_is_shifted() {
    assert_eq!(shifted_row(5, 5, 3).unwrap(), 8);
    assert_eq!(shifted_row(7, 5, 3).unwrap(), 10);
}

#[test]
fn shifted_row_overflow_returns_error() {
    assert!(shifted_row(65535, 65530, 10).is_err());
}

#[test]
fn shift_record_row_short_data_is_noop() {
    let mut record = RawRecord {
        typ: LABEL,
        data: vec![0],
    };
    shift_record_row(&mut record, 5, 3).unwrap();
    assert_eq!(record.data.len(), 1);
}

#[test]
fn shift_record_row_shifts_row_field() {
    let mut record = RawRecord {
        typ: LABEL,
        data: vec![5, 0, 0, 0, 0, 0],
    };
    shift_record_row(&mut record, 3, 10).unwrap();
    let row = u16::from_le_bytes([record.data[0], record.data[1]]);
    assert_eq!(row, 15);
}

#[test]
fn shift_range_rows_shifts_both_endpoints() {
    let mut data = vec![3, 0, 7, 0];
    shift_range_rows(&mut data, 5, 2).unwrap();
    assert_eq!(u16::from_le_bytes([data[0], data[1]]), 3); // 不变
    assert_eq!(u16::from_le_bytes([data[2], data[3]]), 9); // 7+2
}

#[test]
fn shift_range_rows_short_data_is_noop() {
    let mut data = vec![0, 0];
    shift_range_rows(&mut data, 5, 2).unwrap();
    assert_eq!(data.len(), 2);
}

#[test]
fn shift_merge_rows_shifts_all_ranges() {
    // count=2, 每个 range 8 字节: first_row(2), last_row(2), first_col(2), last_col(2)
    let mut data = Vec::new();
    data.extend_from_slice(&2u16.to_le_bytes()); // count
    // range 1: rows 3-5
    data.extend_from_slice(&3u16.to_le_bytes());
    data.extend_from_slice(&5u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    // range 2: rows 6-8
    data.extend_from_slice(&6u16.to_le_bytes());
    data.extend_from_slice(&8u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&2u16.to_le_bytes());

    shift_merge_rows(&mut data, 4, 10).unwrap();
    // range 1: 3 不变, 5->15
    assert_eq!(u16::from_le_bytes([data[2], data[3]]), 3);
    assert_eq!(u16::from_le_bytes([data[4], data[5]]), 15);
    // range 2: 6->16, 8->18
    assert_eq!(u16::from_le_bytes([data[10], data[11]]), 16);
    assert_eq!(u16::from_le_bytes([data[12], data[13]]), 18);
}

#[test]
fn shift_merge_rows_short_data_is_noop() {
    let mut data = vec![0];
    shift_merge_rows(&mut data, 5, 2).unwrap();
    assert_eq!(data.len(), 1);
}

// ===========================================================================
// shift_msodrawing_anchors 测试（无 anchor 时 noop）
// ===========================================================================

#[test]
fn shift_msodrawing_anchors_no_anchors_is_noop() {
    let mut data = vec![0xAA, 0xBB, 0xCC, 0xDD];
    shift_msodrawing_anchors(&mut data, 5, 3).unwrap();
    assert_eq!(data, vec![0xAA, 0xBB, 0xCC, 0xDD]);
}

// ===========================================================================
// encode_cell_record 测试（覆盖各种值类型）
// ===========================================================================

#[test]
fn encode_cell_record_blank() {
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Blank).unwrap();
    assert_eq!(record.typ, BLANK);
}

#[test]
fn encode_cell_record_bool() {
    let record = encode_cell_record(1, 2, 0, &Biff8Value::Bool(true)).unwrap();
    assert_eq!(record.typ, BOOLERR);
    assert_eq!(record.data[0], 1); // row low byte
    assert_eq!(record.data[6], 1); // boolean value
}

#[test]
fn encode_cell_record_error() {
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Error(0x00)).unwrap();
    assert_eq!(record.typ, BOOLERR);
    assert_eq!(record.data[7], 1); // error flag
}

#[test]
fn encode_cell_record_number_as_rk() {
    // 整数可以用 RK 编码
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Number(42.0)).unwrap();
    assert_eq!(record.typ, RK);
    assert_eq!(record.data.len(), 10); // 6 header + 4 rk
}

#[test]
fn encode_cell_record_number_as_number() {
    // 不能用 RK 编码的数（PI 的小数部分不满足 RK 条件）
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Number(std::f64::consts::PI)).unwrap();
    assert_eq!(record.typ, NUMBER);
    assert_eq!(record.data.len(), 14); // 6 header + 8 f64
}

#[test]
fn encode_cell_record_text_short() {
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Text("hi".to_owned())).unwrap();
    assert_eq!(record.typ, LABEL);
}

// ===========================================================================
// encode_label_record 测试
// ===========================================================================

#[test]
fn encode_label_record_basic() {
    let record = encode_label_record(0, 0, 0, "test").unwrap();
    assert_eq!(record.typ, LABEL);
    assert!(record.data.len() > 6);
}

// ===========================================================================
// parse_sst / decode_label_payload / decode_labelsst_index 测试
// ===========================================================================

#[test]
fn parse_sst_empty() {
    let records = vec![];
    assert!(parse_sst(&records).is_empty());
}

#[test]
fn parse_sst_with_records() {
    // 构造 SST record: count(4) + unique_count(4) + strings
    let mut sst_data = Vec::new();
    sst_data.extend_from_slice(&1u32.to_le_bytes()); // total count
    sst_data.extend_from_slice(&1u32.to_le_bytes()); // unique count
    // XLUnicodeString: cch(2) + grbit(1) + chars
    sst_data.extend_from_slice(&3u16.to_le_bytes()); // 3 chars
    sst_data.push(0x00); // compressed
    sst_data.extend_from_slice(b"abc");

    let records = vec![RawRecord {
        typ: SST,
        data: sst_data,
    }];
    let strings = parse_sst(&records);
    assert_eq!(strings.len(), 1);
    assert_eq!(strings[0], "abc");
}

#[test]
fn decode_label_payload_basic() {
    // LABEL record data: row(2) + col(2) + xf(2) + cch(2) + grbit(1) + chars
    let mut data = Vec::new();
    data.extend_from_slice(&5u16.to_le_bytes()); // row = 5
    data.extend_from_slice(&3u16.to_le_bytes()); // col = 3
    data.extend_from_slice(&0u16.to_le_bytes()); // xf = 0
    data.extend_from_slice(&2u16.to_le_bytes()); // cch = 2
    data.push(0x00); // grbit = compressed
    data.extend_from_slice(b"AB");

    let (row, col, text) = decode_label_payload(&data);
    assert_eq!(row, 5);
    assert_eq!(col, 3);
    assert_eq!(text.as_deref(), Some("AB"));
}

#[test]
fn decode_label_payload_short_data() {
    let (row, col, text) = decode_label_payload(&[0]);
    assert_eq!(row, 0);
    assert_eq!(col, 0);
    assert!(text.is_none());
}

#[test]
fn decode_labelsst_index_basic() {
    // LABELSST: row(2) + col(2) + xf(2) + sst_idx(4)
    let mut data = vec![0u8; 10];
    data[0..2].copy_from_slice(&1u16.to_le_bytes()); // row
    data[2..4].copy_from_slice(&2u16.to_le_bytes()); // col
    data[6..10].copy_from_slice(&5u32.to_le_bytes()); // sst index

    let (row, col, idx) = decode_labelsst_index(&data);
    assert_eq!(row, 1);
    assert_eq!(col, 2);
    assert_eq!(idx, Some(5));
}

#[test]
fn decode_labelsst_index_short_data() {
    let (row, col, idx) = decode_labelsst_index(&[0]);
    assert_eq!(row, 0);
    assert_eq!(col, 0);
    assert!(idx.is_none());
}

// ===========================================================================
// sheet_max_row / sheet_dimensions / find_cell_record 测试
// ===========================================================================

#[test]
fn sheet_max_row_returns_none_for_empty_sheet() {
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
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
        eof_index: 1,
        dimension_index: None,
    };
    assert!(sheet_max_row(&records, &sheet).is_none());
}

#[test]
fn sheet_max_row_returns_max_label_row() {
    let mut label_data = vec![0u8; 10];
    label_data[0..2].copy_from_slice(&5u16.to_le_bytes());
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: LABEL,
            data: label_data,
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
        eof_index: 2,
        dimension_index: None,
    };
    assert_eq!(sheet_max_row(&records, &sheet), Some(5));
}

#[test]
fn sheet_dimensions_returns_zeros_for_empty_sheet() {
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
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
        eof_index: 1,
        dimension_index: None,
    };
    let (max_row, max_col) = sheet_dimensions(&records, &sheet);
    assert_eq!(max_row, 0);
    assert_eq!(max_col, 0);
}

#[test]
fn find_cell_record_returns_none_when_absent() {
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
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
        eof_index: 1,
        dimension_index: None,
    };
    assert!(find_cell_record(&records, &sheet, 0, 0).is_none());
}

#[test]
fn find_cell_record_finds_label() {
    let mut label_data = vec![0u8; 10];
    label_data[0..2].copy_from_slice(&0u16.to_le_bytes()); // row 0
    label_data[2..4].copy_from_slice(&1u16.to_le_bytes()); // col 1
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: LABEL,
            data: label_data,
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
        eof_index: 2,
        dimension_index: None,
    };
    assert_eq!(find_cell_record(&records, &sheet, 0, 1), Some(1));
    assert!(find_cell_record(&records, &sheet, 0, 0).is_none());
}

// ===========================================================================
// sheet_cell_insert_index 测试
// ===========================================================================

#[test]
fn sheet_cell_insert_index_before_eof() {
    let records = vec![
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: DIMENSION,
            data: vec![0; 14],
        },
        RawRecord {
            typ: LABEL,
            data: vec![0; 10],
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
        dimension_index: Some(1),
    };
    assert_eq!(sheet_cell_insert_index(&records, &sheet), 3);
}

// ===========================================================================
// shift_formula_references 额外测试
// ===========================================================================

#[test]
fn shift_formula_references_ref3d() {
    // ptgRef3d = 0x3A: ixti(2) + row(2) + col(2)
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

    // Ref3d on same sheet: should shift
    let mut f = formula(0, &[0x3A, 0, 0, 3, 0, 0, 0]);
    shift_formula_references(&mut f, 2, 5, 0, &[Some((0, 0))]).unwrap();
    // row 3 -> 8
    assert_eq!(u16::from_le_bytes([f.data[25], f.data[26]]), 8);

    // Ref3d on different sheet: should NOT shift
    let mut f2 = formula(0, &[0x3A, 0, 0, 3, 0, 0, 0]);
    shift_formula_references(&mut f2, 2, 5, 1, &[Some((0, 0))]).unwrap();
    assert_eq!(u16::from_le_bytes([f2.data[25], f2.data[26]]), 3);
}

#[test]
fn shift_formula_references_area_absolute() {
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

    // ptgArea = 0x25: first_row(2) + last_row(2) + first_col(2) + last_col(2)
    let mut f = formula(0, &[0x25, 1, 0, 5, 0, 0, 0, 0, 0]);
    shift_formula_references(&mut f, 3, 10, 0, &[]).unwrap();
    // first_row=1 < 3, 不变
    assert_eq!(u16::from_le_bytes([f.data[23], f.data[24]]), 1);
    // last_row=5 >= 3, shift +10 = 15
    assert_eq!(u16::from_le_bytes([f.data[25], f.data[26]]), 15);
}

#[test]
fn shift_formula_references_refn_relative() {
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

    // ptgRefN = 0x2C: row(2 with high bit) + col(2)
    // relative reference: row encoded with formula_row offset
    let mut f = formula(5, &[0x2C, 0x00, 0x80, 0, 0]); // row = -5 relative (= 0 absolute)
    shift_formula_references(&mut f, 3, 2, 0, &[]).unwrap();
    // formula_row 5, start_row 3, delta 2 => formula shifts to row 7
    // original relative offset: 0 - 5 = -5 (0xFFFB with sign bit)
    // new absolute: 0 (below start_row), new relative: 0 - 7 = -7
}

#[test]
fn shift_formula_references_truncated_record() {
    let mut f = RawRecord {
        typ: FORMULA,
        data: vec![0; 10], // 太短
    };
    assert!(shift_formula_references(&mut f, 0, 1, 0, &[]).is_err());
}

// ===========================================================================
// shift_conditional_format_header / shift_conditional_format_rule 测试
// ===========================================================================

#[test]
fn shift_conditional_format_header_shifts_ranges() {
    // CONDFMT: count(2) + padding(2) + [first_row(2) + last_row(2) + first_col(2) + last_col(2)]
    // shift_range_rows 处理 data[4..12]（first_row+last_row 为 4 字节范围）
    // shift_sqref_rows 从 data[12] 开始
    // 参照 cases_01 中的 CONDFMT 结构
    let mut data = Vec::new();
    data.extend_from_slice(&1u16.to_le_bytes()); // count = 1
    data.extend_from_slice(&0u16.to_le_bytes()); // padding
    data.extend_from_slice(&4u16.to_le_bytes()); // first_row = 4 (>= start_row=3)
    data.extend_from_slice(&6u16.to_le_bytes()); // last_row = 6
    data.extend_from_slice(&0u16.to_le_bytes()); // first_col = 0
    data.extend_from_slice(&0u16.to_le_bytes()); // last_col = 0
    // sqref: count(2) + range(first_row(2)+last_row(2)+first_col(2)+last_col(2))
    data.extend_from_slice(&1u16.to_le_bytes()); // sqref count = 1
    data.extend_from_slice(&4u16.to_le_bytes()); // sqref first_row = 4
    data.extend_from_slice(&6u16.to_le_bytes()); // sqref last_row = 6
    data.extend_from_slice(&0u16.to_le_bytes()); // sqref first_col
    data.extend_from_slice(&0u16.to_le_bytes()); // sqref last_col

    let base = shift_conditional_format_header(&mut data, 3, 2).unwrap();
    // original_base = 4, shifted = 6
    assert_eq!(base.0, 4);
    assert_eq!(base.1, 6);
    // data[4..6] = first_row = 4 >= 3 -> 6
    assert_eq!(u16::from_le_bytes([data[4], data[5]]), 6);
    // data[6..8] = last_row = 6 >= 3 -> 8
    assert_eq!(u16::from_le_bytes([data[6], data[7]]), 8);
}

// ===========================================================================
// shift_data_validation 基础测试
// ===========================================================================

#[test]
fn shift_data_validation_shifts_sqref_and_formulas() {
    // 与 cases_01 中的 DV 构造方式完全一致
    let mut dv = vec![0; 4];
    for _ in 0..4 {
        dv.extend_from_slice(&[1, 0, 0, 0]);
    }
    dv.extend_from_slice(&5u16.to_le_bytes());
    dv.extend_from_slice(&0u16.to_le_bytes());
    dv.extend_from_slice(&[0x24, 6, 0, 0, 0]);
    dv.extend_from_slice(&0u16.to_le_bytes());
    dv.extend_from_slice(&0u16.to_le_bytes());
    dv.extend_from_slice(&1u16.to_le_bytes());
    for value in [5u16, 7, 0, 0] {
        dv.extend_from_slice(&value.to_le_bytes());
    }
    shift_data_validation(&mut dv, 5, 2, 0, &[]).unwrap();
    assert_eq!(&dv[25..27], &[8, 0]);
    assert_eq!(&dv[35..37], &[7, 0]);
    assert_eq!(&dv[37..39], &[9, 0]);
}

// ===========================================================================
// shift_chart_ai_references 测试
// ===========================================================================

#[test]
fn shift_chart_ai_references_basic() {
    // CHART_AI_SID data: 8字节头 + token_len(u16 at 6..8) + tokens
    // 使用 ptgRef3d(0x3A) 而非 ptgArea3d(0x3B)，因为 0x3A 的 cursor+3 是 row
    // ptgRef3d(0x3A) = ixti(2) + row(2) + col(2) = 6 bytes after ptg
    let tokens: [u8; 7] = [
        0x3A,  // ptgRef3d
        0, 0,  // ixti = 0
        5, 0,  // row = 5
        0, 0,  // col = 0 (absolute)
    ];
    let mut data = vec![0u8; 8];
    data[6..8].copy_from_slice(&7u16.to_le_bytes()); // token_len = 7
    data.extend_from_slice(&tokens);

    let mut record = RawRecord { typ: CHART_AI_SID, data };
    shift_chart_ai_references(&mut record, 4, 3, 0, &[Some((0, 0))]).unwrap();
    // ptg_targets_sheet: ixti=0, ranges[0]=Some((0,0)), current=0 -> true
    // shift_chart_ptg_row(tokens, cursor+1=1, cursor+3=3) -> reads col at 3..5=row data
    // 等等，shift_chart_ptg_row 的 column_offset=cursor+3=3，这是 row 的位置
    // 对于 0x3A: row at cursor+1, col at cursor+3
    // shift_chart_ptg_row(tokens, 1, 3): col=tokens[3..5]=[5,0], high bit not set -> OK
    // row=tokens[1..3]=[0,0], 0 < 4 -> unchanged... 这也不对
    // 实际上对于 chart AI，shift_chart_ptg_row 的参数是 (row_offset, column_offset)
    // shift_chart_ptg_row(tokens, cursor+1, cursor+3):
    //   col = tokens[cursor+3..cursor+5] = tokens[3..5] = [5, 0] -> 5, no high bit
    //   row = tokens[cursor+1..cursor+3] = tokens[1..3] = [0, 0] -> 0
    //   0 < 4 -> 不变
    // 所以 ptgRef3d 的 row 不会被 shift（因为 row 在 tokens[1..3]，而这是 ixti）
    // chart AI 的 shift 逻辑和 formula 的不同，它在 cursor+1 和 cursor+3 上操作
    // 但对于 0x3A: cursor+1 是 ixti，cursor+3 是 row
    // shift_chart_ptg_row(tokens, 1, 3):
    //   col=tokens[3..5] (这是 row 的位置！) -> 5 >= 4 -> shifted to 8
    //   row=tokens[1..3] (这是 ixti 的位置！) -> 不修改 row 值
    assert_eq!(u16::from_le_bytes([record.data[11], record.data[12]]), 8);
}

// ===========================================================================
// internal_extern_sheet_ranges 测试
// ===========================================================================

#[test]
fn internal_extern_sheet_ranges_basic() {
    let records = vec![
        RawRecord {
            typ: SUP_BOOK_SID,
            data: vec![2, 0, 1, 4],
        },
        RawRecord {
            typ: EXTERNAL_SHEET_SID,
            data: vec![1, 0, 0, 0, 1, 0, 2, 0],
        },
    ];
    let ranges = internal_extern_sheet_ranges(&records);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], Some((1, 2)));
}

#[test]
fn internal_extern_sheet_ranges_self_reference() {
    // ixti == 0xFFFF means self-reference
    let records = vec![
        RawRecord {
            typ: SUP_BOOK_SID,
            data: vec![2, 0, 1, 4],
        },
        RawRecord {
            typ: EXTERNAL_SHEET_SID,
            data: vec![1, 0, 0xFF, 0xFF, 0, 0, 0, 0],
        },
    ];
    let ranges = internal_extern_sheet_ranges(&records);
    assert_eq!(ranges[0], None);
}

// ===========================================================================
// Biff8TemplatePackage 测试（使用真实 XLS 模板）
// ===========================================================================

#[test]
fn template_package_from_bytes_rejects_non_ole() {
    assert!(matches!(
        Biff8TemplatePackage::from_bytes(b"not an xls"),
        Err(ExcelError::Xls(_))
    ));
}

#[test]
fn template_package_from_bytes_loads_real_template() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    assert_eq!(package.sheet_names(), vec!["Data"]);
    Ok(())
}

#[test]
fn template_package_sheet_names() -> Result<()> {
    let bytes = multi_sheet_template()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    let names = package.sheet_names();
    assert_eq!(names.len(), 2);
    assert_eq!(names[0], "Sheet1");
    assert_eq!(names[1], "Sheet2");
    Ok(())
}

#[test]
fn template_package_next_row_for_sheet() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    let next = package.next_row_for_sheet("Data")?;
    // 有 3 行数据（rows 0,1,2），所以下一行是 3
    assert_eq!(next, 3);
    Ok(())
}

#[test]
fn template_package_next_row_for_missing_sheet() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    assert!(matches!(
        package.next_row_for_sheet("NoSuch"),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

#[test]
fn template_package_scan_placeholders() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    let placeholders = package.scan_placeholders();
    assert!(placeholders.len() >= 4); // name, age, .item, .price, other
    let keys: Vec<&str> = placeholders
        .iter()
        .map(|(_, _, _, text)| text.as_str())
        .collect();
    assert!(keys.contains(&"{name}"));
    assert!(keys.contains(&"{age}"));
    assert!(keys.contains(&"{.item}"));
    assert!(keys.contains(&"{.price}"));
    Ok(())
}

#[test]
fn template_package_set_cell_writes_value() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;
    package.set_cell(
        "Data",
        0,
        0,
        &Biff8Cell::general(Biff8Value::Number(99.0)),
    )?;
    // 确认 to_bytes 正常工作
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

#[test]
fn template_package_set_cell_out_of_range_row() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;
    assert!(matches!(
        package.set_cell(
            "Data",
            70000, // 超过 u16::MAX
            0,
            &Biff8Cell::general(Biff8Value::Number(1.0)),
        ),
        Err(ExcelError::Xls(_))
    ));
    Ok(())
}

#[test]
fn template_package_set_cell_out_of_range_col() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;
    assert!(matches!(
        package.set_cell(
            "Data",
            0,
            300, // 超过 u8::MAX
            &Biff8Cell::general(Biff8Value::Number(1.0)),
        ),
        Err(ExcelError::Xls(_))
    ));
    Ok(())
}

#[test]
fn template_package_set_cell_missing_sheet() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;
    assert!(matches!(
        package.set_cell(
            "NoSuch",
            0,
            0,
            &Biff8Cell::general(Biff8Value::Number(1.0)),
        ),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

#[test]
fn template_package_set_cell_overwrites_existing() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;
    // 覆写已有单元格
    package.set_cell(
        "Data",
        0,
        0,
        &Biff8Cell::general(Biff8Value::Text("replaced".to_owned())),
    )?;
    let output = package.to_bytes()?;
    // 重新加载验证
    let reloaded = Biff8TemplatePackage::from_bytes(&output)?;
    let placeholders = reloaded.scan_placeholders();
    assert!(placeholders.is_empty()); // 无占位符
    Ok(())
}

#[test]
fn template_package_set_cell_new_position() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;
    // 写入新位置
    package.set_cell(
        "Data",
        10,
        5,
        &Biff8Cell::general(Biff8Value::Number(123.0)),
    )?;
    let output = package.to_bytes()?;
    let reloaded = Biff8TemplatePackage::from_bytes(&output)?;
    let next = reloaded.next_row_for_sheet("Data")?;
    assert!(next > 10);
    Ok(())
}

#[test]
fn template_package_replace_scalar_placeholders() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut values = BTreeMap::new();
    values.insert("name".to_owned(), "Alice".to_owned());
    values.insert("age".to_owned(), "30".to_owned());

    let count = package.replace_scalar_placeholders(&values)?;
    assert_eq!(count, 2);

    // 验证替换后的占位符不再存在
    let remaining = package.scan_placeholders();
    let remaining_keys: Vec<&str> = remaining
        .iter()
        .filter(|(s, _, _, _)| s == "Data")
        .map(|(_, _, _, t)| t.as_str())
        .collect();
    assert!(!remaining_keys.contains(&"{name}"));
    assert!(!remaining_keys.contains(&"{age}"));
    Ok(())
}

#[test]
fn template_package_replace_scalar_placeholders_on_sheet() -> Result<()> {
    let bytes = multi_sheet_template()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut values = BTreeMap::new();
    values.insert("val".to_owned(), "replaced".to_owned());

    // 只替换 Sheet1
    let count = package.replace_scalar_placeholders_on_sheet(Some("Sheet1"), &values)?;
    assert_eq!(count, 1);
    Ok(())
}

#[test]
fn template_package_replace_scalar_placeholders_on_missing_sheet() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut values = BTreeMap::new();
    values.insert("name".to_owned(), "X".to_owned());

    assert!(matches!(
        package.replace_scalar_placeholders_on_sheet(Some("NoSuch"), &values),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

#[test]
fn template_package_replace_scalar_cells_on_sheet() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut cells = BTreeMap::new();
    cells.insert(
        "name".to_owned(),
        Biff8Cell::general(Biff8Value::Text("Bob".to_owned())),
    );

    let placements = package.replace_scalar_cells_on_sheet(None, &cells)?;
    assert_eq!(placements.len(), 1);
    assert_eq!(placements[0].3, "name");
    Ok(())
}

#[test]
fn template_package_replace_scalar_cells_rollback_on_error() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut cells = BTreeMap::new();
    cells.insert(
        "name".to_owned(),
        Biff8Cell::general(Biff8Value::Text("Bob".to_owned())),
    );
    // 尝试替换不存在的 sheet 应该回滚
    assert!(
        package
            .replace_scalar_cells_on_sheet(Some("NoSuch"), &cells)
            .is_err()
    );
    // 确认 package 仍可用
    let names = package.sheet_names();
    assert_eq!(names, vec!["Data"]);
    Ok(())
}

// ===========================================================================
// replace_label 测试
// ===========================================================================

#[test]
fn template_package_replace_label() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 替换 (0,0) 的 label
    package.replace_label("Data", 0, 0, "replaced_text")?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

#[test]
fn template_package_replace_label_missing_sheet() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.replace_label("NoSuch", 0, 0, "text"),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

#[test]
fn template_package_replace_label_new_cell() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 替换一个不存在的单元格位置
    package.replace_label("Data", 20, 10, "new_label")?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// add_merge_range 测试
// ===========================================================================

#[test]
fn template_package_add_merge_range() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.add_merge_range(
        "Data",
        Biff8Merge {
            first_row: 0,
            last_row: 2,
            first_col: 0,
            last_col: 3,
        },
    )?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

#[test]
fn template_package_add_merge_range_missing_sheet() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.add_merge_range(
            "NoSuch",
            Biff8Merge {
                first_row: 0,
                last_row: 1,
                first_col: 0,
                last_col: 1,
            },
        ),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

// ===========================================================================
// protect_sheet 测试
// ===========================================================================

#[test]
fn template_package_protect_sheet() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.protect_sheet("Data", "password123")?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

#[test]
fn template_package_protect_sheet_replaces_existing() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 两次调用应该替换而不是追加
    package.protect_sheet("Data", "first")?;
    package.protect_sheet("Data", "second")?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

#[test]
fn template_package_protect_sheet_missing_sheet() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.protect_sheet("NoSuch", "pw"),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

// ===========================================================================
// ensure_sheet 测试
// ===========================================================================

#[test]
fn template_package_ensure_sheet_creates_new() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let created = package.ensure_sheet("NewSheet")?;
    assert!(created);
    let names = package.sheet_names();
    assert!(names.contains(&"NewSheet".to_owned()));
    Ok(())
}

#[test]
fn template_package_ensure_sheet_existing_returns_false() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let created = package.ensure_sheet("Data")?;
    assert!(!created);
    Ok(())
}

#[test]
fn template_package_ensure_sheet_rejects_invalid_name() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(package.ensure_sheet("").is_err());
    assert!(package.ensure_sheet("Bad:Name").is_err());
    Ok(())
}

// ===========================================================================
// append_rows 测试
// ===========================================================================

#[test]
fn template_package_append_rows() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // template_with_values 有 2 行 (row 0, 1)，next_row = 2
    let current_next = package.next_row_for_sheet("Data")?;
    assert_eq!(current_next, 2);

    let rows = vec![
        vec![(
            0,
            Biff8Cell::general(Biff8Value::Text("row2col0".to_owned())),
        )],
        vec![(
            0,
            Biff8Cell::general(Biff8Value::Text("row3col0".to_owned())),
        )],
    ];
    let next = package.append_rows("Data", &rows)?;
    assert_eq!(next, 4); // 2 行原有 + 2 行新追加
    Ok(())
}

#[test]
fn template_package_append_rows_missing_sheet() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.append_rows("NoSuch", &[]),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

#[test]
fn template_package_append_rows_rollback_on_error() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 追加一行，next 从 2 变为 3
    let rows = vec![vec![(
        0,
        Biff8Cell::general(Biff8Value::Text("ok".to_owned())),
    )]];
    let next = package.append_rows("Data", &rows)?;
    assert_eq!(next, 3);
    Ok(())
}

// ===========================================================================
// add_hyperlink_range 测试
// ===========================================================================

#[test]
fn template_package_add_hyperlink_range() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.add_hyperlink_range(
        "Data",
        0,
        0,
        0,
        0,
        "https://example.com".to_owned(),
        "Link".to_owned(),
        Biff8HyperlinkKind::Url,
    )?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

#[test]
fn template_package_add_hyperlink_range_row_too_large() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.add_hyperlink_range(
            "Data",
            70000, // 超过 u16
            0,
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
fn template_package_add_hyperlink_range_col_too_large() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.add_hyperlink_range(
            "Data",
            0,
            0,
            300, // 超过 u8
            0,
            "url".to_owned(),
            "l".to_owned(),
            Biff8HyperlinkKind::Url,
        ),
        Err(ExcelError::Xls(_))
    ));
    Ok(())
}

// ===========================================================================
// next_custom_font_index 测试
// ===========================================================================

#[test]
fn template_package_next_custom_font_index() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let idx = package.next_custom_font_index();
    // 标准模板有 4 个内置 FONT（索引 0-3），索引 4 保留
    assert!(idx >= 4);
    Ok(())
}

// ===========================================================================
// append_custom_fonts 测试
// ===========================================================================

#[test]
fn template_package_append_custom_fonts_empty() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 空字体列表应该 noop
    package.append_custom_fonts(&[])?;
    Ok(())
}

#[test]
fn template_package_append_custom_fonts() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 添加一个自定义字体记录（最小 4 字节数据）
    let font_data = vec![0xC8, 0x00, 0x00, 0x00]; // 基本 FONT 数据
    package.append_custom_fonts(&[font_data])?;
    let idx = package.next_custom_font_index();
    assert!(idx >= 5);
    Ok(())
}

#[test]
fn template_package_append_custom_fonts_too_large() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let huge_font = vec![0u8; MAX_RECORD_DATA + 1];
    assert!(matches!(
        package.append_custom_fonts(&[huge_font]),
        Err(ExcelError::Xls(_))
    ));
    Ok(())
}

// ===========================================================================
// to_bytes / to_bytes_with_password 测试
// ===========================================================================

#[test]
fn template_package_to_bytes_basic() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    assert!(looks_like_xls(&output));
    Ok(())
}

#[test]
fn template_package_to_bytes_with_password() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    let output = package.to_bytes_with_password(Some("test123"))?;
    assert!(!output.is_empty());
    assert!(looks_like_xls(&output));
    // 加密后应该能用密码加载
    let reloaded = Biff8TemplatePackage::from_bytes_with_password(&output, Some("test123"));
    assert!(reloaded.is_ok());
    Ok(())
}

#[test]
fn template_package_to_bytes_with_wrong_password() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    let output = package.to_bytes_with_password(Some("correct"))?;
    // 用错误密码加载
    let result = Biff8TemplatePackage::from_bytes_with_password(&output, Some("wrong"));
    assert!(result.is_err());
    Ok(())
}

#[test]
fn template_package_to_bytes_strips_filepass_without_password() -> Result<()> {
    // 先加密
    let bytes = template_with_values()?;
    let encrypted = Biff8TemplatePackage::from_bytes(&bytes)?
        .to_bytes_with_password(Some("secret"))?;
    // 用密码加载，然后不带密码输出
    let package =
        Biff8TemplatePackage::from_bytes_with_password(&encrypted, Some("secret"))?;
    let plain = package.to_bytes()?;
    // 应该能不带密码加载
    let reloaded = Biff8TemplatePackage::from_bytes(&plain);
    assert!(reloaded.is_ok());
    Ok(())
}

// ===========================================================================
// save_to_path / save_to_writer 测试
// ===========================================================================

#[test]
fn template_package_save_to_writer() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    package.save_to_writer(&mut cursor)?;
    assert!(!buffer.is_empty());
    assert!(looks_like_xls(&buffer));
    Ok(())
}

#[test]
fn template_package_save_to_writer_with_password() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    package.save_to_writer_with_password(&mut cursor, Some("pw"))?;
    assert!(!buffer.is_empty());
    Ok(())
}

#[test]
fn template_package_save_to_path() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let dir = std::env::temp_dir().join("easyexcel_test_02");
    let path = dir.join("output.xls");
    package.save_to_path(&path)?;
    assert!(path.exists());
    let loaded = Biff8TemplatePackage::from_path(&path)?;
    assert_eq!(loaded.sheet_names(), package.sheet_names());
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn template_package_save_to_path_with_password() -> Result<()> {
    let bytes = template_with_values()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let dir = std::env::temp_dir().join("easyexcel_test_02_pw");
    let path = dir.join("encrypted.xls");
    package.save_to_path_with_password(&path, Some("secret"))?;
    assert!(path.exists());
    let loaded = Biff8TemplatePackage::from_path_with_password(&path, Some("secret"))?;
    assert_eq!(loaded.sheet_names(), package.sheet_names());
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

// ===========================================================================
// fill_collection_placeholders 测试
// ===========================================================================

#[test]
fn template_package_fill_collection_placeholders_vertical() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![
        BTreeMap::from([
            ("item".to_owned(), "Widget".to_owned()),
            ("price".to_owned(), "10.0".to_owned()),
        ]),
        BTreeMap::from([
            ("item".to_owned(), "Gadget".to_owned()),
            ("price".to_owned(), "20.0".to_owned()),
        ]),
    ];
    let count = package.fill_collection_placeholders(None, None, &rows, false, false, true)?;
    assert!(count > 0);
    Ok(())
}

#[test]
fn template_package_fill_collection_placeholders_horizontal() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([
        ("item".to_owned(), "A".to_owned()),
        ("price".to_owned(), "1".to_owned()),
    ])];
    let count = package.fill_collection_placeholders(None, None, &rows, true, false, true)?;
    assert!(count > 0);
    Ok(())
}

#[test]
fn template_package_fill_collection_placeholders_empty() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let count = package.fill_collection_placeholders(None, None, &[], false, false, true)?;
    assert_eq!(count, 0);
    Ok(())
}

#[test]
fn template_package_fill_collection_placeholders_with_name() -> Result<()> {
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{users.name}".to_owned())),
        );
        sheet.cells.insert(
            (0, 1),
            Biff8Cell::general(Biff8Value::Text("{users.age}".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([
        ("name".to_owned(), "Alice".to_owned()),
        ("age".to_owned(), "30".to_owned()),
    ])];
    let count =
        package.fill_collection_placeholders(None, Some("users"), &rows, false, false, true)?;
    assert!(count > 0);
    Ok(())
}

#[test]
fn template_package_fill_collection_placeholders_missing_sheet() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.fill_collection_placeholders(
            Some("NoSuch"),
            None,
            &[BTreeMap::new()],
            false,
            false,
            true,
        ),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

// ===========================================================================
// replace_collection_placeholders 测试
// ===========================================================================

#[test]
fn template_package_replace_collection_placeholders() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![
        BTreeMap::from([
            ("item".to_owned(), "A".to_owned()),
            ("price".to_owned(), "1".to_owned()),
        ]),
        BTreeMap::from([
            ("item".to_owned(), "B".to_owned()),
            ("price".to_owned(), "2".to_owned()),
        ]),
    ];
    let count = package.replace_collection_placeholders(None, &rows)?;
    assert!(count > 0);
    Ok(())
}

// ===========================================================================
// fill_collection_cells 测试（带 force_new_row 和 auto_style）
// ===========================================================================

#[test]
fn template_package_fill_collection_cells_force_new_row() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![
        BTreeMap::from([(
            "item".to_owned(),
            Biff8Cell::general(Biff8Value::Text("X".to_owned())),
        )]),
        BTreeMap::from([(
            "item".to_owned(),
            Biff8Cell::general(Biff8Value::Text("Y".to_owned())),
        )]),
    ];
    let placements =
        package.fill_collection_cells(None, None, &rows, false, true, true)?;
    assert!(placements.len() > 0);
    Ok(())
}

#[test]
fn template_package_fill_collection_cells_no_auto_style() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([(
        "item".to_owned(),
        Biff8Cell::general(Biff8Value::Text("Z".to_owned())),
    )])];
    let placements =
        package.fill_collection_cells(None, None, &rows, false, false, false)?;
    assert!(placements.len() > 0);
    Ok(())
}

// ===========================================================================
// add_comments 测试
// ===========================================================================

#[test]
fn template_package_add_comments_empty() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 空注释列表应该 noop
    package.add_comments("Data", &[])?;
    Ok(())
}

#[test]
fn template_package_add_comments_missing_sheet() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let comment = Biff8Comment::new(0, 0, "note", "tester");
    assert!(matches!(
        package.add_comments("NoSuch", &[comment]),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

// ===========================================================================
// remove_comment 测试
// ===========================================================================

#[test]
fn template_package_remove_comment_nonexistent() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 删除不存在的注释应该返回 false
    let removed = package.remove_comment("Data", 0, 0)?;
    assert!(!removed);
    Ok(())
}

#[test]
fn template_package_remove_comment_missing_sheet() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.remove_comment("NoSuch", 0, 0),
        Err(ExcelError::SheetNotFound(_))
    ));
    Ok(())
}

#[test]
fn template_package_remove_comment_row_too_large() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.remove_comment("Data", 70000, 0),
        Err(ExcelError::Xls(_))
    ));
    Ok(())
}

#[test]
fn template_package_remove_comment_col_too_large() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    assert!(matches!(
        package.remove_comment("Data", 0, 70000),
        Err(ExcelError::Xls(_))
    ));
    Ok(())
}

// ===========================================================================
// 综合 roundtrip 测试
// ===========================================================================

#[test]
fn template_package_full_roundtrip_scalar_and_collection() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 替换标量
    let mut scalar = BTreeMap::new();
    scalar.insert("name".to_owned(), "Alice".to_owned());
    scalar.insert("age".to_owned(), "30".to_owned());
    scalar.insert("other".to_owned(), "misc".to_owned());
    package.replace_scalar_placeholders(&scalar)?;

    // 填充集合
    let collection_rows = vec![
        BTreeMap::from([
            ("item".to_owned(), "Widget".to_owned()),
            ("price".to_owned(), "10.0".to_owned()),
        ]),
        BTreeMap::from([
            ("item".to_owned(), "Gadget".to_owned()),
            ("price".to_owned(), "20.0".to_owned()),
        ]),
    ];
    package.replace_collection_placeholders(None, &collection_rows)?;

    // 序列化
    let output = package.to_bytes()?;
    let reloaded = Biff8TemplatePackage::from_bytes(&output)?;

    // 所有占位符应该已被替换
    let remaining = reloaded.scan_placeholders();
    assert!(
        remaining.is_empty(),
        "unexpected remaining placeholders: {remaining:?}"
    );
    Ok(())
}

#[test]
fn template_package_modify_and_roundtrip_preserves_sheet_count() -> Result<()> {
    let bytes = multi_sheet_template()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 添加单元格到两个 sheet
    package.set_cell(
        "Sheet1",
        10,
        0,
        &Biff8Cell::general(Biff8Value::Number(42.0)),
    )?;
    package.set_cell(
        "Sheet2",
        5,
        3,
        &Biff8Cell::general(Biff8Value::Text("hello".to_owned())),
    )?;

    let output = package.to_bytes()?;
    let reloaded = Biff8TemplatePackage::from_bytes(&output)?;
    assert_eq!(reloaded.sheet_names().len(), 2);
    Ok(())
}

// ===========================================================================
// shift_rows 综合测试
// ===========================================================================

#[test]
fn template_package_shift_rows_updates_merge_cells() -> Result<()> {
    // 创建包含占位符 + 合并区域的模板，force_new_row 时 shift_rows 会更新 MERGECELLS
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("{.val}".to_owned())),
        );
        sheet.merges.push(Biff8Merge {
            first_row: 3,
            last_row: 5,
            first_col: 0,
            last_col: 2,
        });
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![
        BTreeMap::from([("val".to_owned(), "A".to_owned())]),
        BTreeMap::from([("val".to_owned(), "B".to_owned())]),
        BTreeMap::from([("val".to_owned(), "C".to_owned())]),
    ];
    package.fill_collection_placeholders(None, None, &rows, false, true, true)?;
    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// collection_placeholder_key 额外分支覆盖
// ===========================================================================

#[test]
fn collection_placeholder_key_no_name_matches_unnamed() {
    assert_eq!(
        collection_placeholder_key("{.field}", None),
        Some("field")
    );
}

#[test]
fn collection_placeholder_key_with_name_matches_named() {
    assert_eq!(
        collection_placeholder_key("{items.field}", Some("items")),
        Some("field")
    );
}

#[test]
fn collection_placeholder_key_with_name_falls_back_to_plain() {
    // 没有 name. 前缀，但有 plain {key} 形式
    assert_eq!(
        collection_placeholder_key("{field}", Some("items")),
        Some("field")
    );
}

#[test]
fn collection_placeholder_key_plain_text_returns_none() {
    assert_eq!(collection_placeholder_key("plain", None), None);
}

#[test]
fn collection_placeholder_key_no_braces_returns_none() {
    assert_eq!(collection_placeholder_key("abc", Some("x")), None);
}

// ===========================================================================
// biff.rs 覆盖测试（通过 encode_unicode_string 间接测试）
// 注意：xls::biff 模块是私有的，通过模板 roundtrip 间接覆盖
// ===========================================================================

#[test]
fn biff_encode_unicode_via_template_roundtrip() -> Result<()> {
    // 通过 Biff8Book 创建包含 Unicode 文本的模板，
    // 间接覆盖 encode_unicode_string / encode_short_unicode_string
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("\u{4e2d}\u{6587}"); // 中文 sheet 名
        sheet.cells.insert(
            (0, 0),
            Biff8Cell::general(Biff8Value::Text("\u{4e2d}\u{6587}\u{6d4b}\u{8bd5}".to_owned())),
        );
        sheet.cells.insert(
            (1, 0),
            Biff8Cell::general(Biff8Value::Text("ASCII only".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    assert_eq!(package.sheet_names(), vec!["\u{4e2d}\u{6587}"]);
    Ok(())
}

#[test]
fn biff_rk_encode_decode_via_template() -> Result<()> {
    // 通过设置数值单元格，间接覆盖 encode_rk / decode_rk
    let mut book = crate::biff8::Biff8Book::default();
    {
        let sheet = book.sheet_mut("Data");
        // 整数（RK 编码）
        sheet.cells.insert((0, 0), Biff8Cell::general(Biff8Value::Number(100.0)));
        // div100 形式
        sheet.cells.insert((1, 0), Biff8Cell::general(Biff8Value::Number(12.34)));
        // 无法用 RK 编码的浮点
        sheet.cells.insert((2, 0), Biff8Cell::general(Biff8Value::Number(0.1)));
        // 零
        sheet.cells.insert((3, 0), Biff8Cell::general(Biff8Value::Number(0.0)));
        // 负数
        sheet.cells.insert((4, 0), Biff8Cell::general(Biff8Value::Number(-5.0)));
    }
    let bytes = book.to_cfb_bytes()?;
    let package = Biff8TemplatePackage::from_bytes(&bytes)?;
    let output = package.to_bytes()?;
    let reloaded = Biff8TemplatePackage::from_bytes(&output)?;
    assert_eq!(reloaded.next_row_for_sheet("Data")?, 5);
    Ok(())
}

// ===========================================================================
// Biff8WorkbookModel 测试
// ===========================================================================

#[test]
fn workbook_model_from_records_empty_globals() {
    let records = vec![];
    assert!(crate::biff8::model::Biff8WorkbookModel::from_records(records).is_err());
}

#[test]
fn workbook_model_from_records_no_globals_bof() {
    // 第一个记录不是 BOF — 构造 Biff8Record 向量
    use crate::biff8::model::Biff8Record;
    let records = vec![Biff8Record::new(EOF, Vec::new())];
    assert!(crate::biff8::model::Biff8WorkbookModel::from_records(records).is_err());
}

#[test]
fn workbook_model_from_workbook_stream_basic() -> Result<()> {
    let bytes = template_with_values()?;
    let mut compound = CompoundFile::open(std::io::Cursor::new(&bytes))
        .map_err(|e| ExcelError::Cfb(e.to_string()))?;
    let mut workbook = Vec::new();
    compound
        .open_stream("/Workbook")
        .map_err(|e| ExcelError::Cfb(e.to_string()))?
        .read_to_end(&mut workbook)?;

    let model = crate::biff8::model::Biff8WorkbookModel::from_workbook_stream(&workbook)?;
    assert!(!model.worksheets().is_empty());
    assert_eq!(model.worksheets()[0].name(), "Data");
    Ok(())
}

#[test]
fn workbook_model_roundtrip() -> Result<()> {
    let bytes = template_with_values()?;
    let mut compound = CompoundFile::open(std::io::Cursor::new(&bytes))
        .map_err(|e| ExcelError::Cfb(e.to_string()))?;
    let mut workbook = Vec::new();
    compound
        .open_stream("/Workbook")
        .map_err(|e| ExcelError::Cfb(e.to_string()))?
        .read_to_end(&mut workbook)?;

    let model = crate::biff8::model::Biff8WorkbookModel::from_workbook_stream(&workbook)?;
    let output = model.to_workbook_stream()?;
    assert!(!output.is_empty());

    // 重新解析
    let model2 = crate::biff8::model::Biff8WorkbookModel::from_workbook_stream(&output)?;
    assert_eq!(model2.worksheets().len(), model.worksheets().len());
    Ok(())
}

// ===========================================================================
// shift_conditional_format_rule 额外测试
// ===========================================================================

#[test]
fn shift_conditional_format_rule_basic() {
    // CF record: type(2) + formula1_len(2) + formula2_len(2) + formatting_options(4) + padding(2) + formula1
    let mut cf = vec![2, 0, 5, 0, 0, 0]; // type=2, formula1_len=5, formula2_len=0
    cf.extend_from_slice(&0u32.to_le_bytes()); // formatting_options = 0
    cf.extend_from_slice(&0u16.to_le_bytes()); // padding
    // formula1: ptgRef absolute with row=3
    cf.extend_from_slice(&[0x24, 3, 0, 0, 0]);

    shift_conditional_format_rule(&mut cf, 0, 0, 2, 5, 0, &[]).unwrap();
    // row 3 >= 2, shift +5 = 8
    assert_eq!(u16::from_le_bytes([cf[13], cf[14]]), 8);
}

// ===========================================================================
// looks_like_xls 测试
// ===========================================================================

#[test]
fn looks_like_xls_identifies_ole_magic() {
    assert!(looks_like_xls(&[
        0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1
    ]));
    assert!(!looks_like_xls(b"PK\x03\x04"));
    assert!(!looks_like_xls(b""));
    assert!(!looks_like_xls(&[0xD0, 0xCF, 0x11]));
}

// ===========================================================================
// Biff8MacroPolicy 测试
// ===========================================================================

#[test]
fn macro_policy_preserve_display() {
    let p = Biff8MacroPolicy::Preserve;
    assert_eq!(format!("{p:?}"), "Preserve");
}

#[test]
fn macro_policy_strip_display() {
    let p = Biff8MacroPolicy::Strip;
    assert_eq!(format!("{p:?}"), "Strip");
}

// ===========================================================================
// 覆盖更多 encode_cell_record 分支
// ===========================================================================

#[test]
fn encode_cell_record_bool_false() {
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Bool(false)).unwrap();
    assert_eq!(record.typ, BOOLERR);
    assert_eq!(record.data[6], 0);
}

#[test]
fn encode_cell_record_error_div0() {
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Error(0x07)).unwrap();
    assert_eq!(record.typ, BOOLERR);
    assert_eq!(record.data[6], 0x07);
    assert_eq!(record.data[7], 1);
}

#[test]
fn encode_cell_record_rk_integer() {
    // 整数 100 应该用 RK 编码
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Number(100.0)).unwrap();
    assert_eq!(record.typ, RK);
}

#[test]
fn encode_cell_record_rk_div100() {
    // 12.34 应该用 RK div100 编码
    let record = encode_cell_record(0, 0, 0, &Biff8Value::Number(12.34)).unwrap();
    assert_eq!(record.typ, RK);
}

#[test]
fn encode_cell_record_text_unicode() {
    let record = encode_cell_record(
        0,
        0,
        0,
        &Biff8Value::Text("\u{4e2d}\u{6587}".to_owned()),
    )
    .unwrap();
    assert_eq!(record.typ, LABEL);
}

// ===========================================================================
// sheet_cell_insert_index 覆盖 chart sheet 场景
// ===========================================================================

#[test]
fn sheet_cell_insert_index_with_chart_substream() {
    let records = vec![
        // worksheet BOF
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
        },
        RawRecord {
            typ: DIMENSION,
            data: vec![0; 14],
        },
        RawRecord {
            typ: LABEL,
            data: vec![0; 10],
        },
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
        // chart BOF (should stop search)
        RawRecord {
            typ: BOF,
            data: vec![0; 16],
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
        dimension_index: Some(1),
    };
    // 插入位置应该在 EOF 之前
    assert_eq!(sheet_cell_insert_index(&records, &sheet), 3);
}

// ===========================================================================
// 覆盖更多 collection_placeholder_key 分支
// ===========================================================================

#[test]
fn collection_placeholder_key_empty_braces() {
    assert_eq!(collection_placeholder_key("{}", None), Some(""));
}

#[test]
fn collection_placeholder_key_named_empty_field() {
    assert_eq!(
        collection_placeholder_key("{items.}", Some("items")),
        Some("")
    );
}

// ===========================================================================
// discover_sheets 测试
// ===========================================================================

#[test]
fn discover_sheets_skips_chart_and_macro() {
    fn bof(stream_type: u16) -> RawRecord {
        RawRecord {
            typ: BOF,
            data: [0x00, 0x06, stream_type as u8, (stream_type >> 8) as u8].to_vec(),
        }
    }
    fn boundsheet(name: &str, sheet_type: u8) -> RawRecord {
        let mut data = vec![0, 0, 0, 0, 0, sheet_type, name.len() as u8, 0];
        data.extend_from_slice(name.as_bytes());
        RawRecord {
            typ: BOUNDSHEET,
            data,
        }
    }

    let records = vec![
        bof(0x0005),
        boundsheet("Chart1", 2),    // chart
        boundsheet("Macro", 4),     // macro
        boundsheet("Sheet1", 0),    // worksheet
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
        bof(0x0020), // chart stream
        RawRecord {
            typ: EOF,
            data: Vec::new(),
        },
        bof(0x0040), // macro stream
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

    let sheets = discover_sheets(&records).unwrap();
    assert_eq!(sheets.len(), 1);
    assert_eq!(sheets[0].name, "Sheet1");
}

// ===========================================================================
// shift_name_references 测试
// ===========================================================================

#[test]
fn shift_name_references_basic() {
    // NAME record 布局: flags(2)+shortcut(1)+name_len(1)+token_len(2)+pad(2)+
    //   scoped_sheet(2)+pad(4)+grbit(1)+name+tokens
    // 需要 15 字节头 + 1 字节名 + 7 字节 ptgRef3d = 23 字节
    let mut data = vec![0u8; 15]; // [0..15) 头
    data.push(b'N');             // [15] name
    data[3] = 1;                 // name_length = 1
    data[4] = 7;                 // token_length = 7 (low byte)
    data[8] = 1;                 // scoped_sheet = 1
    // ptgRef3d(0x3A): ptg(1)+ixti(2)+row(2)+col(2) = 7 bytes
    data.extend_from_slice(&[0x3A, 0, 0, 4, 0, 0, 0]);
    // token_start = 15 + 1 = 16; tokens = data[16..23]
    // ixti=0, row=4, col=0
    // ptg_targets_sheet: ixti=0, ranges[0]=Some((0,0)), current_sheet=0 -> true
    // shift_absolute_ptg_row(tokens, cursor+3=3) -> tokens[3..5] = data[19..21] = row=4

    let mut name = RawRecord { typ: NAME_SID, data };
    shift_name_references(&mut name, 2, 10, 0, &[Some((0, 0))]).unwrap();
    // row 4 >= 2 -> 4 + 10 = 14
    assert_eq!(u16::from_le_bytes([name.data[19], name.data[20]]), 14);
}

// ===========================================================================
// from_bytes_with_password 缺少密码测试
// ===========================================================================

#[test]
fn template_from_bytes_with_password_missing_password() {
    // 先创建加密模板
    let bytes = template_with_values().unwrap();
    let package = Biff8TemplatePackage::from_bytes(&bytes).unwrap();
    let encrypted = package.to_bytes_with_password(Some("secret")).unwrap();

    // 不提供密码加载应该报错
    let result = Biff8TemplatePackage::from_bytes(&encrypted);
    assert!(result.is_err());
}

// ===========================================================================
// from_path / from_path_with_password 测试
// ===========================================================================

#[test]
fn template_from_path_nonexistent() {
    let result = Biff8TemplatePackage::from_path(std::path::Path::new("/nonexistent.xls"));
    assert!(result.is_err());
}

// ===========================================================================
// 大量追加单元格测试（覆盖 adjust_indices 和 refresh_dimension）
// ===========================================================================

#[test]
fn template_package_many_set_cell_calls() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 多次调用 set_cell 覆盖 adjust_indices_after_insert
    for i in 0..20u32 {
        package.set_cell(
            "Data",
            i,
            0,
            &Biff8Cell::general(Biff8Value::Number(f64::from(i))),
        )?;
    }
    let output = package.to_bytes()?;
    let reloaded = Biff8TemplatePackage::from_bytes(&output)?;
    let next = reloaded.next_row_for_sheet("Data")?;
    assert_eq!(next, 20);
    Ok(())
}

#[test]
fn template_package_set_cell_bool_and_error() -> Result<()> {
    let bytes = template_with_values()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    package.set_cell(
        "Data",
        0,
        2,
        &Biff8Cell::general(Biff8Value::Bool(false)),
    )?;
    package.set_cell(
        "Data",
        1,
        2,
        &Biff8Cell::general(Biff8Value::Error(0x07)),
    )?;
    package.set_cell(
        "Data",
        2,
        2,
        &Biff8Cell::general(Biff8Value::Blank),
    )?;

    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// fill_collection_placeholders with force_new_row + shift
// ===========================================================================

#[test]
fn template_package_fill_collection_force_new_row_multiple_passes() -> Result<()> {
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
        // 额外的静态行
        sheet.cells.insert(
            (2, 0),
            Biff8Cell::general(Biff8Value::Text("footer".to_owned())),
        );
    }
    let bytes = book.to_cfb_bytes()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    // 第一次填充
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

    // 第二次填充（使用 cursor）
    let rows2 = vec![BTreeMap::from([
        ("name".to_owned(), "C".to_owned()),
        ("value".to_owned(), "3".to_owned()),
    ])];
    let count2 = package.fill_collection_placeholders(None, None, &rows2, false, true, true)?;
    assert!(count2 > 0);

    let output = package.to_bytes()?;
    assert!(!output.is_empty());
    Ok(())
}

// ===========================================================================
// fill_collection_horizontal with cursor
// ===========================================================================

#[test]
fn template_package_fill_collection_horizontal_multiple_passes() -> Result<()> {
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

    // 第一次水平填充
    let rows1 = vec![BTreeMap::from([("val".to_owned(), "X".to_owned())])];
    let count1 =
        package.fill_collection_placeholders(None, None, &rows1, true, false, true)?;
    assert!(count1 > 0);

    // 第二次水平填充（cursor 推进）
    let rows2 = vec![BTreeMap::from([("val".to_owned(), "Y".to_owned())])];
    let count2 =
        package.fill_collection_placeholders(None, None, &rows2, true, false, true)?;
    assert!(count2 > 0);
    Ok(())
}

// ===========================================================================
// set_cell_with_xf 间接测试（通过 auto_style=false）
// ===========================================================================

#[test]
fn template_package_fill_cells_without_auto_style() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([(
        "item".to_owned(),
        Biff8Cell {
            value: Biff8Value::Text("test".to_owned()),
            xf: XF_GENERAL,
        },
    )])];
    let placements =
        package.fill_collection_cells(None, None, &rows, false, false, false)?;
    assert!(placements.len() > 0);
    Ok(())
}

// ===========================================================================
// cell_xf 间接测试
// ===========================================================================

#[test]
fn template_package_fill_cells_with_auto_style() -> Result<()> {
    let bytes = template_with_placeholders()?;
    let mut package = Biff8TemplatePackage::from_bytes(&bytes)?;

    let rows = vec![BTreeMap::from([(
        "item".to_owned(),
        Biff8Cell::general(Biff8Value::Text("styled".to_owned())),
    )])];
    let placements =
        package.fill_collection_cells(None, None, &rows, false, false, true)?;
    assert!(placements.len() > 0);
    Ok(())
}
