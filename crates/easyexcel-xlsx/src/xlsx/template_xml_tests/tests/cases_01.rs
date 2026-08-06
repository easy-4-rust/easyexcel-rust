    #[test]
    fn column_names_cover_single_and_multiple_letter_ranges() {
        assert_eq!(column_name(1), "A");
        assert_eq!(column_name(26), "Z");
        assert_eq!(column_name(27), "AA");
        assert_eq!(column_name(52), "AZ");
        assert_eq!(column_name(53), "BA");
        assert_eq!(column_name(703), "AAA");
    }

    #[test]
    fn xml_escape_covers_all_special_characters() {
        assert_eq!(escape_xml(""), "");
        assert_eq!(escape_xml("hello"), "hello");
        assert_eq!(escape_xml("<tag>&\"'"), "&lt;tag&gt;&amp;&quot;&apos;");
    }

    #[test]
    fn cell_reference_parser_covers_valid_invalid_and_bounds() {
        assert_eq!(parse_cell_reference("A1"), Some((1, 1)));
        assert_eq!(parse_cell_reference("AB10"), Some((28, 10)));
        assert_eq!(
            parse_cell_reference("$XFD$1048576"),
            Some((16_384, 1_048_576))
        );
        assert_eq!(parse_cell_reference(""), None);
        assert_eq!(parse_cell_reference("1A"), None);
        assert_eq!(parse_cell_reference("A!1"), None);
        assert_eq!(parse_cell_reference("XFE1"), None);
    }

    #[test]
    fn worksheet_helpers_read_attributes_rows_styles_and_dimensions() {
        assert_eq!(
            attribute_value(r#"<tag attr="value">"#, "attr"),
            Some("value")
        );
        assert_eq!(attribute_value(r#"<tag attr="value">"#, "missing"), None);
        assert_eq!(row_index("row"), None);
        assert_eq!(row_index("row r=\"15\""), Some(15));
        assert_eq!(worksheet_max_row(r#"<row r="5"/><row r="10"/>"#), 10);
        assert_eq!(worksheet_max_row("<row"), 0);
        assert_eq!(cell_style_index(r#"<c r="A1" s="5"/>"#, "A1"), Some(5));
        assert_eq!(cell_style_index(r#"<c r="A1"/>"#, "A1"), None);

        let xml = r#"<worksheet><dimension ref="A1"/><sheetData><row r="1"><c r="A1"/></row><row r="5"><c r="C5"/></row></sheetData></worksheet>"#;
        assert!(update_worksheet_dimension(xml).contains("ref=\"A1:C5\""));
        assert_eq!(
            update_worksheet_dimension("<c r=\"A1\"><v>1</v></c>"),
            "<c r=\"A1\"><v>1</v></c>"
        );
    }
