#[test]
fn collection_and_map_row_data_enter_the_public_writer_backends() -> Result<()> {
    let directory = tempdir()?;
    let collection = CollectionRowData::new(vec![
        CellValue::String("collection".to_owned()),
        CellValue::Int(1),
    ]);
    let map = MapRowData::new(
        [
            (0, CellValue::String("map".to_owned())),
            (1, CellValue::Int(2)),
        ]
        .into_iter()
        .collect(),
    );

    let xlsx_path = directory.path().join("row-data.xlsx");
    let mut writer = ExcelWriter::new(&xlsx_path);
    writer.write(
        [collection.clone()],
        &WriteSheet::<CollectionRowData>::new("Rows").need_head(false),
    )?;
    writer.write(
        [map.clone()],
        &WriteSheet::<MapRowData>::new("Rows").need_head(false),
    )?;
    writer.finish()?;
    let mut xlsx: Xlsx<_> = open_workbook(&xlsx_path).map_err(test_error)?;
    let xlsx_range = xlsx.worksheet_range("Rows").map_err(test_error)?;
    assert_eq!(
        xlsx_range.get_value((0, 0)),
        Some(&Data::String("collection".to_owned()))
    );
    assert_eq!(
        xlsx_range.get_value((1, 0)),
        Some(&Data::String("map".to_owned()))
    );

    let xls_path = directory.path().join("collection-row.xls");
    write_xls::<CollectionRowData, _>(
        &xls_path,
        &WriteOptions {
            need_head: false,
            sheet_name: "Rows".to_owned(),
            ..WriteOptions::default()
        },
        [collection],
    )?;
    let mut xls: Xls<_> = open_workbook(&xls_path).map_err(test_error)?;
    assert_eq!(
        xls.worksheet_range("Rows")
            .map_err(test_error)?
            .get_value((0, 1)),
        Some(&Data::Int(1))
    );

    let csv = write_csv_to_buffer::<MapRowData, _>(
        Path::new("map-row.csv"),
        &WriteOptions {
            need_head: false,
            with_bom: false,
            ..WriteOptions::default()
        },
        [map],
        &mut [],
    )?;
    let text = String::from_utf8(csv).map_err(test_error)?;
    assert_eq!(text.trim_end(), "map,2");

    let sparse = MapRowData::new(
        [
            (0, CellValue::String("kept".to_owned())),
            (2, CellValue::String("outside-size".to_owned())),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        <MapRowData as ExcelRow>::to_row(&sparse)?,
        vec![CellValue::String("kept".to_owned()), CellValue::Empty]
    );

    let read_row = crate::core::RowData::new(
        "Rows",
        0,
        vec![CellValue::String("read".to_owned()), CellValue::Int(3)],
        Arc::new(std::collections::HashMap::new()),
    );
    assert_eq!(
        <CollectionRowData as ExcelRow>::from_row(&read_row)?.values(),
        &[CellValue::String("read".to_owned()), CellValue::Int(3)]
    );
    assert_eq!(
        <MapRowData as ExcelRow>::from_row(&read_row)?.values(),
        &[
            (0, CellValue::String("read".to_owned())),
            (1, CellValue::Int(3))
        ]
        .into_iter()
        .collect()
    );
    Ok(())
}

#[test]
// 语义敏感：xlsx/xls 双后端并行断言，命名刻意对照，故豁免 similar_names。
#[allow(clippy::similar_names)]
// 语义敏感：该测试端到端覆盖 Java 对应用例的完整流程，
// 拆分会降低可读性，故豁免 too_many_lines。
#[allow(clippy::too_many_lines)]
fn absent_option_rows_keep_indexes_without_rows_cells_or_handlers() -> Result<()> {
    struct NeverConvert;

    impl ExcelRow for NeverConvert {
        fn schema() -> &'static [ExcelColumn] {
            &[]
        }

        fn from_row(_row: &crate::core::RowData) -> Result<Self> {
            panic!("absent rows must not be decoded")
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            panic!("absent rows must not be converted")
        }
    }

    #[derive(Default)]
    struct Events {
        rows: std::sync::Mutex<Vec<(u32, Option<usize>)>>,
        cells: std::sync::Mutex<Vec<(u32, Option<usize>)>>,
    }

    struct Probe {
        events: Arc<Events>,
    }

    impl WriteHandler for Probe {
        fn before_row_create(&mut self, context: &WriteRowContext) -> Result<()> {
            self.events
                .rows
                .lock()
                .expect("row events")
                .push((context.row_index, context.relative_row_index));
            Ok(())
        }

        fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
            self.events
                .cells
                .lock()
                .expect("cell events")
                .push((context.row_index, context.relative_row_index));
            Ok(())
        }
    }

    fn rows() -> Vec<Option<CollectionRowData>> {
        vec![
            Some(CollectionRowData::new(vec![CellValue::String(
                "first".to_owned(),
            )])),
            None,
            Some(CollectionRowData::new(vec![CellValue::String(
                "third".to_owned(),
            )])),
        ]
    }

    fn assert_events(events: &Events) {
        assert_eq!(
            *events.rows.lock().expect("row events"),
            vec![(0, Some(0)), (2, Some(2))]
        );
        assert_eq!(
            *events.cells.lock().expect("cell events"),
            vec![(0, Some(0)), (2, Some(2))]
        );
    }

    let directory = tempdir()?;
    let options = WriteOptions {
        need_head: false,
        sheet_name: "Rows".to_owned(),
        with_bom: false,
        ..WriteOptions::default()
    };
    write_xlsx::<Option<NeverConvert>, _>(
        &directory.path().join("absent-never-convert.xlsx"),
        &options,
        [None],
    )?;

    let xlsx_path = directory.path().join("absent.xlsx");
    let xlsx_events = Arc::new(Events::default());
    write_xlsx_with_handlers::<Option<CollectionRowData>, _>(
        &xlsx_path,
        &options,
        rows(),
        &mut [Box::new(Probe {
            events: Arc::clone(&xlsx_events),
        })],
    )?;
    assert_events(&xlsx_events);
    let xlsx_xml = zip_entry(&xlsx_path, "xl/worksheets/sheet1.xml")?;
    assert!(xlsx_xml.contains("<row r=\"1\""));
    assert!(!xlsx_xml.contains("<row r=\"2\""));
    assert!(xlsx_xml.contains("<row r=\"3\""));

    let xls_path = directory.path().join("absent.xls");
    let xls_events = Arc::new(Events::default());
    write_xls_with_handlers::<Option<CollectionRowData>, _>(
        &xls_path,
        &options,
        rows(),
        &mut [Box::new(Probe {
            events: Arc::clone(&xls_events),
        })],
    )?;
    assert_events(&xls_events);
    let mut xls: Xls<_> = open_workbook(&xls_path).map_err(test_error)?;
    let range = xls.worksheet_range("Rows").map_err(test_error)?;
    assert_eq!(
        range.get_value((0, 0)),
        Some(&Data::String("first".to_owned()))
    );
    assert_eq!(range.get_value((1, 0)), Some(&Data::Empty));
    assert_eq!(
        range.get_value((2, 0)),
        Some(&Data::String("third".to_owned()))
    );

    let csv_events = Arc::new(Events::default());
    let csv = write_csv_to_buffer::<Option<CollectionRowData>, _>(
        Path::new("absent.csv"),
        &options,
        rows(),
        &mut [Box::new(Probe {
            events: Arc::clone(&csv_events),
        })],
    )?;
    assert_events(&csv_events);
    assert_eq!(
        String::from_utf8(csv).map_err(test_error)?,
        "first\nthird\n"
    );

    let template_path = directory.path().join("empty-template.xlsx");
    write_xlsx::<CollectionRowData, _>(&template_path, &options, Vec::<CollectionRowData>::new())?;
    let templated_path = directory.path().join("absent-template.xlsx");
    let template_events = Arc::new(Events::default());
    write_xlsx_with_handlers::<Option<CollectionRowData>, _>(
        &templated_path,
        &WriteOptions {
            template_file: Some(template_path),
            ..options
        },
        rows(),
        &mut [Box::new(Probe {
            events: Arc::clone(&template_events),
        })],
    )?;
    assert_events(&template_events);
    let template_xml = zip_entry(&templated_path, "xl/worksheets/sheet1.xml")?;
    assert!(template_xml.contains("<row r=\"1\""));
    assert!(!template_xml.contains("<row r=\"2\""));
    assert!(template_xml.contains("<row r=\"3\""));
    Ok(())
}

