#[test]
#[allow(clippy::too_many_lines)]
fn xlsx_extra_callbacks_follow_rows_and_java_listener_control_flow() -> Result<()> {
    let (_directory, path) = extra_workbook_fixture()?;
    let all_extras = HashSet::from([
        crate::core::CellExtraType::Comment,
        crate::core::CellExtraType::Hyperlink,
        crate::core::CellExtraType::Merge,
    ]);
    let read_options = ReadOptions {
        sheet: SheetSelector::Name("Meta".to_owned()),
        extra_read: all_extras,
        custom_object: Some(CustomReadObject::new("reader-context".to_owned())),
        ..options()
    };
    let mut probe = ExtraProbe::default();
    read_xlsx::<TestRow, _>(&path, &read_options, &mut probe)?;
    assert_eq!(probe.extras.len(), 4);
    let first_extra = probe
        .events
        .iter()
        .position(|event| *event == "extra")
        .expect("extra event");
    assert!(
        probe.events[..first_extra]
            .iter()
            .all(|event| matches!(*event, "head" | "row"))
    );
    assert!(
        probe.events[first_extra..probe.events.len() - 1]
            .iter()
            .all(|event| *event == "extra")
    );
    assert_eq!(probe.events.last(), Some(&"after"));
    assert!(
        probe
            .context_customs
            .iter()
            .all(|value| value.as_deref() == Some("reader-context"))
    );

    let merge = probe
        .extras
        .iter()
        .find(|extra| extra.extra_type() == crate::core::CellExtraType::Merge)
        .expect("merge extra");
    assert_eq!(merge.first_row_index(), 3);
    assert_eq!(merge.last_row_index(), 3);
    assert_eq!(merge.first_column_index(), 0);
    assert_eq!(merge.last_column_index(), 1);
    let hyperlinks = probe
        .extras
        .iter()
        .filter(|extra| extra.extra_type() == crate::core::CellExtraType::Hyperlink)
        .filter_map(crate::core::CellExtra::text)
        .collect::<Vec<_>>();
    assert!(hyperlinks.contains(&"https://example.com"));
    assert!(hyperlinks.contains(&"Meta!A1"));
    let comment = probe
        .extras
        .iter()
        .find(|extra| extra.extra_type() == crate::core::CellExtraType::Comment)
        .expect("comment extra");
    assert_eq!(comment.text(), Some("Author:\ncomment & text"));
    assert_eq!(comment.first_row_index(), 1);
    assert_eq!(comment.first_column_index(), 0);

    let mut comments_only = ExtraProbe::default();
    read_xlsx::<TestRow, _>(
        &path,
        &ReadOptions {
            extra_read: HashSet::from([crate::core::CellExtraType::Comment]),
            ..options()
        },
        &mut comments_only,
    )?;
    assert_eq!(comments_only.extras.len(), 1);
    assert_eq!(
        comments_only.extras[0].extra_type(),
        crate::core::CellExtraType::Comment
    );

    let mut stopped = ExtraProbe {
        stop_after_extra: true,
        ..ExtraProbe::default()
    };
    read_xlsx::<TestRow, _>(&path, &read_options, &mut stopped)?;
    assert_eq!(stopped.extras.len(), 1);
    assert!(!stopped.events.contains(&"after"));

    let mut continued_error = ExtraProbe {
        fail_extra: true,
        error_action: Some(ErrorAction::Continue),
        ..ExtraProbe::default()
    };
    read_xlsx::<TestRow, _>(&path, &read_options, &mut continued_error)?;
    assert_eq!(continued_error.errors, 4);
    assert_eq!(continued_error.events.last(), Some(&"after"));
    assert!(
        continued_error
            .context_customs
            .iter()
            .all(|value| value.as_deref() == Some("reader-context"))
    );

    let mut stopped_error = ExtraProbe {
        fail_extra: true,
        ..ExtraProbe::default()
    };
    assert!(read_xlsx::<TestRow, _>(&path, &read_options, &mut stopped_error).is_err());
    assert_eq!(stopped_error.errors, 1);
    assert!(!stopped_error.events.contains(&"after"));

    let malformed = path.with_file_name("malformed-extra.xlsx");
    rewrite_first_sheet(&path, &malformed, "<worksheet>")?;
    let mut malformed_probe = ExtraProbe::default();
    assert!(read_xlsx::<TestRow, _>(&malformed, &read_options, &mut malformed_probe).is_err());
    Ok(())
}

#[test]
fn csv_rejects_extra_metadata_while_xls_reaches_the_input() {
    let options = ReadOptions {
        extra_read: HashSet::from([crate::core::CellExtraType::Comment]),
        ..options()
    };
    let mut probe = Probe::default();
    assert!(matches!(
        read_xls::<TestRow, _>(Path::new("missing.xls"), &options, &mut probe),
        Err(ExcelError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound
    ));
    assert!(matches!(
        read_csv::<TestRow, _>(Path::new("missing.csv"), &options, &mut probe),
        Err(ExcelError::Unsupported(message)) if message.contains("CSV")
    ));
}
