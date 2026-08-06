    #[test]
    fn writes_openable_container() {
        let mut wb = Workbook::empty();
        let mut s = Sheet::new("Sheet1");
        s.set(0, 0, Cell::Number(1.0));
        s.set(0, 1, Cell::Text("hi".into()));
        wb.sheets.push(s);
        let mut buf = Vec::new();
        write(&wb, Cursor::new(&mut buf)).unwrap();
        // Should be a valid CFB.
        assert!(super::super::looks_like_cfb(&buf));
        cfb::CompoundFile::open(Cursor::new(&buf)).expect("valid cfb");
    }

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
    fn consecutive_numbers_merge_into_mulrk_and_blanks_into_mulblank() {
        // 对应 POI MulRKRecord / MulBlankRecord：连续数字/空白压缩。
        let mut sheet = Sheet::new("S");
        // 行 0：1,2,3,4 → 单条 MULRK
        for col in 0..4u32 {
            sheet.set(0, col, Cell::Number(f64::from(col) + 1.0));
        }
        // 行 1：连续 3 个空白 → 单条 MULBLANK
        // （稀疏模型里无样式 Empty 不落盘；显式空白等价于带样式空单元格）
        for col in 0..3u32 {
            sheet.cells.insert((1, col), Cell::Empty);
        }
        // 行 2：数字夹字符串 → 各自独立（7 可 RK；1/3 不可 RK → NUMBER）
        sheet.set(2, 0, Cell::Number(7.0));
        sheet.set(2, 1, Cell::Text("x".to_owned()));
        sheet.set(2, 2, Cell::Number(1.0 / 3.0));

        let mut wb = Workbook::empty();
        wb.sheets.push(sheet);
        let mut substream = Vec::new();
        let sst = HashMap::new();
        write_worksheet(&mut substream, &wb.sheets[0], &wb, &sst);

        let mut mulrk = 0;
        let mut mulblank = 0;
        let mut rk = 0;
        let mut number = 0;
        for (typ, data) in records(&substream) {
            match typ {
                biff::MULRK => {
                    mulrk += 1;
                    // rw(2) + colFirst(2) + (xf,rk)*4 + colLast(2) = 4 + 24 + 2
                    assert_eq!(data.len(), 4 + 4 * 6 + 2, "4 格 MULRK");
                    // colLast == colFirst + 3
                    let col_first = u16::from_le_bytes([data[2], data[3]]);
                    let col_last = u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]]);
                    assert_eq!(col_last, col_first + 3);
                }
                biff::MULBLANK => {
                    mulblank += 1;
                    assert_eq!(data.len(), 4 + 3 * 2 + 2, "3 格 MULBLANK");
                }
                biff::RK => rk += 1,
                biff::NUMBER => number += 1,
                _ => {}
            }
        }
        assert_eq!(mulrk, 1, "连续 4 个数字合并为 1 条 MULRK");
        assert_eq!(mulblank, 1, "连续 3 个空白合并为 1 条 MULBLANK");
        assert_eq!(rk, 1, "孤立数字 7 用 RK");
        assert_eq!(number, 1, "孤立数字 1/3 用 NUMBER");
    }

    #[test]
    fn mixed_row_keeps_isolated_records() {
        // 无连续数字/空白时退化为逐格记录（与旧行为一致）。
        let mut sheet = Sheet::new("S");
        sheet.set(0, 0, Cell::Number(1.0));
        sheet.set(0, 1, Cell::Text("t".to_owned()));
        sheet.set(0, 2, Cell::Number(2.0));
        let mut wb = Workbook::empty();
        wb.sheets.push(sheet);
        let mut substream = Vec::new();
        let sst = HashMap::new();
        write_worksheet(&mut substream, &wb.sheets[0], &wb, &sst);

        let mut mulrk = 0;
        let mut mulblank = 0;
        let mut rk = 0;
        let mut labelsst = 0;
        for (typ, _) in records(&substream) {
            match typ {
                biff::MULRK => mulrk += 1,
                biff::MULBLANK => mulblank += 1,
                biff::RK => rk += 1,
                biff::LABELSST => labelsst += 1,
                _ => {}
            }
        }
        assert_eq!(mulrk, 0, "无连续数字不合并");
        assert_eq!(mulblank, 0, "无连续空白不合并");
        assert_eq!(rk, 2, "两个孤立数字各一条 RK");
        assert_eq!(labelsst, 1, "文本独立 LABELSST");
    }

    #[test]
    fn pane_record_matches_xlwt_semantics() {
        // golden 字节对照 xlwt 1.3.0 PanesRecord：px/py/rwTop/colLeft/pnnAct。
        for (rows, cols, expected) in [
            // 冻结首行: px=0 py=1 rwTop=1 colLeft=0 pnnAct=2
            (
                1u32,
                0u32,
                [0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00],
            ),
            // 冻结首列: px=1 py=0 rwTop=0 colLeft=1 pnnAct=1
            (
                0u32,
                1u32,
                [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00],
            ),
            // 行列都冻结: px=1 py=1 rwTop=1 colLeft=1 pnnAct=0
            (
                1u32,
                1u32,
                [0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00],
            ),
        ] {
            let mut sheet = Sheet::new("S");
            sheet.set(0, 0, Cell::Number(1.0));
            sheet.frozen.rows = rows;
            sheet.frozen.cols = cols;
            let mut wb = Workbook::empty();
            wb.sheets.push(sheet);
            let mut substream = Vec::new();
            let sst = HashMap::new();
            write_worksheet(&mut substream, &wb.sheets[0], &wb, &sst);

            let pane = records(&substream)
                .into_iter()
                .find(|(typ, _)| *typ == biff::PANE)
                .map(|(_, data)| data)
                .unwrap_or_default();
            assert_eq!(
                &pane[..],
                &expected[..],
                "freeze ({rows},{cols}) PANE golden"
            );
        }
    }
