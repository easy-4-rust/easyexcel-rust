/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#annotationStyleWrite`
#[test]
fn write_test_annotation_style_write() {
    let path = temp_path("annotationStyleWrite.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .head_style(CellStyle::default())
        .content_style(CellStyle::default())
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert_write_10(&path);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#handlerStyleWrite`
#[test]
fn write_test_handler_style_write() {
    let path = temp_path("handlerStyle.xlsx");
    let strategy = HorizontalCellStyleStrategy::new(vec![ExcelCellStyle::new()]);
    EasyExcel::write::<WriteDemoData>(&path)
        .register_write_handler(strategy)
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert!(path.exists());
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#mergeWrite`
#[test]
fn write_test_merge_write() {
    let path = temp_path("mergeWrite.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .loop_merge(LoopMergeStrategy::new(2, 1, 0).unwrap())
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert_write_10(&path);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#tableWrite`
#[test]
fn write_test_table_write() {
    let path = temp_path("tableWrite.xlsx");
    let mut writer = EasyExcel::write::<WriteDemoData>(&path).build();
    let sheet = EasyExcel::writer_sheet::<WriteDemoData>("模板");
    writer.write(write_demo_data(), &sheet).unwrap();
    writer.write(write_demo_data(), &sheet).unwrap();
    writer.finish().unwrap();
    assert_eq!(
        EasyExcel::read_sync::<WriteDemoData>(&path)
            .do_read_sync()
            .unwrap()
            .len(),
        20
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#dynamicHeadWrite`
#[test]
fn write_test_dynamic_head_write() {
    let path = temp_path("dynamicHead.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .head([["字符串标题"], ["日期标题"], ["数字标题"]])
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert_write_10(&path);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#longestMatchColumnWidthWrite`
#[test]
fn write_test_longest_match_column_width_write() {
    let path = temp_path("longestMatch.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .register_write_handler(LongestMatchColumnWidthStyleStrategy::new())
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert_write_10(&path);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#customHandlerWrite`
#[test]
fn write_test_custom_handler_write() {
    #[derive(Default)]
    struct CountingHandler {
        hits: Arc<AtomicUsize>,
    }
    impl WriteHandler for CountingHandler {
        fn after_workbook(&mut self, _ctx: &WriteWorkbookContext) -> Result<()> {
            self.hits.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }
    let hits = Arc::new(AtomicUsize::new(0));
    let path = temp_path("customHandlerWrite.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .register_write_handler(CountingHandler { hits: hits.clone() })
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert!(hits.load(Ordering::Relaxed) >= 1);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#commentWrite`
#[test]
fn write_test_comment_write() {
    #[derive(Debug, Clone, ExcelRow)]
    struct CommentRow {
        #[excel(name = "字符串标题")]
        string: WriteCellData,
        #[excel(name = "日期标题")]
        date: WriteCellData,
    }
    let path = temp_path("commentWrite.xlsx");
    let rows: Vec<CommentRow> = (0..10)
        .map(|i| CommentRow {
            string: WriteCellData::from_string(format!("字符串{i}")),
            date: WriteCellData::from_string("2020-01-01")
                .comment_data(CommentData::new().author("Jiaju Zhuang").text("创建批注!")),
        })
        .collect();
    EasyExcel::write::<CommentRow>(&path)
        .sheet("模板")
        .do_write(rows)
        .unwrap();
    assert!(path.exists());
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#variableTitleWrite`
#[test]
fn write_test_variable_title_write() {
    let path = temp_path("variableTitleWrite.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .head([["字符串标题"], ["日期标题"], ["数字标题"]])
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert_write_10(&path);
}

/// Java: `com.alibaba.easyexcel.test.demo.write.WriteTest#noModelWrite`
#[test]
fn write_test_no_model_write() {
    let path = temp_path("noModelWrite.xlsx");
    let rows: Vec<DynamicRow> = (0..10)
        .map(|i| {
            let mut map = BTreeMap::new();
            map.insert(0, DynamicValue::String(format!("字符串{i}")));
            map.insert(1, DynamicValue::String("2020-01-01".to_owned()));
            map.insert(2, DynamicValue::String("0.56".to_owned()));
            DynamicRow::new(map)
        })
        .collect();
    EasyExcel::write::<DynamicRow>(&path)
        .head([["字符串标题"], ["日期标题"], ["数字标题"]])
        .sheet("模板")
        .do_write(rows)
        .unwrap();
    assert!(
        !EasyExcel::read_dynamic_sync(&path)
            .do_read_sync()
            .unwrap()
            .is_empty()
    );
}

// ============================================================================
// fill.FillTest — 6
// ============================================================================

/// Java: `com.alibaba.easyexcel.test.demo.fill.FillTest#simpleFill`
#[test]
fn fill_test_simple_fill() {
    let template = require_fixture("demo/fill/simple.xlsx");
    let output = temp_path("simpleFill.xlsx");
    let data = TemplateData::new().with("name", "张三").with("number", 5.2);
    EasyExcel::fill_template(&template, &output, &data).unwrap();
    let rows = EasyExcel::read_dynamic_sync(&output)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    assert!(!rows.is_empty());
    let text = format!("{rows:?}");
    assert!(text.contains("张三") || text.contains('5'), "{text}");
}

/// Java: `com.alibaba.easyexcel.test.demo.fill.FillTest#listFill`
#[test]
fn fill_test_list_fill() {
    let template = require_fixture("demo/fill/list.xlsx");
    let output = temp_path("listFill.xlsx");
    let items: Vec<_> = (0..10)
        .map(|i| {
            TemplateData::new()
                .with("name", format!("张三{i}"))
                .with("number", f64::from(i))
        })
        .collect();
    EasyExcel::fill_template_list(
        &template,
        &output,
        &FillWrapper::new(items),
        FillConfig::new(),
    )
    .unwrap();
    assert!(output.exists());
}

/// Java: `com.alibaba.easyexcel.test.demo.fill.FillTest#complexFill`
#[test]
fn fill_test_complex_fill() {
    let template = require_fixture("demo/fill/complex.xlsx");
    let output = temp_path("complexFill.xlsx");
    let items: Vec<_> = (0..5)
        .map(|i| {
            TemplateData::new()
                .with("name", format!("张三{i}"))
                .with("number", 5.2)
        })
        .collect();
    EasyExcel::fill_template_list(
        &template,
        &output,
        &FillWrapper::new(items),
        FillConfig::new().force_new_row(true),
    )
    .unwrap();
    assert!(output.exists());
}

/// Java: `com.alibaba.easyexcel.test.demo.fill.FillTest#complexFillWithTable`
#[test]
fn fill_test_complex_fill_with_table() {
    let template = require_fixture("demo/fill/complexFillWithTable.xlsx");
    let output = temp_path("complexFillWithTable.xlsx");
    let items: Vec<_> = (0..5)
        .map(|i| {
            TemplateData::new()
                .with("name", format!("张三{i}"))
                .with("number", 5.2)
        })
        .collect();
    EasyExcel::fill_template_list(
        &template,
        &output,
        &FillWrapper::new(items),
        FillConfig::new(),
    )
    .unwrap();
    assert!(output.exists());
}

/// Java: `com.alibaba.easyexcel.test.demo.fill.FillTest#horizontalFill`
#[test]
fn fill_test_horizontal_fill() {
    let template = require_fixture("demo/fill/horizontal.xlsx");
    let output = temp_path("horizontalFill.xlsx");
    let data = TemplateData::new().with("name", "张三").with("number", 5.2);
    EasyExcel::fill_template(&template, &output, &data).unwrap();
    assert!(output.exists());
}

/// Java: `com.alibaba.easyexcel.test.demo.fill.FillTest#compositeFill`
#[test]
fn fill_test_composite_fill() {
    let template = require_fixture("demo/fill/composite.xlsx");
    let output = temp_path("compositeFill.xlsx");
    let data = TemplateData::new()
        .with("date", "2019年10月9日")
        .with("total", 1000);
    EasyExcel::fill_template(&template, &output, &data).unwrap();
    assert!(output.exists());
}

// ============================================================================
// rare.WriteTest — 2
// ============================================================================

/// Java: `com.alibaba.easyexcel.test.demo.rare.WriteTest#compressedTemporaryFile`
///
/// Java enables `SXSSFWorkbook.setCompressTempFiles(true)` in `afterWorkbookCreate`
/// so SXSSF gzips spilled sheet XML (CPU for disk). Rust maps that flag to
/// [`easyexcel::ExcelWriterBuilder::compress_temp_files`], which forces
/// constant-memory output and mirrors rows through a gzip spill.
///
/// Coverage:
/// 1. Builder API is wired (not `ExcelError::Unsupported`).
/// 2. Multi-batch stateful write under spill mode (volume intent of Java demo).
#[test]
fn rare_test_compressed_temporary_file() {
    let path = temp_path("rare_compressedTemporaryFile.xlsx");
    // Java: afterWorkbookCreate → sxssfWorkbook.setCompressTempFiles(true)
    let mut writer = EasyExcel::write::<WriteDemoData>(&path)
        .compress_temp_files(true)
        .build();
    assert!(writer.compress_temp_files_enabled());
    let sheet = EasyExcel::writer_sheet::<WriteDemoData>("模板");
    // Java loops 10_000 × 10 rows; keep a smaller but still multi-batch volume.
    for _ in 0..50 {
        writer.write(write_demo_data(), &sheet).unwrap();
    }
    writer.finish().unwrap();
    let spill = writer
        .last_gzip_spill_snapshot()
        .expect("compress_temp_files must produce an observable gzip spill");
    assert!(spill.is_gzip);
    assert!(spill.compressed_len > 0);
    assert!(spill.uncompressed_len > 0);
    assert_eq!(
        EasyExcel::read_sync::<WriteDemoData>(&path)
            .do_read_sync()
            .unwrap()
            .len(),
        500
    );
}

/// Java: `com.alibaba.easyexcel.test.demo.rare.WriteTest#specifiedCellWrite`
///
/// Java:
/// - `RowWriteHandler.afterRowDispose` mutates cell (2,2) on row 2
/// - `WorkbookWriteHandler.afterWorkbookDispose` appends cell on row 99 via POI
///
/// Rust 使用 handler 的后端中立修改计划表达相同的保存前修改。
#[test]
fn rare_test_specified_cell_write() {
    struct SpecifiedCellHandler {
        after_workbook_hits: Arc<AtomicUsize>,
    }
    impl WriteHandler for SpecifiedCellHandler {
        fn before_cell(&mut self, ctx: &mut WriteCellContext) -> Result<()> {
            // Java: afterRowDispose when rowNum == 2 → cell(2) = "测试的第二行数据呀"
            if !ctx.is_head && ctx.row_index == 2 && ctx.column_index == 2 {
                ctx.value = CellValue::String("测试的第二行数据呀".to_owned());
            }
            Ok(())
        }
        fn after_workbook(&mut self, ctx: &WriteWorkbookContext) -> Result<()> {
            self.after_workbook_hits.fetch_add(1, Ordering::Relaxed);
            ctx.set_cell(
                "模板",
                99,
                2,
                CellValue::String("测试的最后一行数据呀".to_owned()),
            )
        }
    }

    let hits = Arc::new(AtomicUsize::new(0));
    let path = temp_path("rare_specifiedCellWrite.xlsx");
    EasyExcel::write::<WriteDemoData>(&path)
        .register_write_handler(SpecifiedCellHandler {
            after_workbook_hits: hits.clone(),
        })
        .sheet("模板")
        .do_write(write_demo_data())
        .unwrap();
    assert!(hits.load(Ordering::Relaxed) >= 1);

    let rows = EasyExcel::read_dynamic_sync(&path)
        .sheet(0usize)
        .head_row_number(0)
        .do_read_sync()
        .unwrap();
    let text = format!("{rows:?}");
    assert!(
        text.contains("测试的第二行数据呀"),
        "before_cell mutation must appear in output: {text}"
    );
    assert!(
        text.contains("测试的最后一行数据呀"),
        "after_workbook mutation must appear in output: {text}"
    );
}
