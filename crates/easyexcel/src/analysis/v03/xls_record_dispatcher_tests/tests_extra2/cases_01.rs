    #[test]
    fn label_sst_without_reference_keeps_state_unchanged() -> Result<()> {
        // 对应 Java：LabelSST 记录体不足 10 字节时不更新 lastReference
        let mut dispatcher = XlsRecordDispatcher::default();
        dispatcher.process_record(LABEL_SST_SID, &[0, 0, 2, 0])?;
        assert!(dispatcher.state().last_label_sst_cell().is_none());
        // 上一记录解析失败后，后续合法 LabelSST 正常解析
        let mut label = vec![3, 0, 2, 0, 0, 0];
        label.extend_from_slice(&0u32.to_le_bytes());
        dispatcher.process_record(LABEL_SST_SID, &label)?;
        assert!(dispatcher.state().last_label_sst_cell().is_some());
        Ok(())
    }

    #[test]
    fn all_sheets_selector_reads_every_worksheet() -> Result<()> {
        // 对应 Java：SheetUtils.match 全表选择时不跳过任何工作表
        let options = crate::ReadOptions {
            sheet: SheetSelector::All,
            ..crate::ReadOptions::default()
        };
        let mut dispatcher = XlsRecordDispatcher::new(&options);

        dispatcher.process_record(BOF_SID, &[0, 0, 0x05, 0x00])?;
        let mut number = vec![0, 0, 0, 0, 0, 0];
        number.extend_from_slice(&1.0f64.to_le_bytes());
        // 第一个工作表
        dispatcher.process_record(BOF_SID, &[0, 0, 0x10, 0x00])?;
        dispatcher.process_record(NUMBER_SID, &number)?;
        // 第二个工作表
        dispatcher.process_record(BOF_SID, &[0, 0, 0x10, 0x00])?;
        dispatcher.process_record(NUMBER_SID, &number)?;

        assert_eq!(dispatcher.state().skipped_record_count(), 0);
        assert_eq!(dispatcher.state().worksheet_bof_count(), 2);
        assert_eq!(dispatcher.state().handled_record_count(), 5);
        Ok(())
    }

    #[test]
    fn continue_record_extends_pending_sst_segments() -> Result<()> {
        // 对应 Java：SST 主记录 + CONTINUE 继续片段分两次解码
        let mut dispatcher = XlsRecordDispatcher::default();
        let mut sst = Vec::new();
        sst.extend_from_slice(&1u32.to_le_bytes());
        sst.extend_from_slice(&1u32.to_le_bytes());
        sst.extend_from_slice(&2u16.to_le_bytes());
        sst.push(0);
        sst.push(b'a');
        // CONTINUE 补齐剩余字符后完成解码
        dispatcher.process_record(SST_SID, &sst)?;
        dispatcher.process_record(CONTINUE_SID, &[0, b'b'])?;
        assert_eq!(dispatcher.state().shared_strings(), &["ab".to_owned()]);
        Ok(())
    }
