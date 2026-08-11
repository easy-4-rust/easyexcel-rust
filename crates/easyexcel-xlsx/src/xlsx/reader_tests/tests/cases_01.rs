    #[test]
    fn cells_values_formulas() {
        let mut wb = Workbook::empty();
        let mut sheet = Sheet::new("Data");
        sheet.set(0, 0, Cell::Number(42.5));
        sheet.set(0, 1, Cell::Text("hello".into()));
        sheet.set(0, 2, Cell::Text(" leading space ".into()));
        sheet.set(1, 0, Cell::Bool(true));
        sheet.set(1, 1, Cell::Error(CellError::Div0));
        sheet.set(
            2,
            0,
            Cell::Formula {
                expr: "A1*2".into(),
                cached: CellValue::Number(85.0),
            },
        );
        sheet.set(
            2,
            1,
            Cell::Formula {
                expr: "B1&\"!\"".into(),
                cached: CellValue::Text("hello!".into()),
            },
        );
        wb.sheets.push(sheet);

        let out = roundtrip(&wb);
        let s = &out.sheets[0];
        assert_eq!(s.name, "Data");
        assert_eq!(s.value(0, 0), CellValue::Number(42.5));
        assert_eq!(s.value(0, 1), CellValue::Text("hello".into()));
        assert_eq!(s.value(0, 2), CellValue::Text(" leading space ".into()));
        assert_eq!(s.value(1, 0), CellValue::Bool(true));
        assert_eq!(s.value(1, 1), CellValue::Error(CellError::Div0));
        match s.get(2, 0) {
            Some(Cell::Formula { expr, cached }) => {
                assert_eq!(expr, "A1*2");
                assert_eq!(*cached, CellValue::Number(85.0));
            }
            other => panic!("expected formula, got {other:?}"),
        }
        match s.get(2, 1) {
            Some(Cell::Formula { expr, cached }) => {
                assert_eq!(expr, "B1&\"!\"");
                assert_eq!(*cached, CellValue::Text("hello!".into()));
            }
            other => panic!("expected formula, got {other:?}"),
        }
    }

    #[test]
    fn merged_and_frozen() {
        let mut wb = Workbook::empty();
        let mut sheet = Sheet::new("S1");
        sheet.set(0, 0, Cell::Text("hdr".into()));
        sheet.merged.push(CellRange::parse_a1("A1:C1").unwrap());
        sheet.frozen = FrozenPanes { rows: 1, cols: 2 };
        wb.sheets.push(sheet);

        let out = roundtrip(&wb);
        let s = &out.sheets[0];
        assert_eq!(s.merged.len(), 1);
        assert_eq!(s.merged[0], CellRange::parse_a1("A1:C1").unwrap());
        assert_eq!(s.frozen, FrozenPanes { rows: 1, cols: 2 });
    }

    #[test]
    fn styles_and_numfmt() {
        let mut wb = Workbook::empty();
        let mut sheet = Sheet::new("Styled");

        let bold_idx = {
            let mut st = CellStyle::default();
            st.font.bold = true;
            st.halign = HAlign::Center;
            wb.styles.intern(st)
        };
        let date_idx = {
            let st = CellStyle {
                number_format: "yyyy-mm-dd".into(),
                ..Default::default()
            };
            wb.styles.intern(st)
        };
        let pct_idx = {
            let st = CellStyle {
                number_format: "0.00%".into(),
                number_format_id: Some(10),
                ..Default::default()
            };
            wb.styles.intern(st)
        };

        sheet.set(0, 0, Cell::Text("title".into()));
        sheet.set_style(0, 0, bold_idx);
        sheet.set(1, 0, Cell::Number(44000.0));
        sheet.set_style(1, 0, date_idx);
        sheet.set(2, 0, Cell::Number(0.5));
        sheet.set_style(2, 0, pct_idx);
        wb.sheets.push(sheet);

        let out = roundtrip(&wb);
        let s = &out.sheets[0];

        let bs = out.styles.get(s.style_at(0, 0).unwrap()).unwrap();
        assert!(bs.font.bold);
        assert_eq!(bs.halign, HAlign::Center);

        let ds = out.styles.get(s.style_at(1, 0).unwrap()).unwrap();
        assert_eq!(ds.number_format, "yyyy-mm-dd");
        assert!(ds.is_date());

        let ps = out.styles.get(s.style_at(2, 0).unwrap()).unwrap();
        assert_eq!(ps.number_format, "0.00%");
    }

    #[test]
    fn date1904_flag() {
        let mut wb = Workbook::empty();
        wb.date_system = DateSystem::Date1904;
        wb.sheets.push(Sheet::new("Sheet1"));
        let out = roundtrip(&wb);
        assert_eq!(out.date_system, DateSystem::Date1904);
    }

    #[test]
    fn multi_sheet_and_visibility() {
        let mut wb = Workbook::empty();
        let mut s1 = Sheet::new("First");
        s1.set(0, 0, Cell::Number(1.0));
        let mut s2 = Sheet::new("Hidden");
        s2.visibility = Visibility::Hidden;
        s2.set(0, 0, Cell::Number(2.0));
        wb.sheets.push(s1);
        wb.sheets.push(s2);

        let out = roundtrip(&wb);
        assert_eq!(out.sheets.len(), 2);
        assert_eq!(out.sheets[0].name, "First");
        assert_eq!(out.sheets[1].name, "Hidden");
        assert_eq!(out.sheets[1].visibility, Visibility::Hidden);
        assert_eq!(out.sheets[1].value(0, 0), CellValue::Number(2.0));
    }

    #[test]
    fn defined_names_roundtrip() {
        let mut wb = Workbook::empty();
        wb.sheets.push(Sheet::new("Sheet1"));
        wb.defined_names.push(DefinedName {
            name: "MyRange".into(),
            refers_to: "Sheet1!$A$1:$B$2".into(),
            scope: None,
            hidden: false,
        });
        let out = roundtrip(&wb);
        assert_eq!(out.defined_names.len(), 1);
        assert_eq!(out.defined_names[0].name, "MyRange");
        assert_eq!(out.defined_names[0].refers_to, "Sheet1!$A$1:$B$2");
    }

    #[test]
    fn opaque_part_roundtrip() {
        let mut wb = Workbook::empty();
        wb.sheets.push(Sheet::new("Sheet1"));
        wb.opaque.push(OpaquePart {
            name: "xl/theme/theme1.xml".into(),
            data: b"<theme>x</theme>".to_vec(),
        });
        let out = roundtrip(&wb);
        let theme = out.opaque.iter().find(|p| p.name == "xl/theme/theme1.xml");
        assert!(theme.is_some(), "theme part should round-trip opaquely");
        assert_eq!(theme.unwrap().data, b"<theme>x</theme>");
    }

    #[test]
    fn password_detection() {
        // A minimal zip containing an EncryptedPackage entry triggers the error.
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
            zip.start_file("EncryptedPackage", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"junk").unwrap();
            zip.finish().unwrap();
        }
        let err = read(Cursor::new(buf)).unwrap_err();
        assert!(matches!(err, Error::PasswordProtected(_)));
    }

    #[test]
    fn zip_bomb_single_entry_rejected() {
        // 单个高压缩比 entry 超过 max_file_bytes 时应被拒绝。
        // 构造 1MB 全零数据（压缩后约 1KB），设置 100KB 限制。
        let bomb = make_zip_bomb(1024 * 1024);
        let limits = ResourceLimits::new(100 * 1024, 256, 2_000_000, 500_000);
        let err = read_with_limits(Cursor::new(bomb), limits).unwrap_err();
        assert!(
            matches!(err, Error::ResourceLimit { resource, .. } if resource == "zip_entry_uncompressed_bytes"),
            "应拒绝单个超大 ZIP entry，实际错误: {err:?}"
        );
    }

    #[test]
    fn zip_bomb_multi_entry_rejected() {
        // 多个小 entry 累积超过 max_file_bytes 时应被拒绝。
        // 构造 10 个 100KB 全零 entry（总计 1MB），设置 500KB 限制。
        let bomb = make_zip_bomb_multi_entry(10, 100 * 1024);
        let limits = ResourceLimits::new(500 * 1024, 256, 2_000_000, 500_000);
        let err = read_with_limits(Cursor::new(bomb), limits).unwrap_err();
        assert!(
            matches!(err, Error::ResourceLimit { resource, .. } if resource == "zip_total_uncompressed_bytes"),
            "应拒绝累积超限的 ZIP entries，实际错误: {err:?}"
        );
    }

    #[test]
    fn zip_bomb_default_limits_allow_normal_file() {
        // 默认 256MB 限制不应拒绝正常的 round-trip 文件。
        let mut wb = Workbook::empty();
        let mut sheet = Sheet::new("Sheet1");
        sheet.set(0, 0, Cell::Number(42.0));
        wb.sheets.push(sheet);

        let mut buf = Vec::new();
        write(&wb, Cursor::new(&mut buf)).expect("write");
        let out = read_with_limits(Cursor::new(buf), ResourceLimits::default()).expect("read");
        assert_eq!(out.sheets.len(), 1);
        assert_eq!(out.sheets[0].value(0, 0), CellValue::Number(42.0));
    }

    #[test]
    fn zip_bomb_with_password_and_limits_rejected() {
        // 加密路径也应受资源限制保护。
        let bomb = make_zip_bomb(1024 * 1024);
        let limits = ResourceLimits::new(100 * 1024, 256, 2_000_000, 500_000);
        // 非 CFB 格式（纯 ZIP），password 会被忽略，走 read_zip_with_limits 路径
        let err = read_with_password_and_limits(Cursor::new(bomb), None, limits).unwrap_err();
        assert!(
            matches!(err, Error::ResourceLimit { resource, .. } if resource == "zip_entry_uncompressed_bytes"),
            "加密路径也应拒绝 ZIP bomb，实际错误: {err:?}"
        );
    }

    // ── build_cell 各分支覆盖 ──────────────────────────────────────────────

    #[test]
    fn build_cell_shared_string_type() {
        let shared = vec!["hello".into(), "world".into()];
        // t="s"，共享字符串索引 1
        let cell = build_cell("s", "1", "", "", false, &shared);
        assert_eq!(cell, Cell::Text("world".into()));
    }

    #[test]
    fn build_cell_shared_string_out_of_range() {
        let shared = vec!["only".into()];
        // 索引 99 超出范围，返回空字符串
        let cell = build_cell("s", "99", "", "", false, &shared);
        assert_eq!(cell, Cell::Text(String::new()));
    }

    #[test]
    fn build_cell_shared_string_invalid_index() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("s", "not_a_number", "", "", false, &shared);
        assert_eq!(cell, Cell::Text(String::new()));
    }

    #[test]
    fn build_cell_str_type() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("str", "raw text", "", "", false, &shared);
        assert_eq!(cell, Cell::Text("raw text".into()));
    }

    #[test]
    fn build_cell_inline_string_type() {
        let shared: Vec<String> = vec![];
        // inlineStr 优先使用 inline 参数
        let cell = build_cell("inlineStr", "raw", "", "inline text", false, &shared);
        assert_eq!(cell, Cell::Text("inline text".into()));
    }

    #[test]
    fn build_cell_inline_string_empty_inline() {
        let shared: Vec<String> = vec![];
        // inlineStr 类型：DOM reader 直接使用 inline 参数（无回退到 raw_value）
        // build_cell(t, v, f, inline, has_formula, shared)
        let cell = build_cell("inlineStr", "raw", "", "", false, &shared);
        // inline 为空时返回空字符串（DOM reader 不做回退）
        assert_eq!(cell, Cell::Text(String::new()));
    }

    #[test]
    fn build_cell_bool_true() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("b", "1", "", "", false, &shared);
        assert_eq!(cell, Cell::Bool(true));
    }

    #[test]
    fn build_cell_bool_false() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("b", "0", "", "", false, &shared);
        assert_eq!(cell, Cell::Bool(false));
    }

    #[test]
    fn build_cell_error_type() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("e", "#DIV/0!", "", "", false, &shared);
        assert_eq!(cell, Cell::Error(CellError::Div0));
    }

    #[test]
    fn build_cell_error_unknown_fallback() {
        let shared: Vec<String> = vec![];
        // 无法解析的错误文本回退到 Value
        let cell = build_cell("e", "UNKNOWN_ERR", "", "", false, &shared);
        assert_eq!(cell, Cell::Error(CellError::Value));
    }

    #[test]
    fn build_cell_error_na() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("e", "#N/A", "", "", false, &shared);
        assert_eq!(cell, Cell::Error(CellError::NA));
    }

    #[test]
    fn build_cell_number_type() {
        let shared: Vec<String> = vec![];
        // t 为空或 "n" 时按数字解析
        let cell = build_cell("", "3.14", "", "", false, &shared);
        assert_eq!(cell, Cell::Number(3.14));
    }

    #[test]
    fn build_cell_number_n_type() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("n", "42", "", "", false, &shared);
        assert_eq!(cell, Cell::Number(42.0));
    }

    #[test]
    fn build_cell_empty_value() {
        let shared: Vec<String> = vec![];
        // t 为空且 v 为空时返回 Empty
        let cell = build_cell("", "", "", "", false, &shared);
        assert_eq!(cell, Cell::Empty);
    }

    #[test]
    fn build_cell_non_numeric_fallback_to_text() {
        let shared: Vec<String> = vec![];
        // 数字类型但值无法解析为 f64 时回退为 Text
        let cell = build_cell("", "not_a_number", "", "", false, &shared);
        assert_eq!(cell, Cell::Text("not_a_number".into()));
    }

    #[test]
    fn build_cell_formula_with_shared_string() {
        let shared = vec!["cached".into()];
        let cell = build_cell("s", "0", "A1+B1", "", true, &shared);
        match cell {
            Cell::Formula { expr, cached } => {
                assert_eq!(expr, "A1+B1");
                assert_eq!(cached, CellValue::Text("cached".into()));
            }
            other => panic!("expected formula, got {other:?}"),
        }
    }

    #[test]
    fn build_cell_formula_with_str_cached() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("str", "result", "A1", "", true, &shared);
        match cell {
            Cell::Formula { expr, cached } => {
                assert_eq!(expr, "A1");
                assert_eq!(cached, CellValue::Text("result".into()));
            }
            other => panic!("expected formula, got {other:?}"),
        }
    }

    #[test]
    fn build_cell_formula_with_bool_cached() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("b", "1", "TRUE()", "", true, &shared);
        match cell {
            Cell::Formula { cached, .. } => {
                assert_eq!(cached, CellValue::Bool(true));
            }
            other => panic!("expected formula, got {other:?}"),
        }
    }

    #[test]
    fn build_cell_formula_with_error_cached() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("e", "#N/A", "A1", "", true, &shared);
        match cell {
            Cell::Formula { cached, .. } => {
                assert_eq!(cached, CellValue::Error(CellError::NA));
            }
            other => panic!("expected formula, got {other:?}"),
        }
    }

    #[test]
    fn build_cell_formula_with_number_cached() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("", "99.5", "SUM(A1:A10)", "", true, &shared);
        match cell {
            Cell::Formula { cached, .. } => {
                assert_eq!(cached, CellValue::Number(99.5));
            }
            other => panic!("expected formula, got {other:?}"),
        }
    }

    #[test]
    fn build_cell_formula_with_empty_cached() {
        let shared: Vec<String> = vec![];
        let cell = build_cell("", "", "A1", "", true, &shared);
        match cell {
            Cell::Formula { cached, .. } => {
                assert_eq!(cached, CellValue::Empty);
            }
            other => panic!("expected formula, got {other:?}"),
        }
    }

    // ── normalize_part_path / normalize_rel_path 覆盖 ──────────────────────

    #[test]
    fn normalize_part_path_handles_absolute_target() {
        // 以 "/" 开头的绝对路径直接去掉前缀
        assert_eq!(normalize_part_path("/xl/worksheets/sheet1.xml"), "xl/worksheets/sheet1.xml");
    }

    #[test]
    fn normalize_part_path_handles_relative_target() {
        assert_eq!(normalize_part_path("worksheets/sheet1.xml"), "xl/worksheets/sheet1.xml");
    }

    #[test]
    fn normalize_part_path_handles_dot_segments() {
        assert_eq!(normalize_part_path("./worksheets/sheet1.xml"), "xl/worksheets/sheet1.xml");
    }

    #[test]
    fn normalize_part_path_handles_parent_segments() {
        assert_eq!(normalize_part_path("../drawings/drawing1.xml"), "drawings/drawing1.xml");
    }

    #[test]
    fn normalize_rel_path_handles_absolute_target() {
        assert_eq!(normalize_rel_path("xl", "/styles.xml"), "styles.xml");
    }

    #[test]
    fn normalize_rel_path_handles_relative_target() {
        assert_eq!(normalize_rel_path("xl/_rels", "worksheets/sheet1.xml"), "xl/_rels/worksheets/sheet1.xml");
    }

    #[test]
    fn normalize_rel_path_handles_parent_traversal() {
        assert_eq!(normalize_rel_path("xl/worksheets/_rels", "../drawings/drawing1.xml"), "xl/worksheets/drawings/drawing1.xml");
    }

    #[test]
    fn normalize_rel_path_handles_empty_base() {
        assert_eq!(normalize_rel_path("", "xl/workbook.xml"), "xl/workbook.xml");
    }

    #[test]
    fn normalize_rel_path_handles_dot_and_empty_segments() {
        assert_eq!(normalize_rel_path("xl", "./worksheets/./sheet1.xml"), "xl/worksheets/sheet1.xml");
    }

    // ── is_known_part 覆盖 ─────────────────────────────────────────────────

    #[test]
    fn is_known_part_recognizes_content_types() {
        assert!(is_known_part("[Content_Types].xml"));
    }

    #[test]
    fn is_known_part_recognizes_rels() {
        assert!(is_known_part("_rels/.rels"));
    }

    #[test]
    fn is_known_part_recognizes_workbook() {
        assert!(is_known_part("xl/workbook.xml"));
        assert!(is_known_part("xl/_rels/workbook.xml.rels"));
    }

    #[test]
    fn is_known_part_recognizes_shared_strings() {
        assert!(is_known_part("xl/sharedStrings.xml"));
    }

    #[test]
    fn is_known_part_recognizes_styles() {
        assert!(is_known_part("xl/styles.xml"));
    }

    #[test]
    fn is_known_part_recognizes_calc_chain() {
        assert!(is_known_part("xl/calcChain.xml"));
    }

    #[test]
    fn is_known_part_recognizes_doc_props() {
        assert!(is_known_part("docProps/core.xml"));
        assert!(is_known_part("docProps/app.xml"));
    }

    #[test]
    fn is_known_part_recognizes_worksheet_xml() {
        assert!(is_known_part("xl/worksheets/sheet1.xml"));
        assert!(is_known_part("xl/worksheets/sheet2.xml"));
    }

    #[test]
    fn is_known_part_rejects_worksheet_rels() {
        assert!(!is_known_part("xl/worksheets/_rels/sheet1.xml.rels"));
    }

    #[test]
    fn is_known_part_rejects_unknown_parts() {
        assert!(!is_known_part("xl/theme/theme1.xml"));
        assert!(!is_known_part("xl/drawings/drawing1.xml"));
        assert!(!is_known_part("xl/media/image1.png"));
    }

    // ── parse_rels 覆盖 ────────────────────────────────────────────────────

    #[test]
    fn parse_rels_extracts_relationships() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
                <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
                <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
            </Relationships>"#;
        let rels = parse_rels(xml).expect("parse_rels");
        assert_eq!(rels.len(), 2);
        assert_eq!(rels.get("rId1").unwrap(), "worksheets/sheet1.xml");
        assert_eq!(rels.get("rId2").unwrap(), "styles.xml");
    }

    #[test]
    fn parse_rels_handles_empty_input() {
        let xml = b"<Relationships/>";
        let rels = parse_rels(xml).expect("parse_rels");
        assert!(rels.is_empty());
    }

    // ── parse_workbook 覆盖 ────────────────────────────────────────────────

    #[test]
    fn parse_workbook_extracts_sheets() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheets>
                    <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
                    <sheet name="Sheet2" sheetId="2" r:id="rId2"/>
                </sheets>
            </workbook>"#;
        let info = parse_workbook(xml).expect("parse_workbook");
        assert_eq!(info.sheets.len(), 2);
        assert_eq!(info.sheets[0].name, "Sheet1");
        assert_eq!(info.sheets[0].rid, "rId1");
        assert_eq!(info.sheets[1].name, "Sheet2");
        assert_eq!(info.date_system, DateSystem::Date1900);
    }

    #[test]
    fn parse_workbook_detects_date1904() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <workbookPr date1904="1"/>
                <sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>
            </workbook>"#;
        let info = parse_workbook(xml).expect("parse_workbook");
        assert_eq!(info.date_system, DateSystem::Date1904);
    }

    #[test]
    fn parse_workbook_detects_date1904_true_string() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <workbookPr date1904="true"/>
                <sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>
            </workbook>"#;
        let info = parse_workbook(xml).expect("parse_workbook");
        assert_eq!(info.date_system, DateSystem::Date1904);
    }

    #[test]
    fn parse_workbook_extracts_defined_names() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheets><sheet name="S" sheetId="1" r:id="rId1"/></sheets>
                <definedNames>
                    <definedName name="MyRange" localSheetId="0" hidden="1">Sheet1!$A$1:$B$2</definedName>
                </definedNames>
            </workbook>"#;
        let info = parse_workbook(xml).expect("parse_workbook");
        assert_eq!(info.defined_names.len(), 1);
        assert_eq!(info.defined_names[0].name, "MyRange");
        assert_eq!(info.defined_names[0].refers_to, "Sheet1!$A$1:$B$2");
        assert_eq!(info.defined_names[0].scope, Some(0));
        assert!(info.defined_names[0].hidden);
    }

    #[test]
    fn parse_workbook_handles_sheet_visibility() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheets>
                    <sheet name="Visible" sheetId="1" r:id="rId1"/>
                    <sheet name="Hidden" sheetId="2" r:id="rId2" state="hidden"/>
                    <sheet name="VeryHidden" sheetId="3" r:id="rId3" state="veryHidden"/>
                </sheets>
            </workbook>"#;
        let info = parse_workbook(xml).expect("parse_workbook");
        assert_eq!(info.sheets.len(), 3);
        assert_eq!(info.sheets[0].visibility, Visibility::Visible);
        assert_eq!(info.sheets[1].visibility, Visibility::Hidden);
        assert_eq!(info.sheets[2].visibility, Visibility::VeryHidden);
    }

    // ── parse_worksheet 覆盖 ───────────────────────────────────────────────

    #[test]
    fn parse_worksheet_handles_shared_string_cell() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData>
                    <row r="1"><c r="A1" t="s"><v>0</v></c></row>
                </sheetData>
            </worksheet>"#;
        let shared = vec!["hello".to_string()];
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &shared, &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.value(0, 0), CellValue::Text("hello".into()));
    }

    #[test]
    fn parse_worksheet_handles_number_cell() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData>
                    <row r="1"><c r="A1"><v>42.5</v></c></row>
                </sheetData>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.value(0, 0), CellValue::Number(42.5));
    }

    #[test]
    fn parse_worksheet_handles_bool_cell() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData>
                    <row r="1"><c r="A1" t="b"><v>1</v></c></row>
                </sheetData>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.value(0, 0), CellValue::Bool(true));
    }

    #[test]
    fn parse_worksheet_handles_error_cell() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData>
                    <row r="1"><c r="A1" t="e"><v>#DIV/0!</v></c></row>
                </sheetData>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.value(0, 0), CellValue::Error(CellError::Div0));
    }

    #[test]
    fn parse_worksheet_handles_inline_string() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData>
                    <row r="1"><c r="A1" t="inlineStr"><is><t>inline text</t></is></c></row>
                </sheetData>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.value(0, 0), CellValue::Text("inline text".into()));
    }

    #[test]
    fn parse_worksheet_handles_str_type_cell() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData>
                    <row r="1"><c r="A1" t="str"><v>formula result</v></c></row>
                </sheetData>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.value(0, 0), CellValue::Text("formula result".into()));
    }

    #[test]
    fn parse_worksheet_handles_formula_cell() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData>
                    <row r="1"><c r="A1"><f>A1*2</f><v>10</v></c></row>
                </sheetData>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        match sheet.get(0, 0) {
            Some(Cell::Formula { expr, cached }) => {
                assert_eq!(expr, "A1*2");
                assert_eq!(*cached, CellValue::Number(10.0));
            }
            other => panic!("expected formula, got {other:?}"),
        }
    }

    #[test]
    fn parse_worksheet_handles_empty_cell() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData>
                    <row r="1"><c r="A1"/></row>
                </sheetData>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.value(0, 0), CellValue::Empty);
    }

    #[test]
    fn parse_worksheet_handles_merge_cells() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData/>
                <mergeCells>
                    <mergeCell ref="A1:C1"/>
                    <mergeCell ref="A2:B3"/>
                </mergeCells>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.merged.len(), 2);
        assert_eq!(sheet.merged[0], CellRange::parse_a1("A1:C1").unwrap());
        assert_eq!(sheet.merged[1], CellRange::parse_a1("A2:B3").unwrap());
    }

    #[test]
    fn parse_worksheet_handles_frozen_pane() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetViews><sheetView>
                    <pane xSplit="2" ySplit="1" state="frozen"/>
                </sheetView></sheetViews>
                <sheetData/>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.frozen, FrozenPanes { rows: 1, cols: 2 });
    }

    #[test]
    fn parse_worksheet_ignores_non_frozen_pane() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetViews><sheetView>
                    <pane xSplit="2" ySplit="1" state="split"/>
                </sheetView></sheetViews>
                <sheetData/>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.frozen, FrozenPanes { rows: 0, cols: 0 });
    }

    #[test]
    fn parse_worksheet_handles_column_info() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <cols>
                    <col min="1" max="3" width="20" hidden="1"/>
                    <col min="5" max="5" width="40"/>
                </cols>
                <sheetData/>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        // col 1-3 (0-based 0-2) hidden, width 20
        assert!(sheet.columns.get(&0).unwrap().hidden);
        assert_eq!(sheet.columns.get(&0).unwrap().width, Some(20.0));
        assert!(sheet.columns.get(&2).unwrap().hidden);
        // col 5 (0-based 4) visible, width 40
        assert!(!sheet.columns.get(&4).unwrap().hidden);
        assert_eq!(sheet.columns.get(&4).unwrap().width, Some(40.0));
    }

    #[test]
    fn parse_worksheet_handles_row_info() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData>
                    <row r="1" ht="30" hidden="1"><c r="A1"><v>1</v></c></row>
                    <row r="2"><c r="A2"><v>2</v></c></row>
                </sheetData>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        let row0 = sheet.rows.get(&0).unwrap();
        assert_eq!(row0.height, Some(30.0));
        assert!(row0.hidden);
        // row 2 无额外信息，不记录
        assert!(sheet.rows.get(&1).is_none());
    }

    #[test]
    fn parse_worksheet_handles_table_part_rids() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData/>
                <tableParts>
                    <tablePart r:id="rId1"/>
                    <tablePart r:id="rId2"/>
                </tableParts>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let _ = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(table_rids, vec!["rId1", "rId2"]);
    }

    #[test]
    fn parse_worksheet_handles_empty_no_rows() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
                <sheetData/>
            </worksheet>"#;
        let mut table_rids = Vec::new();
        let sheet = parse_worksheet(xml, &[], &[], &mut table_rids).expect("parse");
        assert_eq!(sheet.value(0, 0), CellValue::Empty);
    }

    // ── parse_cell_ref 覆盖 ────────────────────────────────────────────────

    #[test]
    fn parse_cell_ref_with_explicit_reference() {
        use quick_xml::events::BytesStart;
        let e = BytesStart::new("c").with_attributes([("r", "B3")]);
        let result = parse_cell_ref(&e, 0);
        assert_eq!(result, Some((2, 1))); // B3 -> row 2, col 1 (0-based)
    }

    #[test]
    fn parse_cell_ref_fallback_to_row() {
        use quick_xml::events::BytesStart;
        let e = BytesStart::new("c");
        let result = parse_cell_ref(&e, 5);
        assert_eq!(result, Some((5, 0)));
    }

    // ── parse_core_props / parse_app_props 覆盖 ────────────────────────────

    #[test]
    fn parse_core_props_extracts_metadata() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                xmlns:dc="http://purl.org/dc/elements/1.1/"
                xmlns:dcterms="http://purl.org/dc/terms/">
                <dc:title>My Document</dc:title>
                <dc:creator>Author Name</dc:creator>
                <dcterms:created>2024-01-01T00:00:00Z</dcterms:created>
                <dcterms:modified>2024-06-15T12:00:00Z</dcterms:modified>
            </cp:coreProperties>"#;
        let mut meta = Metadata::default();
        parse_core_props(xml, &mut meta);
        assert_eq!(meta.title.as_deref(), Some("My Document"));
        assert_eq!(meta.author.as_deref(), Some("Author Name"));
        assert_eq!(meta.created.as_deref(), Some("2024-01-01T00:00:00Z"));
        assert_eq!(meta.modified.as_deref(), Some("2024-06-15T12:00:00Z"));
    }

    #[test]
    fn parse_core_props_handles_empty_input() {
        let xml = b"<cp:coreProperties/>";
        let mut meta = Metadata::default();
        parse_core_props(xml, &mut meta);
        assert!(meta.title.is_none());
        assert!(meta.author.is_none());
    }

    #[test]
    fn parse_app_props_extracts_metadata() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
                <Application>Microsoft Excel</Application>
                <Company>Test Corp</Company>
            </Properties>"#;
        let mut meta = Metadata::default();
        parse_app_props(xml, &mut meta);
        assert_eq!(meta.application.as_deref(), Some("Microsoft Excel"));
        assert_eq!(meta.company.as_deref(), Some("Test Corp"));
    }

    #[test]
    fn parse_app_props_handles_empty_input() {
        let xml = b"<Properties/>";
        let mut meta = Metadata::default();
        parse_app_props(xml, &mut meta);
        assert!(meta.application.is_none());
        assert!(meta.company.is_none());
    }

    // ── read_zip_with_limits 错误路径 ──────────────────────────────────────

    #[test]
    fn read_zip_rejects_invalid_zip() {
        let garbage = b"this is not a zip file";
        let err = read(Cursor::new(garbage.to_vec())).unwrap_err();
        assert!(matches!(err, Error::Zip(_)));
    }

    #[test]
    fn read_zip_rejects_missing_workbook() {
        // ZIP 有效但缺少 xl/workbook.xml
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
            zip.start_file("dummy.txt", zip::write::SimpleFileOptions::default()).unwrap();
            std::io::Write::write_all(&mut zip, b"hello").unwrap();
            zip.finish().unwrap();
        }
        let err = read(Cursor::new(buf)).unwrap_err();
        assert!(err.to_string().contains("missing xl/workbook.xml"));
    }

    #[test]
    fn read_zip_with_encrypted_package_entry_rejected() {
        // ZIP 内含 EncryptedPackage 条目（非 CFB 包装）时应拒绝
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
            zip.start_file("EncryptedPackage", zip::write::SimpleFileOptions::default()).unwrap();
            std::io::Write::write_all(&mut zip, b"encrypted data").unwrap();
            zip.finish().unwrap();
        }
        let err = read(Cursor::new(buf)).unwrap_err();
        assert!(matches!(err, Error::PasswordProtected(_)));
    }

    // ── multi-sheet 读写 roundtrip ─────────────────────────────────────────

    #[test]
    fn multi_sheet_with_data_roundtrip() {
        let mut wb = Workbook::empty();
        for i in 0_u32..5 {
            let mut sheet = Sheet::new(format!("Sheet{i}"));
            sheet.set(0, 0, Cell::Number(f64::from(i)));
            wb.sheets.push(sheet);
        }
        let out = roundtrip(&wb);
        assert_eq!(out.sheets.len(), 5);
        for i in 0_u32..5 {
            assert_eq!(out.sheets[i as usize].name, format!("Sheet{i}"));
            assert_eq!(out.sheets[i as usize].value(0, 0), CellValue::Number(f64::from(i)));
        }
    }

    // ── Metadata roundtrip ─────────────────────────────────────────────────

    #[test]
    fn metadata_roundtrip() {
        let mut wb = Workbook::empty();
        wb.sheets.push(Sheet::new("Sheet1"));
        wb.metadata.title = Some("Test Title".into());
        wb.metadata.author = Some("Test Author".into());
        wb.metadata.company = Some("Test Company".into());
        wb.metadata.application = Some("EasyExcel-Rust".into());

        let out = roundtrip(&wb);
        assert_eq!(out.metadata.title.as_deref(), Some("Test Title"));
        assert_eq!(out.metadata.author.as_deref(), Some("Test Author"));
        assert_eq!(out.metadata.company.as_deref(), Some("Test Company"));
        assert_eq!(out.metadata.application.as_deref(), Some("EasyExcel-Rust"));
    }
