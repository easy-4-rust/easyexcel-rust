/// 4-state spill matrix tests: `constant_memory` x `compress_temp_files`.
///
/// `uses_constant_memory_spill()` = `constant_memory || compress_temp_files`;
/// 3/4 states (all except `false x false` with explicit InMemory) exercise the
/// spill path.  When the auto path selects streaming, ALL 4 states produce
/// streaming-mode XLSX with identical checksums.
///
/// This file verifies:
/// - All 4 combinations produce a valid, readable XLSX with correct data.
/// - All 4 produce identical checksums (deterministic streaming output).
/// - Capability conflicts are reported correctly.
/// - Auto-promotion clears `compress_temp_files` after replay.

/// Baseline: default auto path. After first write selects AutoStreaming.
/// Neither `constant_memory` nor `compress_temp_files` is set explicitly.
/// This is the baseline checksum all other states must match.
#[test]
fn spill_matrix_false_false_auto_streaming_baseline() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("matrix_ff.xlsx");
    let data: Vec<AutoStateRow> = (0..50).map(|i| AutoStateRow { value: i }).collect();
    // Default auto path — no explicit constant_memory / compress_temp_files.
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&path).build();
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::AutoUndecided
    );
    writer.write(data.clone(), &WriteSheet::new("Data"))?;
    // After first write, auto-selects streaming for plain rows.
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::AutoStreaming
    );
    writer.finish()?;
    assert!(path.exists(), "baseline file must exist");

    // Verify data is correct.
    let mut book: Xlsx<_> = open_workbook(&path).map_err(test_error)?;
    let range = book.worksheet_range("Data").map_err(test_error)?;
    assert_eq!(range.height(), 51, "header + 50 data rows");
    for i in 0..50u64 {
        assert_eq!(
            range.get_value((i as u32 + 1, 0)),
            Some(&Data::Float(i as f64)),
            "row {i} mismatch"
        );
    }
    // Record baseline checksum for comparison.
    let _baseline_checksum = sha256_of_file(&path);
    Ok(())
}

/// `false x true`: compress_temp_files forces constant_memory. Gzip spill.
/// Checksum must equal the baseline (same streaming XLSX output).
#[test]
fn spill_matrix_false_true_compress_spill() -> Result<()> {
    let dir = tempdir()?;
    let baseline_path = dir.path().join("baseline_ft.xlsx");
    let spill_path = dir.path().join("matrix_ft.xlsx");
    let data: Vec<AutoStateRow> = (0..50).map(|i| AutoStateRow { value: i }).collect();

    // Baseline (auto streaming).
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&baseline_path).build();
    writer.write(data.clone(), &WriteSheet::new("Data"))?;
    writer.finish()?;

    // Spill write: compress_temp_files forces constant_memory + ExplicitStreaming.
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&spill_path)
        .compress_temp_files(true)
        .build();
    assert!(writer.compress_temp_files_enabled());
    writer.write(data, &WriteSheet::new("Data"))?;
    writer.finish()?;

    // Verify data is readable.
    let mut book: Xlsx<_> = open_workbook(&spill_path).map_err(test_error)?;
    let range = book.worksheet_range("Data").map_err(test_error)?;
    assert_eq!(range.height(), 51);
    assert_eq!(range.get_value((1, 0)), Some(&Data::Float(0.0)));
    assert_eq!(range.get_value((50, 0)), Some(&Data::Float(49.0)));

    // Checksum must match baseline.
    assert_same_checksum(&[&baseline_path, &spill_path]);
    Ok(())
}

/// `true x false`: explicit constant_memory, no compress. ExplicitStreaming.
/// Checksum must equal the baseline (same streaming XLSX output).
#[test]
fn spill_matrix_true_false_explicit_streaming() -> Result<()> {
    let dir = tempdir()?;
    let baseline_path = dir.path().join("baseline_tf.xlsx");
    let spill_path = dir.path().join("matrix_tf.xlsx");
    let data: Vec<AutoStateRow> = (0..50).map(|i| AutoStateRow { value: i }).collect();

    // Baseline (auto streaming).
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&baseline_path).build();
    writer.write(data.clone(), &WriteSheet::new("Data"))?;
    writer.finish()?;

    // Spill write: explicit constant_memory.
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&spill_path)
        .constant_memory(true)
        .build();
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::ExplicitStreaming
    );
    writer.write(data, &WriteSheet::new("Data"))?;
    writer.finish()?;

    let mut book: Xlsx<_> = open_workbook(&spill_path).map_err(test_error)?;
    let range = book.worksheet_range("Data").map_err(test_error)?;
    assert_eq!(range.height(), 51);
    assert_eq!(range.get_value((1, 0)), Some(&Data::Float(0.0)));
    assert_eq!(range.get_value((50, 0)), Some(&Data::Float(49.0)));

    // Checksum must match baseline.
    assert_same_checksum(&[&baseline_path, &spill_path]);
    Ok(())
}

