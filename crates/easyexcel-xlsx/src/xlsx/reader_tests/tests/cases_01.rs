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
