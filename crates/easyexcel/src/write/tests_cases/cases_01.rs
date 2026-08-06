include!("cases_01_split/chunk_01.rs");













impl ExcelRow for DimensionRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("field", "Field", Some(0), 0, None).with_column_width(30),
            ExcelColumn::new("type", "Type", Some(1), 0, None),
            ExcelColumn::new("explicit", "Explicit", Some(2), 0, None),
        ];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
            .column_width(18)
            .head_row_height(24)
            .content_row_height(16);
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self)
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![
            CellValue::String("field".to_owned()),
            CellValue::String("type".to_owned()),
            CellValue::String("explicit".to_owned()),
        ])
    }
}

impl ExcelRow for SparseRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] =
            &[ExcelColumn::new("value", "Value", Some(10_000), 0, None)];
        COLUMNS
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self)
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![CellValue::String("value".to_owned())])
    }
}

#[allow(dead_code)]
struct AnchoredImageRow {
    cell: WriteCellData,
}

impl ExcelRow for AnchoredImageRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] =
            &[ExcelColumn::new("cell", "Images", Some(0), 0, None).with_column_width(20)];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
            .head_row_height(18)
            .content_row_height(30);
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self {
            cell: WriteCellData::new(CellValue::Empty),
        })
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![self.cell.to_excel_cell(
            &crate::core::ConvertContext {
                sheet_name: "Images".to_owned(),
                row_index: 1,
                column_index: Some(0),
                field: "cell",
                format: None,
                date_time_format: None,
                number_format: None,
                use_1904_windowing: false,
            },
        )?])
    }
}

#[allow(dead_code)]
struct RichTextRow {
    value: RichTextStringData,
}

impl ExcelRow for RichTextRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Rich", Some(0), 0, None)];
        COLUMNS
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self {
            value: RichTextStringData::default(),
        })
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![CellValue::RichText(self.value.clone())])
    }
}

impl ExcelRow for EveryCell {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("empty", "Empty", Some(0), 0, None),
            ExcelColumn::new("string", "String", Some(1), 0, None),
            ExcelColumn::new("error", "Error", Some(2), 0, None),
            ExcelColumn::new("boolean", "Boolean", Some(3), 0, None),
            ExcelColumn::new("integer", "Integer", Some(4), 0, None),
            ExcelColumn::new("float", "Float", Some(5), 0, None),
            ExcelColumn::new("date", "Date", Some(6), 0, Some("%d/%m/%Y")),
            ExcelColumn::new(
                "datetime",
                "DateTime",
                Some(7),
                0,
                Some("%Y-%m-%d %H:%M:%S"),
            ),
            ExcelColumn::new("large", "Large", Some(8), 0, None),
            ExcelColumn::new("missing", "Missing", Some(9), 0, None),
            ExcelColumn::new("formula", "Formula", Some(10), 0, None),
            ExcelColumn::new("link", "Link", Some(11), 0, None),
            ExcelColumn::new("comment", "Comment", Some(12), 0, None),
            ExcelColumn::new("image", "Image", Some(13), 0, None),
            ExcelColumn::new("decimal", "Decimal", Some(14), 0, None),
        ];
        const WIDE_COLUMNS: &[ExcelColumn] =
            &[ExcelColumn::new("wide", "Wide", Some(65_536), 0, None)];
        const ANNOTATED_WIDE_COLUMNS: &[ExcelColumn] =
            &[ExcelColumn::new("wide", "Wide", Some(65_536), 0, None).with_column_width(10)];
        const BACKEND_WIDE_COLUMNS: &[ExcelColumn] =
            &[ExcelColumn::new("wide", "Wide", Some(65_535), 0, None).with_column_width(10)];
        USE_BACKEND_WIDE_SCHEMA.with(|backend_wide| {
            if backend_wide.get() {
                BACKEND_WIDE_COLUMNS
            } else {
                USE_ANNOTATED_WIDE_SCHEMA.with(|annotated_wide| {
                    if annotated_wide.get() {
                        ANNOTATED_WIDE_COLUMNS
                    } else {
                        USE_WIDE_SCHEMA.with(|wide| if wide.get() { WIDE_COLUMNS } else { COLUMNS })
                    }
                })
            }
        })
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Err(ExcelError::Unsupported("test-only writer row".to_owned()))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        if self.fail {
            return Err(ExcelError::Format("row conversion failed".to_owned()));
        }
        Ok(self.cells.clone())
    }
}

#[allow(dead_code)]
fn every_cell() -> EveryCell {
    let date = NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid date");
    EveryCell {
        cells: vec![
            CellValue::Empty,
            CellValue::String("text".to_owned()),
            CellValue::Error("#DIV/0!".to_owned()),
            CellValue::Bool(true),
            CellValue::Int(-12),
            CellValue::Float(1.25),
            CellValue::Date(date),
            CellValue::DateTime(date.and_hms_opt(12, 34, 56).expect("valid time")),
            CellValue::Int(i64::MAX),
            CellValue::Empty,
            CellValue::Formula("SUM(E2:F2)".to_owned()),
            CellValue::Hyperlink {
                url: "https://www.rust-lang.org".to_owned(),
                text: "Rust".to_owned(),
            },
            CellValue::Comment {
                value: Box::new(CellValue::String("annotated".to_owned())),
                text: "cell note".to_owned(),
            },
            CellValue::Image(tiny_png()),
            CellValue::Decimal("123.45".parse().expect("valid decimal")),
        ],
        fail: false,
    }
}

