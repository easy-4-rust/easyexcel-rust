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
