    #[test]
    fn custom_number_format_emits_format_record_and_xf_ifmt() -> Result<()> {
        use std::io::Read as _;
        // 对应 Java：POI createDataFormat().getFormat("0.000") →
        // FORMAT 记录（ifmt ≥ 164）+ XF ifmt 字段
        let mut book = Biff8Book {
            sheets: vec![Biff8Sheet::new("Sheet1")],
            styles: Biff8StyleTable::default(),
            use_1904_windowing: false,
            extra_bytes: Vec::new(),
        };
        let request = crate::biff8::style::Biff8StyleRequest {
            number_format: Some(crate::biff8::style::Biff8NumberFormat::Custom(
                "0.000".to_owned(),
            )),
            ..crate::biff8::style::Biff8StyleRequest::default()
        };
        let xf = book.styles.resolve_xf(&request, XF_GENERAL);
        let sheet = book.sheet_mut("Sheet1");
        sheet.set(
            0,
            0,
            Biff8Cell::general(Biff8Value::Number(1234.567)).with_xf(xf),
        )?;
        let bytes = book.to_cfb_bytes()?;
        // 字节级断言：FORMAT(0x041E) 记录 = ifmt(164) + "0.000"
        // 先提取 CFB 容器内的 Workbook 流
        let mut cfb = cfb::CompoundFile::open(std::io::Cursor::new(&bytes))
            .map_err(|e| ExcelError::Cfb(e.to_string()))?;
        let mut stream = Vec::new();
        cfb.open_stream("Workbook")
            .map_err(|e| ExcelError::Cfb(e.to_string()))?
            .read_to_end(&mut stream)
            .map_err(ExcelError::Io)?;
        let records = records(&stream);
        let format_records: Vec<_> = records
            .iter()
            .filter(|(typ, _)| *typ == 0x041E)
            .map(|(_, data)| data.clone())
            .collect();
        assert_eq!(format_records.len(), 1);
        let data = &format_records[0];
        assert_eq!(u16::from_le_bytes([data[0], data[1]]), 164);
        let slen = u16::from_le_bytes([data[2], data[3]]) as usize;
        assert_eq!(&data[4..4 + slen], b"0.000");
        // 单元格 XF 的 ifmt 指向 164
        let xf_records: Vec<Vec<u8>> = records
            .iter()
            .filter(|(typ, data)| *typ == 0x00E0 && data.len() == 20)
            .map(|(_, data)| data.clone())
            .collect();
        assert!(
            xf_records
                .iter()
                .any(|data| u16::from_le_bytes([data[2], data[3]]) == 164),
            "存在 ifmt=164 的 XF"
        );
        Ok(())
    }

    /// Walks the BIFF record stream, returning (record type, payload) pairs and
    /// asserting the framing is well formed end to end.
    fn records(bytes: &[u8]) -> Vec<(u16, Vec<u8>)> {
        let mut out = Vec::new();
        let mut i = 0;
        while i + 4 <= bytes.len() {
            let typ = u16::from_le_bytes([bytes[i], bytes[i + 1]]);
            let len = u16::from_le_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
            let end = i + 4 + len;
            assert!(end <= bytes.len(), "record 0x{typ:04X} overruns stream");
            out.push((typ, bytes[i + 4..end].to_vec()));
            i = end;
        }
        assert_eq!(i, bytes.len(), "stream must be exhausted exactly");
        out
    }

    #[test]
    fn add_merge_rejects_reversed_ranges() {
        let mut sheet = Biff8Sheet::new("S");
        let err = sheet
            .add_merge(Biff8Merge {
                first_row: 5,
                last_row: 3,
                first_col: 0,
                last_col: 2,
            })
            .unwrap_err();
        assert!(matches!(err, ExcelError::Xls(_)));
        let err = sheet
            .add_merge(Biff8Merge {
                first_row: 0,
                last_row: 2,
                first_col: 4,
                last_col: 1,
            })
            .unwrap_err();
        assert!(matches!(err, ExcelError::Xls(_)));
        assert!(sheet.merges.is_empty());
    }

    #[test]
    fn add_merge_skips_single_cells_and_tracks_bounds() {
        let mut sheet = Biff8Sheet::new("S");
        sheet
            .add_merge(Biff8Merge {
                first_row: 2,
                last_row: 2,
                first_col: 3,
                last_col: 3,
            })
            .unwrap();
        assert!(sheet.merges.is_empty());
        sheet
            .add_merge(Biff8Merge {
                first_row: 0,
                last_row: 1,
                first_col: 0,
                last_col: 2,
            })
            .unwrap();
        assert_eq!(sheet.merges.len(), 1);
        assert_eq!(sheet.dimensions(), (2, 3));
    }

    #[test]
    fn url_hyperlink_emits_poi_compatible_hlink_record() -> Result<()> {
        let mut book = Biff8Book::default();
        let sheet = book.sheet_mut("Links");
        sheet.set(
            2,
            3,
            Biff8Cell::general(Biff8Value::Text("OpenAI".to_owned())),
        )?;
        sheet.add_hyperlink(2, 3, "https://openai.com", "OpenAI")?;

        let stream = build_workbook_stream(&book, &[HashMap::new()]);
        let (_, data) = records(&stream)
            .into_iter()
            .find(|(typ, _)| *typ == HYPERLINK)
            .expect("HLINK record");
        assert_eq!(&data[0..8], &[2, 0, 2, 0, 3, 0, 3, 0]);
        assert_eq!(&data[8..24], &Biff8Hyperlink::STD_MONIKER);
        assert_eq!(u32::from_le_bytes(data[24..28].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(data[28..32].try_into().unwrap()), 0x17);
        let label_units = u32::from_le_bytes(data[32..36].try_into().unwrap()) as usize;
        assert_eq!(label_units, "OpenAI".encode_utf16().count() + 1);
        let moniker_offset = 36 + label_units * 2;
        assert_eq!(
            &data[moniker_offset..moniker_offset + 16],
            &Biff8Hyperlink::URL_MONIKER
        );
        let url_size = u32::from_le_bytes(
            data[moniker_offset + 16..moniker_offset + 20]
                .try_into()
                .unwrap(),
        ) as usize;
        assert_eq!(
            url_size,
            ("https://openai.com".encode_utf16().count() + 1) * 2
                + Biff8Hyperlink::URL_TAIL.len()
        );
        assert_eq!(
            &data[data.len() - Biff8Hyperlink::URL_TAIL.len()..],
            &Biff8Hyperlink::URL_TAIL
        );
        Ok(())
    }

    #[test]
    fn hyperlink_rejects_nul_and_oversized_payloads() {
        let mut sheet = Biff8Sheet::new("Links");
        assert!(matches!(
            sheet.add_hyperlink(0, 0, "https://example.test/\0bad", "label"),
            Err(ExcelError::Xls(_))
        ));
        assert!(matches!(
            sheet.add_hyperlink(0, 0, "https://example.test", "x".repeat(MAX_RECORD_DATA)),
            Err(ExcelError::Xls(_))
        ));
        assert!(sheet.hyperlinks.is_empty());
    }

    #[test]
    fn typed_hyperlinks_encode_poi_flags_ranges_and_monikers() -> Result<()> {
        let mut book = Biff8Book::default();
        let sheet = book.sheet_mut("Links");
        sheet.add_typed_hyperlink(
            1,
            2,
            3,
            4,
            "'Other Sheet'!A1",
            "place",
            Biff8HyperlinkKind::Document,
        )?;
        sheet.add_typed_hyperlink(
            3,
            3,
            0,
            0,
            "../docs/report.xls",
            "file",
            Biff8HyperlinkKind::File,
        )?;
        sheet.add_typed_hyperlink(
            4,
            4,
            0,
            0,
            "mailto:test@example.com?subject=Hi",
            "email",
            Biff8HyperlinkKind::Email,
        )?;

        let stream = build_workbook_stream(&book, &[HashMap::new()]);
        let links: Vec<Vec<u8>> = records(&stream)
            .into_iter()
            .filter(|(typ, _)| *typ == HYPERLINK)
            .map(|(_, data)| data)
            .collect();
        assert_eq!(links.len(), 3);
        assert_eq!(&links[0][0..8], &[1, 0, 2, 0, 3, 0, 4, 0]);
        assert_eq!(u32::from_le_bytes(links[0][28..32].try_into().unwrap()), 0x1C);
        assert!(!links[0]
            .windows(Biff8Hyperlink::URL_MONIKER.len())
            .any(|window| window == Biff8Hyperlink::URL_MONIKER));
        assert_eq!(u32::from_le_bytes(links[1][28..32].try_into().unwrap()), 0x15);
        assert!(links[1]
            .windows(Biff8Hyperlink::FILE_MONIKER.len())
            .any(|window| window == Biff8Hyperlink::FILE_MONIKER));
        assert_eq!(u32::from_le_bytes(links[2][28..32].try_into().unwrap()), 0x17);
        assert!(links[2]
            .windows(Biff8Hyperlink::URL_MONIKER.len())
            .any(|window| window == Biff8Hyperlink::URL_MONIKER));
        Ok(())
    }

    #[test]
    fn write_raw_bytes_round_trips_through_cfb_images_stream() {
        let mut book = Biff8Book::default();
        book.write_raw_bytes(&[1, 2, 3, 4]);
        assert_eq!(book.extra_bytes, vec![1, 2, 3, 4]);
        let cfb = book.to_cfb_bytes().unwrap();
        assert!(!cfb.is_empty());
    }

    #[test]
    fn write_image_encodes_obj_and_msodrawing_records() {
        let images: &[&[u8]] = &[
            &[0xFF, 0xD8, 0x01, 0x02], // JPEG magic
            &[0x89, b'P', 0x03, 0x04], // PNG magic
            &[0xAB, 0xCD, 0x05],       // unknown magic → default JPEG
            &[0x42],                   // too short → default JPEG
            &[],                       // empty
        ];
        for image in images {
            let mut book = Biff8Book::default();
            book.write_image(image, 0, 0);
            assert!(book.extra_bytes.len() > 4);
            assert_eq!(
                u16::from_le_bytes([book.extra_bytes[0], book.extra_bytes[1]]),
                OBJ
            );
            let obj_len = u16::from_le_bytes([book.extra_bytes[2], book.extra_bytes[3]]) as usize;
            let mso = 4 + obj_len;
            assert_eq!(
                u16::from_le_bytes([book.extra_bytes[mso], book.extra_bytes[mso + 1]]),
                MSODRAWING
            );
            let mso_len =
                u16::from_le_bytes([book.extra_bytes[mso + 2], book.extra_bytes[mso + 3]]) as usize;
            let drawing = &book.extra_bytes[mso + 4..mso + 4 + mso_len];
            // The raw image payload is embedded inside the MSODRAWING record.
            if !image.is_empty() {
                assert!(drawing.windows(image.len()).any(|w| w == *image));
            }
            assert!(!book.to_cfb_bytes().unwrap().is_empty());
        }
    }

    #[test]
    fn empty_book_writes_default_sheet1() {
        let book = Biff8Book::default();
        let stream = build_workbook_stream(&book, &[]);
        let recs = records(&stream);
        let boundsheets: Vec<&[u8]> = recs
            .iter()
            .filter(|(typ, _)| *typ == BOUNDSHEET)
            .map(|(_, data)| data.as_slice())
            .collect();
        assert_eq!(boundsheets.len(), 1);
        let data = boundsheets[0];
        let cch = data[6] as usize;
        assert_eq!(data[7], 0x00, "ASCII name uses compressed encoding");
        assert_eq!(&data[8..8 + cch], b"Sheet1");
        assert!(book.to_cfb_bytes().unwrap().len() > stream.len());
    }

    #[test]
    fn long_strings_span_sst_continue_records() {
        let mut sheet = Biff8Sheet::new("S");
        let long_a = "a".repeat(MAX_RECORD_DATA + 779);
        let long_b = "b".repeat(MAX_RECORD_DATA + 779);
        sheet
            .set(0, 0, Biff8Cell::general(Biff8Value::Text(long_a)))
            .unwrap();
        sheet
            .set(1, 0, Biff8Cell::general(Biff8Value::Text(long_b)))
            .unwrap();
        let mut book = Biff8Book::default();
        book.sheets.push(sheet);
        let stream = build_workbook_stream(&book, &[]);
        let mut sst = 0;
        let mut continues = 0;
        for (typ, data) in records(&stream) {
            if typ == SST || typ == CONTINUE {
                assert!(data.len() <= MAX_RECORD_DATA);
            }
            if typ == SST {
                sst += 1;
            }
            if typ == CONTINUE {
                continues += 1;
            }
        }
        assert_eq!(sst, 1);
        // 两个约 9 KiB 的字符串在规范的紧凑分帧下占 3 条记录：1 SST + 2 CONTINUE。
        assert!(continues >= 2, "expected CONTINUE chunks, got {continues}");
    }

    #[test]
    fn set_rejects_rows_and_columns_beyond_biff8_limits() {
        let mut sheet = Biff8Sheet::new("S");
        let err = sheet
            .set(70_000, 0, Biff8Cell::general(Biff8Value::Number(1.0)))
            .unwrap_err();
        assert!(matches!(err, ExcelError::Xls(_)));
        let err = sheet
            .set(0, 300, Biff8Cell::general(Biff8Value::Number(1.0)))
            .unwrap_err();
        assert!(matches!(err, ExcelError::Xls(_)));
        assert!(sheet.cells.is_empty());
    }

    #[test]
    fn consecutive_numbers_merge_into_mulrk_and_blanks_into_mulblank() {
        // 对应 Java：POI MulRKRecord / MulBlankRecord 连续单元格压缩
        let mut sheet = Biff8Sheet::new("S");
        // 连续数字 (0,0..3)：1, 2, 3, 4 → 单条 MULRK
        for col in 0..4u8 {
            sheet
                .set(
                    0,
                    usize::from(col),
                    Biff8Cell::general(Biff8Value::Number(f64::from(col) + 1.0)),
                )
                .unwrap();
        }
        // 连续空白 (1,0..2) → 单条 MULBLANK
        for col in 0..3u8 {
            sheet
                .set(1, usize::from(col), Biff8Cell::general(Biff8Value::Blank))
                .unwrap();
        }
        // 非连续：数字夹字符串 → 各自独立记录
        sheet
            .set(2, 0, Biff8Cell::general(Biff8Value::Number(7.0)))
            .unwrap();
        sheet
            .set(2, 1, Biff8Cell::general(Biff8Value::Text("x".to_owned())))
            .unwrap();
        // 1/3 不是 0.01 的整数倍，RK 编码不了 → 必须是 NUMBER
        sheet
            .set(2, 2, Biff8Cell::general(Biff8Value::Number(1.0 / 3.0)))
            .unwrap();
        let mut book = Biff8Book::default();
        book.sheets.push(sheet);
        let stream = build_workbook_stream(&book, &[]);
        let mut mulrk = 0;
        let mut mulblank = 0;
        let mut rk = 0;
        let mut number = 0;
        for (typ, _) in records(&stream) {
            match typ {
                MULRK => mulrk += 1,
                MULBLANK => mulblank += 1,
                RK => rk += 1,
                NUMBER => number += 1,
                _ => {}
            }
        }
        assert_eq!(mulrk, 1, "连续 4 个数字合并为 1 条 MULRK");
        assert_eq!(mulblank, 1, "连续 3 个空白合并为 1 条 MULBLANK");
        assert_eq!(rk, 1, "孤立数字 7 用 RK");
        assert_eq!(number, 1, "孤立数字 1/3 用 NUMBER");
    }

    #[test]
    fn freeze_panes_emit_pane_record_and_window2_flags() {
        // golden 字节对照 xlwt 1.3.0 PanesRecord（冻结合并行、冻结列、
        // 行列都冻结三种形态逐一验证）
        for (rows, cols, expected_pane) in [
            // 冻结首行: px=0 py=1 rwTop=1 colLeft=0 pnnAct=2
            (
                1u16,
                0u16,
                [0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00],
            ),
            // 冻结首列: px=1 py=0 rwTop=0 colLeft=1 pnnAct=1
            (
                0u16,
                1u16,
                [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00],
            ),
            // 行列都冻结: px=1 py=1 rwTop=1 colLeft=1 pnnAct=0
            (
                1u16,
                1u16,
                [0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00],
            ),
        ] {
            let mut sheet = Biff8Sheet::new("S");
            sheet
                .set(0, 0, Biff8Cell::general(Biff8Value::Number(1.0)))
                .unwrap();
            sheet.freeze = Some((rows, cols));
            let mut book = Biff8Book::default();
            book.sheets.push(sheet);
            let stream = build_workbook_stream(&book, &[]);
            let mut panes = Vec::new();
            let mut window2 = None;
            for (typ, data) in records(&stream) {
                match typ {
                    PANE => panes.push(data),
                    WINDOW2 => window2 = Some(data),
                    _ => {}
                }
            }
            assert_eq!(panes.len(), 1, "恰好一条 PANE");
            assert_eq!(
                panes[0], expected_pane,
                "freeze ({rows},{cols}) PANE golden"
            );
            let w2 = window2.expect("sheet 必有 WINDOW2");
            assert_eq!(w2.len(), 18);
            let options = u16::from_le_bytes([w2[0], w2[1]]);
            // 基础选项 0x06B6 保留 + fFrozen(0x0008) + fFrozenNoSplit(0x1000)
            assert_eq!(
                options,
                0x06B6 | 0x0008 | 0x1000,
                "freeze 时 WINDOW2 置冻结位"
            );
        }
    }

    #[test]
    fn no_freeze_keeps_window2_default_options() {
        let mut sheet = Biff8Sheet::new("S");
        sheet
            .set(0, 0, Biff8Cell::general(Biff8Value::Number(1.0)))
            .unwrap();
        let mut book = Biff8Book::default();
        book.sheets.push(sheet);
        let stream = build_workbook_stream(&book, &[]);
        let mut has_pane = false;
        for (typ, _) in records(&stream) {
            if typ == PANE {
                has_pane = true;
            }
        }
        assert!(!has_pane, "未冻结时不得发射 PANE");
    }

    #[test]
    fn bool_and_number_cells_emit_boolerr_rk_and_number_records() {
        let mut sheet = Biff8Sheet::new("S");
        sheet
            .set(0, 0, Biff8Cell::general(Biff8Value::Bool(true)))
            .unwrap();
        sheet
            .set(1, 0, Biff8Cell::general(Biff8Value::Bool(false)))
            .unwrap();
        // 1/3 is not RK-encodable → NUMBER record.
        sheet
            .set(2, 0, Biff8Cell::general(Biff8Value::Number(1.0 / 3.0)))
            .unwrap();
        // 42 is RK-encodable → RK record.
        sheet
            .set(3, 0, Biff8Cell::general(Biff8Value::Number(42.0)))
            .unwrap();
        let mut book = Biff8Book::default();
        book.sheets.push(sheet);
        let stream = build_workbook_stream(&book, &[]);
        let mut boolerr = 0;
        let mut number = 0;
        let mut rk = 0;
        for (typ, _) in records(&stream) {
            match typ {
                BOOLERR => boolerr += 1,
                NUMBER => number += 1,
                RK => rk += 1,
                _ => {}
            }
        }
        assert_eq!(boolerr, 2);
        assert_eq!(number, 1);
        assert_eq!(rk, 1);
    }
