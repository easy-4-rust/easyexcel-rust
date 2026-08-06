#[test]
#[allow(clippy::too_many_lines)]
fn facade_owned_stream_matches_close_and_exception_finish_semantics() -> Result<()> {
    let xlsx = ExcelOutputStream::new(FacadeProbeWrite::default());
    let inspect_xlsx = xlsx.clone();
    let sheet = EasyExcel::writer_sheet::<Value>("Values");
    let mut writer = EasyExcel::write::<Value>("response.xlsx")
        .auto_close_stream(false)
        .to_output_stream(xlsx)
        .build();
    writer.write([Value("one".to_owned())], &sheet)?;
    writer.write([Value("two".to_owned())], &sheet)?;
    writer.finish()?;
    writer.finish()?;
    assert!(writer.is_finished());
    assert!(!inspect_xlsx.is_closed());
    assert!(
        inspect_xlsx
            .with_inner(|output| output.bytes.starts_with(b"PK"))
            .unwrap_or(false)
    );

    let csv = ExcelOutputStream::new(FacadeProbeWrite::default());
    let inspect_csv = csv.clone();
    let csv_sheet = EasyExcel::writer_sheet::<Value>("Values");
    let mut csv_writer = EasyExcel::write::<Value>("response.csv")
        .with_bom(false)
        .auto_close_stream(false)
        .to_output_stream(csv)
        .build();
    csv_writer.write([Value("one".to_owned())], &csv_sheet)?;
    csv_writer.write([Value("two".to_owned())], &csv_sheet)?;
    csv_writer.finish()?;
    assert_eq!(
        inspect_csv.with_inner(|output| output.bytes.clone()),
        Some(b"Value\none\ntwo\n".to_vec())
    );
    let mut invalid_csv_writer = EasyExcel::write::<Value>("response.csv")
        .charset("not-a-charset")
        .to_output_stream(ExcelOutputStream::new(FacadeProbeWrite::default()))
        .build();
    assert!(matches!(
        invalid_csv_writer.finish(),
        Err(ExcelError::Unsupported(_))
    ));

    let discarded = ExcelOutputStream::new(FacadeProbeWrite::default());
    let inspect_discarded = discarded.clone();
    let mut discarded_writer = EasyExcel::write::<FallibleValue>("response.xlsx")
        .auto_close_stream(false)
        .to_output_stream(discarded)
        .build();
    let fallible_sheet = EasyExcel::writer_sheet::<FallibleValue>("Values");
    discarded_writer.write(
        [FallibleValue {
            value: "kept-in-workbook",
            fail: false,
        }],
        &fallible_sheet,
    )?;
    assert!(
        discarded_writer
            .write(
                [FallibleValue {
                    value: "fail",
                    fail: true,
                }],
                &fallible_sheet,
            )
            .is_err()
    );
    discarded_writer.finish_on_exception()?;
    assert_eq!(
        inspect_discarded.with_inner(|output| output.bytes.len()),
        Some(0)
    );

    let emitted = ExcelOutputStream::new(FacadeProbeWrite::default());
    let inspect_emitted = emitted.clone();
    let mut emitted_writer = EasyExcel::write::<FallibleValue>("response.xlsx")
        .auto_close_stream(false)
        .write_excel_on_exception(true)
        .to_output_stream(emitted)
        .build();
    emitted_writer.write(
        [FallibleValue {
            value: "emitted",
            fail: false,
        }],
        &fallible_sheet,
    )?;
    emitted_writer.finish_on_exception()?;
    assert!(
        inspect_emitted
            .with_inner(|output| output.bytes.starts_with(b"PK"))
            .unwrap_or(false)
    );

    let discarded_csv = ExcelOutputStream::new(FacadeProbeWrite::default());
    let inspect_discarded_csv = discarded_csv.clone();
    let mut discarded_csv_writer = EasyExcel::write::<FallibleValue>("response.csv")
        .with_bom(false)
        .auto_close_stream(false)
        .to_output_stream(discarded_csv)
        .build();
    discarded_csv_writer.write(
        [FallibleValue {
            value: "discarded",
            fail: false,
        }],
        &fallible_sheet,
    )?;
    discarded_csv_writer.finish_on_exception()?;
    assert_eq!(
        inspect_discarded_csv.with_inner(|output| output.bytes.len()),
        Some(0)
    );

    let emitted_csv = ExcelOutputStream::new(FacadeProbeWrite::default());
    let inspect_emitted_csv = emitted_csv.clone();
    let mut emitted_csv_writer = EasyExcel::write::<FallibleValue>("response.csv")
        .with_bom(false)
        .auto_close_stream(false)
        .write_excel_on_exception(true)
        .to_output_stream(emitted_csv)
        .build();
    emitted_csv_writer.write(
        [FallibleValue {
            value: "emitted",
            fail: false,
        }],
        &fallible_sheet,
    )?;
    emitted_csv_writer.finish_on_exception()?;
    assert_eq!(
        inspect_emitted_csv.with_inner(|output| output.bytes.clone()),
        Some(b"Value\nemitted\n".to_vec())
    );

    for output in [
        FacadeProbeWrite {
            fail_write: true,
            ..FacadeProbeWrite::default()
        },
        FacadeProbeWrite {
            fail_flush: true,
            ..FacadeProbeWrite::default()
        },
    ] {
        let mut failed_commit = EasyExcel::write::<Value>("response.csv")
            .with_bom(false)
            .auto_close_stream(false)
            .to_output_stream(ExcelOutputStream::new(output))
            .build();
        failed_commit.write([Value("failure".to_owned())], &csv_sheet)?;
        assert!(failed_commit.finish().is_err());
    }

    let one_shot = ExcelOutputStream::new(FacadeProbeWrite::default());
    let inspect_one_shot = one_shot.clone();
    assert!(
        EasyExcel::write::<FallibleValue>("response.xlsx")
            .auto_close_stream(false)
            .write_excel_on_exception(true)
            .to_output_stream(one_shot)
            .do_write([
                FallibleValue {
                    value: "emitted-before-error",
                    fail: false,
                },
                FallibleValue {
                    value: "error",
                    fail: true,
                },
            ])
            .is_err()
    );
    assert!(
        inspect_one_shot
            .with_inner(|output| output.bytes.starts_with(b"PK"))
            .unwrap_or(false)
    );

    let cleanup_failure = ExcelOutputStream::new(FacadeProbeWrite {
        fail_write: true,
        ..FacadeProbeWrite::default()
    });
    assert!(
        EasyExcel::write::<FallibleValue>("response.xlsx")
            .auto_close_stream(false)
            .write_excel_on_exception(true)
            .to_output_stream(cleanup_failure)
            .do_write([
                FallibleValue {
                    value: "emitted-before-error",
                    fail: false,
                },
                FallibleValue {
                    value: "error",
                    fail: true,
                },
            ])
            .is_err()
    );

    let mut invalid_encrypted = EasyExcel::write::<Value>("response.xlsx")
        .password("123456")
        .auto_close_stream(false)
        .to_output_stream(ExcelOutputStream::new(FacadeProbeWrite::default()))
        .build();
    invalid_encrypted
        .workbook_mut()
        .add_worksheet()
        .set_name("Duplicate")
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    invalid_encrypted
        .workbook_mut()
        .add_worksheet()
        .set_name("Duplicate")
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    assert!(invalid_encrypted.finish().is_err());

    let closed = ExcelOutputStream::new(FacadeProbeWrite::default());
    let inspect_closed = closed.clone();
    EasyExcel::write::<FallibleValue>("response.xlsx")
        .to_output_stream(closed)
        .do_write([
            FallibleValue {
                value: "closed-one",
                fail: false,
            },
            FallibleValue {
                value: "closed-two",
                fail: false,
            },
        ])?;
    assert!(inspect_closed.is_closed());
    inspect_closed.close()?;
    let mut closed_writer = inspect_closed.clone();
    assert!(closed_writer.write_all(b"rejected").is_err());
    assert!(closed_writer.flush().is_err());

    let failed_close = ExcelOutputStream::new(FacadeProbeWrite {
        fail_flush: true,
        ..FacadeProbeWrite::default()
    });
    assert!(failed_close.close().is_err());

    let poisoned_close = ExcelOutputStream::new(FacadeProbeWrite::default());
    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        poisoned_close.with_inner(|_| panic!("poison facade output lock"));
    }));
    assert!(panic_result.is_err());
    assert!(poisoned_close.close().is_err());
    let mut poisoned_writer = poisoned_close.clone();
    assert!(poisoned_writer.write_all(b"rejected").is_err());
    assert!(poisoned_writer.flush().is_err());
    Ok(())
}

