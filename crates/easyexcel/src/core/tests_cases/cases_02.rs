include!("cases_02_split/chunk_01.rs");

// ============================================================================
// 19. ReadListener tests (Java: ExceptionDataTest)
// ============================================================================

struct TestListener {
    rows: Vec<String>,
    _batch_idx: usize,
}

impl TestListener {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            _batch_idx: 0,
        }
    }
}

impl ReadListener<String> for TestListener {
    fn invoke(&mut self, data: String, _context: &AnalysisContext) -> Result<()> {
        self.rows.push(data);
        Ok(())
    }

    fn on_exception(&mut self, _error: &ExcelError, _context: &AnalysisContext) -> ErrorAction {
        ErrorAction::Continue
    }

    fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> Result<()> {
        Ok(())
    }
}







// ============================================================================
// 20. PageReadListener tests (Java: PageReadListenerTest)
// ============================================================================





// ============================================================================
// 21. ExcelError conversion chain
// ============================================================================





// ============================================================================
// 22. AnalysisContext tests
// ============================================================================







// ============================================================================
// 23. WriteHandler tests (Java: WriteHandlerTest)
// ============================================================================

struct TestWriteHandler {
    order: i32,
    before_workbook_called: std::sync::atomic::AtomicBool,
    before_cell_value: Option<CellValue>,
}

impl WriteHandler for TestWriteHandler {
    fn order(&self) -> i32 {
        self.order
    }
    fn before_workbook(&mut self, _ctx: &WriteWorkbookContext) -> Result<()> {
        self.before_workbook_called
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    fn before_cell(&mut self, ctx: &mut WriteCellContext) -> Result<()> {
        self.before_cell_value = Some(ctx.value.clone());
        Ok(())
    }
}





// ============================================================================
// 24. ConverterRegistry tests (Java: ConverterTest)
// ============================================================================

struct PrefixConverter;

impl Converter<String> for PrefixConverter {
    fn convert_to_rust_data(&self, ctx: &ReadConverterContext<'_>) -> Result<String> {
        let val = ctx.cell().map_or_else(String::new, CellValue::as_text);
        Ok(format!("custom:{val}"))
    }
    fn convert_to_excel_data(
        &self,
        ctx: &WriteConverterContext<'_, String>,
    ) -> Result<WriteCellData> {
        Ok(WriteCellData::from_string(format!(
            "custom:{}",
            ctx.value()
        )))
    }
}







// ============================================================================
// 25. StringImageConverter tests
// ============================================================================



// ============================================================================
// 26. UrlImageConverter tests
// ============================================================================



// ============================================================================
// 27. ExcelTypeEnum tests (Java: ExcelTypeEnumTest)
// ============================================================================





// ============================================================================
// 28. BuiltinFormats tests (Java: BuiltinFormatsTest)
// ============================================================================





// ============================================================================
// 29. ExcelXmlConstants tests
// ============================================================================



// ============================================================================
// 30. EasyExcelConstants tests
// ============================================================================



// ============================================================================
// 31. WriteWorkbookContext / WriteSheetContext / WriteRowContext tests
// ============================================================================

include!("cases_02_split/chunk_02.rs");







// ============================================================================
// 32. BooleanEnum tests
// ============================================================================



// ============================================================================
// 33. ExcelHorizontalAlignment / VerticalAlignment / BorderStyle / FillPattern tests
// ============================================================================







// ============================================================================
// 34. ExcelDataFormat tests
// ============================================================================



// ============================================================================
// 35. ReadCellData / WriteCellData integration
// ============================================================================



// ============================================================================
// 36. DynamicRow ExcelRow impl
// ============================================================================



// ============================================================================
// 37. RowData display_values and decimal_values
// ============================================================================

