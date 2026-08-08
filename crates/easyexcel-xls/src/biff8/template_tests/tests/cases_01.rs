    #[test]
    fn detects_ole_magic_and_rejects_non_ole_template() {
        assert!(looks_like_xls(&[
            0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1,
        ]));
        assert!(!looks_like_xls(b"PK\x03\x04"));
        assert!(matches!(
            Biff8TemplatePackage::from_bytes(b"not an xls"),
            Err(ExcelError::Xls(_))
        ));
    }

    #[test]
    fn placeholder_keys_cover_scalar_named_and_unnamed_collection_forms() {
        assert_eq!(scalar_placeholder_key("{name}"), "name");
        assert_eq!(scalar_placeholder_key("{{name}}"), "name");
        assert_eq!(collection_placeholder_key("{.name}", None), Some("name"));
        assert_eq!(
            collection_placeholder_key("{users.name}", Some("users")),
            Some("name")
        );
        assert_eq!(
            collection_placeholder_key("{fallback}", Some("users")),
            Some("fallback")
        );
        assert_eq!(collection_placeholder_key("plain", None), None);
    }

    #[test]
    fn chart_sheet_consumes_its_boundsheet_and_preserves_top_level_offsets() -> Result<()> {
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
            boundsheet("Chart1", 2),
            boundsheet("Data", 0),
            RawRecord {
                typ: EOF,
                data: Vec::new(),
            },
            bof(0x0020),
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

        let sheets = discover_sheets(&records)?;
        assert_eq!(sheets.len(), 1);
        assert_eq!(sheets[0].name, "Data");

        let workbook = assemble_workbook(&records)?;
        let reparsed = split_records(&workbook)?;
        let offsets = reparsed
            .iter()
            .filter(|record| record.typ == BOUNDSHEET)
            .map(|record| {
                u32::from_le_bytes([
                    record.data[0],
                    record.data[1],
                    record.data[2],
                    record.data[3],
                ]) as usize
            })
            .collect::<Vec<_>>();
        assert_eq!(offsets.len(), 2);
        assert_eq!(
            u16::from_le_bytes([workbook[offsets[0]], workbook[offsets[0] + 1]]),
            BOF
        );
        assert_eq!(
            u16::from_le_bytes([workbook[offsets[0] + 6], workbook[offsets[0] + 7]]),
            0x0020
        );
        assert_eq!(
            u16::from_le_bytes([workbook[offsets[1] + 6], workbook[offsets[1] + 7]]),
            DT_WORKSHEET
        );
        Ok(())
    }

    fn workbook_with_vba(project: &[u8]) -> Result<Vec<u8>> {
        use std::io::Write as _;
        let mut book = crate::biff8::Biff8Book::default();
        book.sheets.push(crate::biff8::Biff8Sheet::new("Data"));
        let mut cursor = Cursor::new(book.to_cfb_bytes()?);
        {
            let mut compound = CompoundFile::open(&mut cursor)
                .map_err(|error| ExcelError::Cfb(error.to_string()))?;
            compound
                .create_storage_all("/_VBA_PROJECT_CUR/VBA")
                .map_err(|error| ExcelError::Cfb(error.to_string()))?;
            for (path, bytes) in [
                ("/_VBA_PROJECT_CUR/PROJECT", project),
                ("/_VBA_PROJECT_CUR/VBA/dir", b"compressed-dir".as_slice()),
                ("/_VBA_PROJECT_CUR/VBA/Module1", b"module-code".as_slice()),
            ] {
                let mut stream = compound
                    .create_stream(path)
                    .map_err(|error| ExcelError::Cfb(error.to_string()))?;
                stream.write_all(bytes)?;
            }
            compound
                .flush()
                .map_err(|error| ExcelError::Cfb(error.to_string()))?;
        }
        Ok(cursor.into_inner())
    }

    #[test]
    fn template_rewrite_preserves_vba_storage_streams_byte_for_byte() -> Result<()> {
        use std::io::Read as _;

        let mut package = Biff8TemplatePackage::from_bytes(&workbook_with_vba(b"project")?)?;
        package.set_cell(
            "Data",
            0,
            0,
            &Biff8Cell::general(Biff8Value::Text("updated".to_owned())),
        )?;
        let output = package.to_bytes()?;
        let mut compound = CompoundFile::open(Cursor::new(output))
            .map_err(|error| ExcelError::Cfb(error.to_string()))?;
        for (path, expected) in [
            ("/_VBA_PROJECT_CUR/PROJECT", b"project".as_slice()),
            ("/_VBA_PROJECT_CUR/VBA/dir", b"compressed-dir".as_slice()),
            ("/_VBA_PROJECT_CUR/VBA/Module1", b"module-code".as_slice()),
        ] {
            let mut actual = Vec::new();
            compound
                .open_stream(path)
                .map_err(|error| ExcelError::Cfb(error.to_string()))?
                .read_to_end(&mut actual)?;
            assert_eq!(actual, expected, "VBA stream changed: {path}");
        }
        Ok(())
    }

    #[test]
    fn macro_policy_can_strip_or_replace_the_complete_vba_storage() -> Result<()> {
        use std::io::Read as _;

        let template = workbook_with_vba(b"old-project")?;
        let package = Biff8TemplatePackage::from_bytes(&template)?;
        let stripped = package.to_bytes_with_password_and_macro_policy(
            None,
            &Biff8MacroPolicy::Strip,
        )?;
        let stripped_cfb = CompoundFile::open(Cursor::new(stripped))
            .map_err(|error| ExcelError::Cfb(error.to_string()))?;
        assert!(!stripped_cfb.exists("/_VBA_PROJECT_CUR"));

        let replacement = workbook_with_vba(b"new-project")?;
        let replaced = package.to_bytes_with_password_and_macro_policy(
            None,
            &Biff8MacroPolicy::Replace(replacement),
        )?;
        let mut replaced_cfb = CompoundFile::open(Cursor::new(replaced))
            .map_err(|error| ExcelError::Cfb(error.to_string()))?;
        let mut project = Vec::new();
        replaced_cfb
            .open_stream("/_VBA_PROJECT_CUR/PROJECT")
            .map_err(|error| ExcelError::Cfb(error.to_string()))?
            .read_to_end(&mut project)?;
        assert_eq!(project, b"new-project");
        Ok(())
    }

    #[test]
    fn appended_cells_stay_before_embedded_chart_substreams() -> Result<()> {
        fn bof(stream_type: u16) -> RawRecord {
            RawRecord {
                typ: BOF,
                data: [0x00, 0x06, stream_type as u8, (stream_type >> 8) as u8].to_vec(),
            }
        }
        let mut boundsheet = vec![0, 0, 0, 0, 0, 0, 4, 0];
        boundsheet.extend_from_slice(b"Data");
        let mut existing_cell = vec![0, 0, 0, 0, 0, 0];
        existing_cell.extend_from_slice(&encode_unicode_string("existing"));
        let records = vec![
            bof(0x0005),
            RawRecord {
                typ: BOUNDSHEET,
                data: boundsheet,
            },
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
                typ: LABEL,
                data: existing_cell,
            },
            RawRecord {
                typ: 0x00EC,
                data: vec![1, 2, 3],
            },
            RawRecord {
                typ: 0x005D,
                data: vec![4, 5, 6],
            },
            bof(0x0020),
            RawRecord {
                typ: 0x1002,
                data: vec![7, 8],
            },
            RawRecord {
                typ: EOF,
                data: Vec::new(),
            },
            RawRecord {
                typ: 0x023E,
                data: vec![0; 18],
            },
            RawRecord {
                typ: EOF,
                data: Vec::new(),
            },
        ];
        let sheets = discover_sheets(&records)?;
        assert_eq!(sheets.len(), 1);
        assert_eq!(sheet_cell_insert_index(&records, &sheets[0]), 6);
        Ok(())
    }

    #[test]
    fn force_new_row_shifts_comment_image_and_chart_client_anchors() -> Result<()> {
        fn client_anchor(first_row: u16, last_row: u16) -> Vec<u8> {
            let mut data = Vec::new();
            data.extend_from_slice(&0u16.to_le_bytes());
            data.extend_from_slice(&0xF010u16.to_le_bytes());
            data.extend_from_slice(&18u32.to_le_bytes());
            data.extend_from_slice(&0u16.to_le_bytes());
            for value in [0u16, 0, first_row, 0, 3, 0, last_row, 0] {
                data.extend_from_slice(&value.to_le_bytes());
            }
            data
        }

        let mut drawing = vec![0xAA, 0xBB, 0xCC];
        drawing.extend_from_slice(&client_anchor(4, 8));
        drawing.extend_from_slice(&[0x11, 0x22]);
        drawing.extend_from_slice(&client_anchor(1, 2));
        shift_msodrawing_anchors(&mut drawing, 3, 5)?;

        let anchors = [11usize, 39];
        assert_eq!(
            u16::from_le_bytes(drawing[anchors[0] + 6..anchors[0] + 8].try_into().unwrap()),
            9
        );
        assert_eq!(
            u16::from_le_bytes(drawing[anchors[0] + 14..anchors[0] + 16].try_into().unwrap()),
            13
        );
        assert_eq!(
            u16::from_le_bytes(drawing[anchors[1] + 6..anchors[1] + 8].try_into().unwrap()),
            1
        );
        assert_eq!(
            u16::from_le_bytes(drawing[anchors[1] + 14..anchors[1] + 16].try_into().unwrap()),
            2
        );
        Ok(())
    }

    #[test]
    fn force_new_row_rewrites_absolute_and_relative_formula_tokens() -> Result<()> {
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

        let mut absolute = formula(1, &[0x24, 4, 0, 0, 0]);
        shift_formula_references(&mut absolute, 3, 2, 0, &[])?;
        assert_eq!(&absolute.data[23..25], &[6, 0]);

        let mut crossing_relative = formula(1, &[0x24, 3, 0, 0, 0x80]);
        shift_formula_references(&mut crossing_relative, 3, 2, 0, &[])?;
        assert_eq!(&crossing_relative.data[23..25], &[5, 0]);

        let mut moving_relative = formula(4, &[0x24, 2, 0, 0, 0x80]);
        shift_formula_references(&mut moving_relative, 3, 2, 0, &[])?;
        assert_eq!(&moving_relative.data[23..25], &[2, 0]);

        let mut area = formula(0, &[0x25, 2, 0, 4, 0, 0, 0, 0, 0]);
        shift_formula_references(&mut area, 3, 2, 0, &[])?;
        assert_eq!(&area.data[23..25], &[2, 0]);
        assert_eq!(&area.data[25..27], &[6, 0]);

        let mut reference_3d = formula(0, &[0x3A, 0, 0, 4, 0, 0, 0]);
        shift_formula_references(&mut reference_3d, 3, 2, 0, &[Some((0, 0))])?;
        assert_eq!(&reference_3d.data[25..27], &[6, 0]);
        let mut other_sheet_3d = formula(0, &[0x3A, 0, 0, 4, 0, 0, 0]);
        shift_formula_references(&mut other_sheet_3d, 3, 2, 1, &[Some((0, 0))])?;
        assert_eq!(&other_sheet_3d.data[25..27], &[4, 0]);

        let records = vec![
            RawRecord {
                typ: SUP_BOOK_SID,
                data: vec![2, 0, 1, 4],
            },
            RawRecord {
                typ: EXTERNAL_SHEET_SID,
                data: vec![1, 0, 0, 0, 1, 0, 1, 0],
            },
        ];
        assert_eq!(internal_extern_sheet_ranges(&records), vec![Some((1, 1))]);

        let chart_tokens = [0x3B, 0, 0, 0, 0, 31, 0, 0, 0, 1, 0];
        let mut chart_ai = RawRecord {
            typ: CHART_AI_SID,
            data: vec![1, 2, 0, 0, 0, 0, 11, 0],
        };
        chart_ai.data.extend_from_slice(&chart_tokens);
        shift_chart_ai_references(&mut chart_ai, 10, 2, 0, &[Some((0, 0))])?;
        assert_eq!(&chart_ai.data[11..13], &[0, 0]);
        assert_eq!(&chart_ai.data[13..15], &[33, 0]);
        Ok(())
    }

    #[test]
    fn force_new_row_shifts_name_conditional_format_and_data_validation_references() -> Result<()> {
        let mut name = RawRecord {
            typ: NAME_SID,
            data: vec![0, 0, 0, 1, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, b'N'],
        };
        name.data
            .extend_from_slice(&[0x3B, 0, 0, 4, 0, 6, 0, 0, 0, 0, 0]);
        shift_name_references(&mut name, 3, 2, 0, &[Some((0, 0))])?;
        assert_eq!(&name.data[19..21], &[6, 0]);
        assert_eq!(&name.data[21..23], &[8, 0]);

        let mut condfmt = Vec::new();
        condfmt.extend_from_slice(&1u16.to_le_bytes());
        condfmt.extend_from_slice(&0u16.to_le_bytes());
        for value in [4u16, 6, 0, 0] {
            condfmt.extend_from_slice(&value.to_le_bytes());
        }
        condfmt.extend_from_slice(&1u16.to_le_bytes());
        for value in [4u16, 6, 0, 0] {
            condfmt.extend_from_slice(&value.to_le_bytes());
        }
        let base = shift_conditional_format_header(&mut condfmt, 5, 2)?;
        assert_eq!(base, (4, 4));
        assert_eq!(&condfmt[6..8], &[8, 0]);
        assert_eq!(&condfmt[16..18], &[8, 0]);

        let mut cf = vec![2, 0, 5, 0, 0, 0];
        cf.extend_from_slice(&0u32.to_le_bytes());
        cf.extend_from_slice(&0u16.to_le_bytes());
        cf.extend_from_slice(&[0x24, 6, 0, 0, 0]);
        shift_conditional_format_rule(&mut cf, base.0, base.1, 5, 2, 0, &[])?;
        assert_eq!(&cf[13..15], &[8, 0]);

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
        shift_data_validation(&mut dv, 5, 2, 0, &[])?;
        assert_eq!(&dv[25..27], &[8, 0]);
        assert_eq!(&dv[35..37], &[7, 0]);
        assert_eq!(&dv[37..39], &[9, 0]);
        Ok(())
    }

    #[test]
    fn force_new_row_migrates_real_poi_dv_cf_and_name_records() -> Result<()> {
        use std::collections::BTreeMap;
        use std::io::Read as _;

        use base64::Engine as _;
        use flate2::read::GzDecoder;

        let compressed = base64::engine::general_purpose::STANDARD
            .decode(
                include_str!("../fixtures/poi-dv-cf-name.xls.gz.b64")
                    .trim(),
            )
            .map_err(|error| ExcelError::Xls(error.to_string()))?;
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut template = Vec::new();
        decoder.read_to_end(&mut template)?;

        let mut package = Biff8TemplatePackage::from_bytes(&template)?;
        let rows = ["one", "two", "three"]
            .into_iter()
            .map(|value| BTreeMap::from([("value".to_owned(), value.to_owned())]))
            .collect::<Vec<_>>();
        assert_eq!(
            package.fill_collection_placeholders(
                Some("Data"),
                None,
                &rows,
                false,
                true,
                true,
            )?,
            3
        );
        let output = package.to_bytes()?;
        if let Ok(path) = std::env::var("EASYEXCEL_POI_SHIFT_OUTPUT") {
            std::fs::write(path, &output)?;
        }
        let mut compound = CompoundFile::open(Cursor::new(output))
            .map_err(|error| ExcelError::Cfb(error.to_string()))?;
        let mut workbook = Vec::new();
        compound
            .open_stream("/Workbook")
            .map_err(|error| ExcelError::Cfb(error.to_string()))?
            .read_to_end(&mut workbook)?;
        let records = split_records(&workbook)?;

        let name = records
            .iter()
            .find(|record| record.typ == NAME_SID)
            .expect("POI NAME record");
        let name_token_start = 15 + usize::from(name.data[3]);
        assert_eq!(&name.data[name_token_start + 3..name_token_start + 5], &[6, 0]);
        assert_eq!(&name.data[name_token_start + 5..name_token_start + 7], &[7, 0]);

        let condfmt = records
            .iter()
            .find(|record| record.typ == CONDITIONAL_FORMATTING_HEADER_SID)
            .expect("POI CONDFMT record");
        assert_eq!(&condfmt.data[4..8], &[6, 0, 7, 0]);
        assert_eq!(&condfmt.data[14..18], &[6, 0, 7, 0]);

        let cf = records
            .iter()
            .find(|record| record.typ == CONDITIONAL_FORMATTING_RULE_SID)
            .expect("POI CF record");
        let formatting_options =
            u32::from_le_bytes(cf.data[6..10].try_into().expect("CF options"));
        let formula_start = 12
            + if formatting_options & 0x0400_0000 != 0 { 118 } else { 0 }
            + if formatting_options & 0x1000_0000 != 0 { 8 } else { 0 }
            + if formatting_options & 0x2000_0000 != 0 { 4 } else { 0 };
        assert_eq!(&cf.data[formula_start + 1..formula_start + 3], &[6, 0]);

        let dv = records
            .iter()
            .find(|record| record.typ == DATA_VALIDATION_SID)
            .expect("POI DV record");
        let mut sqref_offset = 4;
        for _ in 0..4 {
            sqref_offset = unicode_string_end(&dv.data, sqref_offset)?;
        }
        let formula1_length = usize::from(read_u16_at(
            &dv.data,
            sqref_offset,
            "DV formula1 length",
        )?);
        sqref_offset = sqref_offset
            .saturating_add(4)
            .saturating_add(formula1_length);
        let formula2_length = usize::from(read_u16_at(
            &dv.data,
            sqref_offset,
            "DV formula2 length",
        )?);
        sqref_offset = sqref_offset
            .saturating_add(4)
            .saturating_add(formula2_length);
        assert_eq!(&dv.data[sqref_offset + 2..sqref_offset + 6], &[6, 0, 7, 0]);
        Ok(())
    }
