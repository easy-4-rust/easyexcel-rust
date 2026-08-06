    #[test]
    fn dispatches_blank_bool_rk_hyperlink_note_and_text_object() -> Result<()> {
        // 对应 Java：XlsSaxAnalyser.processRecord 全量 SID 路由（启用态）
        let mut dispatcher = XlsRecordDispatcher::new(&enabled_options());

        // BLANK
        dispatcher.process_record(BLANK_SID, &[1, 0, 2, 0, 3, 0])?;
        assert_eq!(
            dispatcher.state().last_blank_cell(),
            Some(BlankCell { row: 1, column: 2 })
        );
        // BOOL_ERR
        dispatcher.process_record(BOOL_ERR_SID, &[4, 0, 5, 0, 0, 0, 1, 0])?;
        assert_eq!(
            dispatcher.state().last_boolean_cell(),
            Some(BoolCell {
                row: 4,
                column: 5,
                value: true
            })
        );
        // RK
        dispatcher.process_record(RK_SID, &[7, 0, 8, 0, 0, 0])?;
        assert_eq!(
            dispatcher.state().last_rk_cell(),
            Some(BlankCell { row: 7, column: 8 })
        );
        // HYPERLINK（启用）
        dispatcher.process_record(HYPERLINK_SID, &[0, 0, 1, 0, 0, 0, 1, 0])?;
        // NOTE（启用）
        dispatcher.process_record(NOTE_SID, &[2, 0, 3, 0, 0, 0])?;
        // MERGE（启用）
        dispatcher.process_record(MERGE_CELLS_SID, &[1, 0, 0, 0, 1, 0, 0, 0, 1, 0])?;
        // TEXT_OBJECT
        let mut txo = vec![0, 0, 5, 0];
        txo.extend_from_slice(&[0u8; 8]);
        txo.extend_from_slice(b"note");
        dispatcher.process_record(TEXT_OBJECT_SID, &txo)?;
        // OBJ
        dispatcher.process_record(OBJ_SID, &[])?;
        // LABEL
        dispatcher.process_record(LABEL_SID, &[0, 0, 1, 0, 0, 0, 0, 0])?;
        // INDEX
        let mut index = vec![0u8; 16];
        index[8..12].copy_from_slice(&9u32.to_le_bytes());
        dispatcher.process_record(INDEX_SID, &index)?;
        assert_eq!(dispatcher.state().approximate_total_row_number(), Some(9));
        // EOF
        dispatcher.process_record(EOF_SID, &[])?;
        assert_eq!(dispatcher.state().eof_count(), 1);
        // DUMMY
        dispatcher.process_record(DUMMY_RECORD_SID, &[])?;
        assert_eq!(dispatcher.state().handled_record_count(), 12);
        Ok(())
    }

    #[test]
    fn disabled_hyperlink_and_note_are_skipped() -> Result<()> {
        // 对应 Java：support()=false 的处理器跳过并计数
        let mut dispatcher = XlsRecordDispatcher::default();
        dispatcher.process_record(HYPERLINK_SID, &[0, 0, 1, 0, 0, 0, 1, 0])?;
        dispatcher.process_record(NOTE_SID, &[2, 0, 3, 0, 0, 0])?;
        assert_eq!(dispatcher.state().skipped_record_count(), 2);
        assert_eq!(dispatcher.state().handled_record_count(), 0);
        Ok(())
    }

    #[test]
    // 对应 Java：断言为 BIFF `f64` 位级往返值（`to_le_bytes`/`from_le_bytes` 无损），
    // 精确相等即预期语义，不做容差比较。
    #[allow(clippy::float_cmp)]
    fn name_selector_reads_only_the_matching_sheet() -> Result<()> {
        // 对应 Java：SheetUtils.match 按名称匹配工作表
        let options = ReadOptions {
            sheet: SheetSelector::Name("Second".to_owned()),
            ..ReadOptions::default()
        };
        let mut dispatcher = XlsRecordDispatcher::new(&options);

        dispatcher.process_record(BOF_SID, &[0, 0, 0x05, 0x00])?;
        let mut first = vec![0x20, 0, 0, 0, 0, 0, 5, 0];
        first.extend_from_slice(b"First");
        dispatcher.process_record(BOUND_SHEET_SID, &first)?;
        let mut second = vec![0x40, 0, 0, 0, 0, 0, 6, 0];
        second.extend_from_slice(b"Second");
        dispatcher.process_record(BOUND_SHEET_SID, &second)?;
        assert_eq!(dispatcher.state().bound_sheets().len(), 2);

        let mut number = vec![0, 0, 0, 0, 0, 0];
        number.extend_from_slice(&1.0f64.to_le_bytes());
        // 第一个工作表（First）不读
        dispatcher.process_record(BOF_SID, &[0, 0, 0x10, 0x00])?;
        dispatcher.process_record(NUMBER_SID, &number)?;
        assert!(dispatcher.state().last_number_cell().is_none());
        // 第二个工作表（Second）读取
        dispatcher.process_record(BOF_SID, &[0, 0, 0x10, 0x00])?;
        dispatcher.process_record(NUMBER_SID, &number)?;
        assert_eq!(dispatcher.state().last_number_cell().unwrap().value, 1.0);
        Ok(())
    }

    #[test]
    fn bof_unknown_type_and_short_bof_are_tolerated() -> Result<()> {
        // 对应 Java：非工作簿/工作表 BOF 类型忽略；短 BOF 也容忍
        let mut dispatcher = XlsRecordDispatcher::default();
        dispatcher.process_record(BOF_SID, &[0, 0, 0x99, 0x00])?;
        dispatcher.process_record(BOF_SID, &[0, 0])?;
        assert_eq!(dispatcher.state().workbook_bof_count(), 0);
        assert_eq!(dispatcher.state().worksheet_bof_count(), 0);
        Ok(())
    }

    #[test]
    fn continue_without_pending_is_unknown_and_truncated_formula_string_fails() -> Result<()> {
        // 对应 Java：孤儿 CONTINUE 记入 unknown；截断的公式字符串在收尾时报错
        let mut dispatcher = XlsRecordDispatcher::default();
        dispatcher.process_record(CONTINUE_SID, &[0])?;
        assert_eq!(dispatcher.state().unknown_record_count(), 1);

        // STRING_SID 声明 3 个字符但只有 1 字节 → 解码失败，收尾时报错
        dispatcher.process_record(STRING_SID, &[3, 0, 0, b'a'])?;
        let mut number = vec![0, 0, 0, 0, 0, 0];
        number.extend_from_slice(&1.0f64.to_le_bytes());
        assert!(dispatcher.process_record(NUMBER_SID, &number).is_err());
        Ok(())
    }

    #[test]
    fn reset_preserves_feature_flags_and_clears_state() -> Result<()> {
        // 对应 Java：每个工作簿读取前 reset 保持 support 配置
        let mut dispatcher = XlsRecordDispatcher::new(&enabled_options());
        dispatcher.process_record(NOTE_SID, &[2, 0, 3, 0, 0, 0])?;
        assert_eq!(dispatcher.state().handled_record_count(), 1);
        dispatcher.reset();
        assert_eq!(dispatcher.state().handled_record_count(), 0);
        // reset 后 hyperlink/note/merge 仍启用
        dispatcher.process_record(NOTE_SID, &[2, 0, 3, 0, 0, 0])?;
        dispatcher.process_record(HYPERLINK_SID, &[0, 0, 1, 0, 0, 0, 1, 0])?;
        dispatcher.process_record(MERGE_CELLS_SID, &[1, 0, 0, 0, 1, 0, 0, 0, 1, 0])?;
        assert_eq!(dispatcher.state().skipped_record_count(), 0);
        assert_eq!(dispatcher.state().handled_record_count(), 3);
        Ok(())
    }