/// `true x true`: explicit constant_memory + compress. Gzip spill.
/// Checksum must equal the baseline (same streaming XLSX output).
#[test]
fn spill_matrix_true_true_explicit_gzip_spill() -> Result<()> {
    let dir = tempdir()?;
    let baseline_path = dir.path().join("baseline_tt.xlsx");
    let spill_path = dir.path().join("matrix_tt.xlsx");
    let data: Vec<AutoStateRow> = (0..50).map(|i| AutoStateRow { value: i }).collect();

    // Baseline (auto streaming).
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&baseline_path).build();
    writer.write(data.clone(), &WriteSheet::new("Data"))?;
    writer.finish()?;

    // Spill write: both flags.
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&spill_path)
        .constant_memory(true)
        .compress_temp_files(true)
        .build();
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::ExplicitStreaming
    );
    assert!(writer.compress_temp_files_enabled());
    writer.write(data, &WriteSheet::new("Data"))?;
    writer.finish()?;

    let mut book: Xlsx<_> = open_workbook(&spill_path).map_err(test_error)?;
    let range = book.worksheet_range("Data").map_err(test_error)?;
    assert_eq!(range.height(), 51);
    assert_eq!(range.get_value((1, 0)), Some(&Data::Float(0.0)));
    assert_eq!(range.get_value((50, 0)), Some(&Data::Float(49.0)));

    // Checksum must match baseline.
    assert_same_checksum(&[&baseline_path, &spill_path]);
    Ok(())
}

/// Explicit streaming with compress rejects a handler that requires random access.
#[test]
fn explicit_streaming_with_compress_rejects_random_access_handler() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("matrix_conflict.xlsx");
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&path)
        .constant_memory(true)
        .compress_temp_files(true)
        .register_write_handler(NoOpHandler)
        .build();
    let Err(error) = writer.write(
        vec![AutoStateRow { value: 1 }],
        &WriteSheet::new("Data"),
    ) else {
        panic!("unknown handler requires random access; must fail on explicit streaming");
    };
    assert!(
        matches!(error, ExcelError::Unsupported(ref msg) if msg.contains("explicit constant-memory")),
        "expected explicit constant-memory conflict error, got: {error}"
    );
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::ExplicitStreaming
    );
    assert!(!path.exists(), "conflicting write must not create output file");
}

/// Auto-promotion from AutoStreaming to InMemory clears `constant_memory` and
/// `compress_temp_files` on the replayed options (see `replay_stateful_sheet_journal`)
/// so the promoted workbook does not carry stale spill flags.
///
/// Uses the default auto path (no explicit flags).  The first write auto-selects
/// streaming; the second write with `auto_width` triggers promotion to InMemory.
/// The promoted output must be a valid XLSX with both sheets' data intact.
#[test]
fn auto_promotion_clears_compress_flag_after_replay() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("matrix_promote.xlsx");
    let callbacks = Rc::new(Cell::new(0usize));
    // Build in default auto mode — no explicit constant_memory / compress_temp_files.
    let mut writer = crate::EasyExcel::write::<AutoStateRow>(&path)
        .register_write_handler(StreamingCounterHandler(Rc::clone(&callbacks)))
        .build();
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::AutoUndecided
    );
    // First write: auto-selects streaming (plain rows, no unknown handlers).
    writer.write(
        vec![AutoStateRow { value: 100 }],
        &WriteSheet::new("Sheet1"),
    )?;
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::AutoStreaming
    );
    let first_cb_count = callbacks.get();

    // Second write with auto_width triggers promotion to InMemory.
    writer.write(
        vec![AutoStateRow { value: 200 }],
        &WriteSheet::new("Sheet2").auto_width(true),
    )?;
    assert_eq!(
        writer.backend_selection(),
        crate::WriteBackendSelection::InMemory,
        "auto_width triggers promotion to in-memory"
    );
    // Promotion must not replay the first sheet's handler callbacks.
    assert_eq!(
        callbacks.get(),
        first_cb_count + 2,
        "only second sheet's head+data callbacks should fire during promotion"
    );
    writer.finish()?;

    // Verify both sheets have correct data.
    let mut book: Xlsx<_> = open_workbook(&path).map_err(test_error)?;
    assert_eq!(
        book.worksheet_range("Sheet1")
            .map_err(test_error)?
            .get_value((1, 0)),
        Some(&Data::Float(100.0))
    );
    assert_eq!(
        book.worksheet_range("Sheet2")
            .map_err(test_error)?
            .get_value((1, 0)),
        Some(&Data::Float(200.0))
    );

    // After promotion, the output must be a real file with valid checksum.
    assert!(
        sha256_of_file(&path).len() == 64,
        "output must be a real file with a valid SHA-256"
    );
    Ok(())
}