#[allow(dead_code)]
fn tiny_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8,
        0xcf, 0xc0, 0xf0, 0x1f, 0x00, 0x05, 0x00, 0x01, 0xff, 0x89, 0x99, 0x3d, 0x1d, 0x00, 0x00,
        0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ]
}

#[allow(dead_code)]
struct RecordingHandler {
    order: i32,
    events: Rc<RefCell<Vec<String>>>,
}

impl WriteHandler for RecordingHandler {
    fn order(&self) -> i32 {
        self.order
    }

    fn before_workbook(&mut self, context: &WriteWorkbookContext) -> Result<()> {
        self.events.borrow_mut().push(format!(
            "{}:before_workbook:{}",
            self.order,
            context.path().display()
        ));
        Ok(())
    }

    fn after_workbook(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        self.events
            .borrow_mut()
            .push(format!("{}:after_workbook", self.order));
        Ok(())
    }

    fn before_sheet(&mut self, context: &WriteSheetContext) -> Result<()> {
        self.events.borrow_mut().push(format!(
            "{}:before_sheet:{}",
            self.order,
            context.sheet_name()
        ));
        Ok(())
    }

    fn after_sheet(&mut self, _context: &WriteSheetContext) -> Result<()> {
        self.events
            .borrow_mut()
            .push(format!("{}:after_sheet", self.order));
        Ok(())
    }

    fn before_row(&mut self, context: &WriteRowContext) -> Result<()> {
        self.events
            .borrow_mut()
            .push(format!("{}:before_row:{}", self.order, context.is_head));
        Ok(())
    }

    fn after_row(&mut self, context: &WriteRowContext) -> Result<()> {
        self.events
            .borrow_mut()
            .push(format!("{}:after_row:{}", self.order, context.is_head));
        Ok(())
    }

    fn before_cell(&mut self, context: &mut WriteCellContext) -> Result<()> {
        self.events.borrow_mut().push(format!(
            "{}:before_cell:{}:{}",
            self.order, context.is_head, context.column_index
        ));
        if self.order < 0 {
            match (context.is_head, context.field) {
                (true, Some("empty")) | (false, Some("error")) => context.skip = true,
                (true, Some("string")) => context.value = CellValue::Bool(true),
                (true, Some("error")) => {
                    context.value = CellValue::Error("header-error".to_owned());
                }
                (false, Some("string")) => {
                    context.value = CellValue::String("transformed".to_owned());
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn after_cell(&mut self, context: &WriteCellContext) -> Result<()> {
        self.events.borrow_mut().push(format!(
            "{}:after_cell:{}:{}",
            self.order, context.is_head, context.skip
        ));
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum FailureStage {
    BeforeWorkbook,
    BeforeSheet,
    BeforeHeadRow,
    BeforeHeadCell,
    AfterHeadCell,
    AfterHeadRow,
    BeforeDataRow,
    BeforeDataCell,
    AfterDataCell,
    AfterDataRow,
    AfterSheet,
    AfterWorkbook,
}

#[allow(dead_code)]
struct FailingHandler(FailureStage);

#[allow(dead_code)]
struct InvalidHeaderValueHandler;

impl WriteHandler for InvalidHeaderValueHandler {
    fn before_cell(&mut self, context: &mut WriteCellContext) -> Result<()> {
        context.column_index = u16::MAX;
        context.value = CellValue::Bool(true);
        Ok(())
    }
}

impl FailingHandler {
    #[allow(dead_code)]
    fn result(&self, stage: FailureStage) -> Result<()> {
        if self.0 == stage {
            Err(ExcelError::Format("handler failed".to_owned()))
        } else {
            Ok(())
        }
    }
}

impl WriteHandler for FailingHandler {
    fn before_workbook(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        self.result(FailureStage::BeforeWorkbook)
    }

    fn after_workbook(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        self.result(FailureStage::AfterWorkbook)
    }

    fn before_sheet(&mut self, _context: &WriteSheetContext) -> Result<()> {
        self.result(FailureStage::BeforeSheet)
    }

    fn after_sheet(&mut self, _context: &WriteSheetContext) -> Result<()> {
        self.result(FailureStage::AfterSheet)
    }

    fn before_row(&mut self, context: &WriteRowContext) -> Result<()> {
        self.result(if context.is_head {
            FailureStage::BeforeHeadRow
        } else {
            FailureStage::BeforeDataRow
        })
    }

    fn after_row(&mut self, context: &WriteRowContext) -> Result<()> {
        self.result(if context.is_head {
            FailureStage::AfterHeadRow
        } else {
            FailureStage::AfterDataRow
        })
    }

    fn before_cell(&mut self, context: &mut WriteCellContext) -> Result<()> {
        self.result(if context.is_head {
            FailureStage::BeforeHeadCell
        } else {
            FailureStage::BeforeDataCell
        })
    }

    fn after_cell(&mut self, context: &WriteCellContext) -> Result<()> {
        self.result(if context.is_head {
            FailureStage::AfterHeadCell
        } else {
            FailureStage::AfterDataCell
        })
    }
}

