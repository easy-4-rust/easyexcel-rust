include!("cases_02_split/chunk_01.rs");

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
struct ConverterRow {
    #[excel(name = "Value", index = 0)]
    value: String,
}

#[derive(Clone, Copy)]
struct PrefixConverter {
    prefix: &'static str,
    cell_type: CellDataType,
}

impl PrefixConverter {
    const fn string(prefix: &'static str) -> Self {
        Self {
            prefix,
            cell_type: CellDataType::String,
        }
    }
}

impl Converter<String> for PrefixConverter {
    fn support_excel_type(&self) -> CellDataType {
        self.cell_type
    }

    fn convert_to_rust_data(&self, context: &ReadConverterContext<'_>) -> Result<String> {
        Ok(format!(
            "{}:{}",
            self.prefix,
            context.cell().map_or_else(String::new, CellValue::as_text)
        ))
    }

    fn convert_to_excel_data(
        &self,
        context: &WriteConverterContext<'_, String>,
    ) -> Result<WriteCellData> {
        Ok(WriteCellData::from_string(format!(
            "{}:{}",
            self.prefix,
            context.value()
        )))
    }
}

#[derive(Default)]
struct FieldPrefixConverter;

impl Converter<String> for FieldPrefixConverter {
    fn convert_to_rust_data(&self, context: &ReadConverterContext<'_>) -> Result<String> {
        Ok(format!(
            "field:{}",
            context.cell().map_or_else(String::new, CellValue::as_text)
        ))
    }

    fn convert_to_excel_data(
        &self,
        context: &WriteConverterContext<'_, String>,
    ) -> Result<WriteCellData> {
        Ok(WriteCellData::from_string(format!(
            "field:{}",
            context.value()
        )))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
struct FieldConverterRow {
    #[excel(name = "Value", index = 0, converter = FieldPrefixConverter)]
    value: String,
}

#[derive(Default)]
struct RejectingWriteConverter;

impl Converter<String> for RejectingWriteConverter {
    fn convert_to_excel_data(
        &self,
        context: &WriteConverterContext<'_, String>,
    ) -> Result<WriteCellData> {
        if context.value() == "fail" {
            Err(ExcelError::Format("converter rejected value".to_owned()))
        } else {
            Ok(WriteCellData::from_string(context.value().clone()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
struct LocatedWriteFailureRow {
    #[excel(name = "Forced", index = 2)]
    forced: String,
    #[excel(name = "Late", order = 20)]
    late: String,
    #[excel(name = "Failing", order = 10, converter = RejectingWriteConverter)]
    failing: String,
}

struct WideCell(CellValue);

impl ExcelRow for WideCell {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] =
            &[ExcelColumn::new("value", "Value", Some(16_384), 0, None)];
        COLUMNS
    }

    fn from_row(_row: &RowData) -> Result<Self> {
        Err(ExcelError::Unsupported("write-only test row".to_owned()))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![self.0.clone()])
    }
}

struct SingleCell(CellValue);

impl ExcelRow for SingleCell {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
        COLUMNS
    }

    fn from_row(_row: &RowData) -> Result<Self> {
        Err(ExcelError::Unsupported("write-only test row".to_owned()))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![self.0.clone()])
    }
}

fn tiny_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8,
        0xcf, 0xc0, 0xf0, 0x1f, 0x00, 0x05, 0x00, 0x01, 0xff, 0x89, 0x99, 0x3d, 0x1d, 0x00, 0x00,
        0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ]
}

#[derive(Default)]
struct Listener(Vec<Value>);

struct FailingListener;

struct NoopWriteHandler;

#[derive(Clone, Default)]
struct DynamicListener(Arc<Mutex<Vec<DynamicRow>>>);

#[derive(Clone, Default)]
struct ConverterListener(Arc<Mutex<Vec<ConverterRow>>>);

impl WriteHandler for NoopWriteHandler {}

struct FailingFacadeWriteHandler {
    before_workbook: bool,
    before_cell: bool,
}

impl WriteHandler for FailingFacadeWriteHandler {
    fn before_workbook(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        if self.before_workbook {
            Err(ExcelError::Format(
                "injected before-workbook failure".to_owned(),
            ))
        } else {
            Ok(())
        }
    }

    fn before_cell(&mut self, _context: &mut WriteCellContext) -> Result<()> {
        if self.before_cell {
            Err(ExcelError::Format(
                "injected before-cell failure".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}

impl ReadListener<Value> for Listener {
    fn invoke(&mut self, data: Value, _context: &AnalysisContext) -> Result<()> {
        self.0.push(data);
        Ok(())
    }
}

impl ReadListener<Value> for FailingListener {
    fn invoke_head(
        &mut self,
        _head: &std::collections::HashMap<String, usize>,
        _context: &AnalysisContext,
    ) -> Result<()> {
        Err(ExcelError::Format("injected listener failure".to_owned()))
    }

    fn invoke(&mut self, _data: Value, _context: &AnalysisContext) -> Result<()> {
        Ok(())
    }
}

impl ReadListener<DynamicRow> for DynamicListener {
    fn invoke(&mut self, data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
        self.0.lock().expect("dynamic listener lock").push(data);
        Ok(())
    }
}

impl ReadListener<ConverterRow> for ConverterListener {
    fn invoke(&mut self, data: ConverterRow, _context: &AnalysisContext) -> Result<()> {
        self.0.lock().expect("converter listener lock").push(data);
        Ok(())
    }
}





include!("cases_02_split/chunk_02.rs");

