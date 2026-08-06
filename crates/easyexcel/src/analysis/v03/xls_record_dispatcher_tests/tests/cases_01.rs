    #[test]
    // 对应 Java：断言为 BIFF `f64` 位级往返值（`to_le_bytes`/`from_le_bytes` 无损），
    // 精确相等即预期语义，不做容差比较。
    #[allow(clippy::float_cmp)]
    fn dispatches_number_to_real_handler_and_keeps_unknown_records_ignorable() -> Result<()> {
        let mut dispatcher = XlsRecordDispatcher::default();
        let mut number = vec![2, 0, 3, 0, 7, 0];
        number.extend_from_slice(&42.5f64.to_le_bytes());

        dispatcher.process_record(NUMBER_SID, &number)?;
        dispatcher.process_record(0x1234, &[])?;

        assert_eq!(dispatcher.state().total_record_count(), 2);
        assert_eq!(dispatcher.state().handled_record_count(), 1);
        assert_eq!(dispatcher.state().unknown_record_count(), 1);
        let cell = dispatcher
            .state()
            .last_number_cell()
            .expect("number handler output");
        assert_eq!((cell.row, cell.column, cell.format_index), (2, 3, 7));
        assert_eq!(cell.value, 42.5);
        Ok(())
    }

    #[test]
    fn support_predicate_skips_merge_when_not_requested() -> Result<()> {
        let mut dispatcher = XlsRecordDispatcher::default();
        dispatcher.process_record(MERGE_CELLS_SID, &[0; 10])?;
        assert_eq!(dispatcher.state().handled_record_count(), 0);
        assert_eq!(dispatcher.state().skipped_record_count(), 1);
        Ok(())
    }

    #[test]
    // 对应 Java：断言为 BIFF `f64` 位级往返值（`to_le_bytes`/`from_le_bytes` 无损），
    // 精确相等即预期语义，不做容差比较。
    #[allow(clippy::float_cmp)]
    fn selected_sheet_skips_ignorable_records_until_next_bof() -> Result<()> {
        let mut dispatcher = XlsRecordDispatcher::default();
        let workbook_bof = [0, 0, 0x05, 0x00];
        let worksheet_bof = [0, 0, 0x10, 0x00];
        let mut first_number = vec![0, 0, 0, 0, 0, 0];
        first_number.extend_from_slice(&1.0f64.to_le_bytes());
        let mut second_number = vec![0, 0, 0, 0, 0, 0];
        second_number.extend_from_slice(&2.0f64.to_le_bytes());

        dispatcher.process_record(BOF_SID, &workbook_bof)?;
        dispatcher.process_record(BOF_SID, &worksheet_bof)?;
        dispatcher.process_record(NUMBER_SID, &first_number)?;
        dispatcher.process_record(EOF_SID, &[])?;
        dispatcher.process_record(BOF_SID, &worksheet_bof)?;
        dispatcher.process_record(NUMBER_SID, &second_number)?;

        assert_eq!(
            dispatcher
                .state()
                .last_number_cell()
                .expect("first sheet number")
                .value,
            1.0
        );
        assert_eq!(dispatcher.state().skipped_record_count(), 1);
        Ok(())
    }

    #[test]
    fn sst_continue_resolves_following_label_sst() -> Result<()> {
        let mut dispatcher = XlsRecordDispatcher::default();
        let mut sst = Vec::new();
        sst.extend_from_slice(&1u32.to_le_bytes());
        sst.extend_from_slice(&1u32.to_le_bytes());
        sst.extend_from_slice(&4u16.to_le_bytes());
        sst.push(0);
        sst.extend_from_slice(b"  ");
        dispatcher.process_record(SST_SID, &sst)?;
        dispatcher.process_record(CONTINUE_SID, &[0, b'o', b'k'])?;

        let mut label = vec![3, 0, 2, 0, 0, 0];
        label.extend_from_slice(&0u32.to_le_bytes());
        dispatcher.process_record(LABEL_SST_SID, &label)?;

        assert_eq!(dispatcher.state().shared_strings(), &["  ok".to_owned()]);
        assert_eq!(
            dispatcher.state().last_label_sst_cell(),
            Some(&LabelSstCell::String {
                row: 3,
                column: 2,
                value: "ok".to_owned(),
            })
        );
        Ok(())
    }

    #[test]
    fn formula_string_record_completes_pending_cached_result_across_continue() -> Result<()> {
        let mut dispatcher = XlsRecordDispatcher::default();
        let formula = vec![5, 0, 4, 0, 0, 0, 0x00, 0, 0, 0, 0, 0, 0xFF, 0xFF];
        dispatcher.process_record(FORMULA_SID, &formula)?;
        assert!(
            dispatcher
                .state()
                .last_formula_cell()
                .is_none_or(|cell| cell.cached_type != FormulaCachedType::String)
        );

        let string = vec![4, 0, 0, b'a', b'b'];
        dispatcher.process_record(STRING_SID, &string)?;
        dispatcher.process_record(CONTINUE_SID, &[0, b'c', b'd'])?;

        let cell = dispatcher
            .state()
            .last_formula_cell()
            .expect("completed string formula");
        assert_eq!((cell.row, cell.column), (5, 4));
        assert_eq!(cell.string_value.as_deref(), Some("abcd"));
        assert!(!cell.pending_string);
        Ok(())
    }

    #[test]
    fn finish_records_rejects_truncated_continuable_record() -> Result<()> {
        let mut dispatcher = XlsRecordDispatcher::default();
        let mut sst = Vec::new();
        sst.extend_from_slice(&1u32.to_le_bytes());
        sst.extend_from_slice(&1u32.to_le_bytes());
        sst.extend_from_slice(&2u16.to_le_bytes());
        sst.push(0);
        sst.push(b'a');
        dispatcher.process_record(SST_SID, &sst)?;
        assert!(dispatcher.finish_records().is_err());
        Ok(())
    }
