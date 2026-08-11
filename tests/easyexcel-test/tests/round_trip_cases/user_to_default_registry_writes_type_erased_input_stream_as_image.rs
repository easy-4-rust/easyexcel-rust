#[derive(Debug, Clone, PartialEq, ExcelRow)]
struct User {
    #[excel(name = "姓名", index = 0)]
    name: String,
    #[excel(name = "年龄", index = 1)]
    age: Option<u32>,
    #[excel(name = "注册日期", index = 2, format = "%Y-%m-%d")]
    registered_on: NaiveDate,
    #[excel(ignore)]
    transient: String,
}

fn test_user(name: &str, age: u32) -> User {
    User {
        name: name.to_owned(),
        age: Some(age),
        registered_on: NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid test date"),
        transient: String::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
struct ImageConverterRow {
    #[excel(name = "Primitive bytes", index = 0)]
    primitive_bytes: Vec<u8>,
    #[excel(name = "Boxed bytes", index = 1)]
    boxed_bytes: Box<[u8]>,
    #[excel(name = "Fixed bytes", index = 2)]
    fixed_bytes: [u8; 70],
    #[excel(name = "File", index = 3)]
    file: PathBuf,
    #[excel(name = "String file", index = 4, converter = easyexcel::StringImageConverter)]
    string_file: String,
}

#[derive(Debug, ExcelRow)]
struct StreamUrlImageRow {
    #[excel(name = "InputStream", index = 0, converter = InputStreamImageConverter)]
    stream: ImageInputStream<Cursor<Vec<u8>>>,
    #[excel(name = "URL", index = 1, converter = UrlImageConverter)]
    url: Url,
}

#[derive(Debug, ExcelRow)]
struct DefaultInputStreamImageRow {
    #[excel(name = "InputStream", index = 0)]
    stream: ImageInputStream,
}

#[derive(Debug, Clone, PartialEq, ExcelRow)]
struct MultiImageRow {
    #[excel(name = "Images", index = 0)]
    cell: WriteCellData,
}

#[derive(Debug, Clone, PartialEq, ExcelRow)]
struct RichTextFacadeRow {
    #[excel(name = "Rich", index = 0)]
    value: RichTextStringData,
}

#[derive(Default)]
struct NameConverter;

impl Converter<String> for NameConverter {
    fn convert_to_rust_data(&self, context: &ReadConverterContext<'_>) -> Result<String> {
        Ok(context
            .cell()
            .map_or_else(String::new, CellValue::as_text)
            .strip_prefix("excel:")
            .unwrap_or_default()
            .to_owned())
    }

    fn convert_to_excel_data(
        &self,
        context: &WriteConverterContext<'_, String>,
    ) -> Result<WriteCellData> {
        Ok(WriteCellData::from_string(format!(
            "excel:{}",
            context.value()
        )))
    }
}

#[derive(Default)]
struct FormulaConverter;

impl Converter<String> for FormulaConverter {
    fn convert_to_rust_data(&self, context: &ReadConverterContext<'_>) -> Result<String> {
        Ok(context
            .formula()
            .map_or_else(String::new, |formula| formula.formula_value().to_owned()))
    }

    fn convert_to_excel_data(
        &self,
        context: &WriteConverterContext<'_, String>,
    ) -> Result<WriteCellData> {
        Ok(WriteCellData::from_formula(context.value().clone()))
    }
}

#[derive(Default)]
struct RuntimeStyleConverter;

impl Converter<f64> for RuntimeStyleConverter {
    fn convert_to_excel_data(
        &self,
        context: &WriteConverterContext<'_, f64>,
    ) -> Result<WriteCellData> {
        let mut cell = WriteCellData::new(CellValue::Float(*context.value()));
        cell.set_write_cell_style(Some(ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Rgb(0x00_ff_00)),
            ..ExcelCellStyle::new()
        }.into()));
        cell.get_or_create_data_format()
            .set_format(Some("0.0000".to_owned()));
        Ok(cell)
    }
}

#[derive(Debug, Clone, PartialEq, ExcelRow)]
struct RuntimeStyledValue {
    #[excel(name = "Value", index = 0, converter = RuntimeStyleConverter)]
    value: f64,
}

#[derive(Debug, Clone, PartialEq, ExcelRow)]
struct RawStyledValue {
    #[excel(name = "Value", index = 0)]
    value: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
struct ConvertedName {
    #[excel(name = "姓名", index = 0, converter = NameConverter)]
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
struct RawName {
    #[excel(name = "姓名", index = 0)]
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
struct FormulaExpression {
    #[excel(name = "Formula", index = 0, converter = FormulaConverter)]
    formula: String,
}

#[derive(Debug, Clone, PartialEq, ExcelRow)]
struct CachedFormulaValue {
    #[excel(name = "Formula", index = 0)]
    value: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
struct LargeInteger {
    #[excel(name = "整数", index = 0)]
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
struct ArbitraryInteger {
    #[excel(name = "BigInteger", index = 0)]
    value: BigInt,
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
#[excel(column_width = 18, head_row_height = 24, content_row_height = 16)]
struct AnnotatedDimensions {
    #[excel(name = "姓名", index = 0, column_width = 30)]
    name: String,
    #[excel(name = "年龄", index = 1)]
    age: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, ExcelRow)]
#[excel(
    head_style(
        horizontal_alignment = "center",
        fill_pattern = "solid",
        fill_foreground_color = 0x00ff_0000,
        border_bottom = "thin"
    ),
    content_style(wrapped = true),
    head_font_style(font_name = "Arial", font_height_in_points = 14, bold = true),
    content_font_style(italic = true),
    once_absolute_merge(
        first_row_index = 0,
        last_row_index = 0,
        first_column_index = 0,
        last_column_index = 1
    )
)]
struct AnnotatedStyles {
    #[excel(
        name = "姓名",
        index = 0,
        head_style(fill_pattern = "solid", fill_foreground_color = 0x0000_00ff),
        head_font_style(font_height_in_points = 20),
        content_loop_merge(each_row = 2, column_extend = 1)
    )]
    name: String,
    #[excel(name = "年龄", index = 1)]
    age: u32,
}

struct EveryPublicCell;

impl ExcelRow for EveryPublicCell {
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
            ExcelColumn::new("formula", "Formula", Some(9), 0, None),
            ExcelColumn::new("link", "Link", Some(10), 0, None),
            ExcelColumn::new("comment", "Comment", Some(11), 0, None),
            ExcelColumn::new("image", "Image", Some(12), 0, None),
        ];
        COLUMNS
    }

    fn from_row(_row: &easyexcel::RowData) -> Result<Self> {
        Err(ExcelError::Unsupported("write-only test row".to_owned()))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        let date = NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid date");
        Ok(vec![
            CellValue::Empty,
            CellValue::String("text".to_owned()),
            CellValue::Error("#DIV/0!".to_owned()),
            CellValue::Bool(true),
            CellValue::Int(-12),
            CellValue::Float(1.25),
            CellValue::Date(date),
            CellValue::DateTime(date.and_hms_opt(12, 34, 56).expect("valid time")),
            CellValue::Int(i64::MAX),
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
        ])
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

fn serve_image_once(
    status: &str,
    body: Vec<u8>,
    declared_length: usize,
) -> Result<(Url, thread::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let status = status.to_owned();
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("accept image request");
        let mut request = [0_u8; 1024];
        let _ = socket.read(&mut request).expect("read image request");
        write!(
            socket,
            "HTTP/1.1 {status}\r\nContent-Length: {declared_length}\r\nConnection: close\r\n\r\n"
        )
        .expect("write image response head");
        socket.write_all(&body).expect("write image response body");
    });
    let url = Url::parse(&format!("http://{address}/logo.png"))
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    Ok((url, server))
}

#[test]
fn writes_and_reads_typed_rows_with_java_style_builders() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("users.xlsx");
    let users = vec![
        User {
            name: "张三".to_owned(),
            age: Some(30),
            registered_on: NaiveDate::from_ymd_opt(2026, 7, 17).expect("valid date"),
            transient: String::new(),
        },
        User {
            name: "李四".to_owned(),
            age: None,
            registered_on: NaiveDate::from_ymd_opt(2025, 1, 2).expect("valid date"),
            transient: String::new(),
        },
    ];

    EasyExcel::write::<User>(&path)
        .sheet("用户")
        .freeze_head(true)
        .constant_memory(true)
        .do_write_iter(users.clone())?;

    let actual = EasyExcel::read_sync::<User>(&path)
        .sheet("用户")
        .do_read_sync()?;
    assert_eq!(actual, users);
    Ok(())
}

#[test]
fn stateful_csv_finishes_a_real_multi_batch_public_workflow() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("users.csv");
    let sheet = EasyExcel::writer_sheet::<User>("用户");
    let mut writer = EasyExcel::write::<User>(&path).with_bom(false).build();
    writer
        .write([test_user("张三", 30)], &sheet)?
        .write([test_user("李四", 31)], &sheet)?;
    writer.finish()?;
    writer.finish()?;
    let mut empty_writer = EasyExcel::write::<User>(directory.path().join("empty.csv")).build();
    empty_writer.finish()?;

    let rows = EasyExcel::read_sync::<User>(&path).do_read_sync()?;
    assert_eq!(rows, [test_user("张三", 30), test_user("李四", 31)]);
    Ok(())
}

#[test]
fn integers_beyond_excels_exact_number_range_round_trip_as_text() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("large-integers.xlsx");
    let values = vec![
        LargeInteger { value: 42 },
        LargeInteger { value: i64::MAX },
        LargeInteger { value: i64::MIN },
        LargeInteger { value: 1 },
        LargeInteger { value: 2 },
        LargeInteger { value: 3 },
        LargeInteger { value: 4 },
    ];
    EasyExcel::write::<LargeInteger>(&path)
        .content_styles([
            CellStyle::new()
                .italic(true)
                .font_color(0x11_22_33)
                .background_color(0xEE_DD_CC)
                .horizontal_alignment(HorizontalAlignment::General)
                .vertical_alignment(VerticalAlignment::Top)
                .wrap_text(true)
                .number_format("0"),
            CellStyle::new()
                .horizontal_alignment(HorizontalAlignment::Left)
                .vertical_alignment(VerticalAlignment::Center)
                .bold(true),
            CellStyle::new()
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Bottom),
            CellStyle::new()
                .horizontal_alignment(HorizontalAlignment::Right)
                .vertical_alignment(VerticalAlignment::Justify),
            CellStyle::new()
                .horizontal_alignment(HorizontalAlignment::Fill)
                .vertical_alignment(VerticalAlignment::Distributed),
            CellStyle::new().horizontal_alignment(HorizontalAlignment::Justify),
            CellStyle::new().horizontal_alignment(HorizontalAlignment::CenterAcross),
        ])
        .do_write(values.clone())?;
    assert_eq!(
        EasyExcel::read_sync::<LargeInteger>(&path).do_read_sync()?,
        values
    );
    Ok(())
}

#[test]
fn java_big_integer_fields_round_trip_without_precision_loss() -> Result<()> {
    let directory = tempdir()?;
    let path = directory.path().join("big-integers.xlsx");
    let values = vec![
        ArbitraryInteger {
            value: BigInt::from(42),
        },
        ArbitraryInteger {
            value: "1234567890123456789012345678901234567890"
                .parse()
                .expect("valid big integer"),
        },
        ArbitraryInteger {
            value: "-987654321098765432109876543210987654321"
                .parse()
                .expect("valid big integer"),
        },
    ];

    EasyExcel::write::<ArbitraryInteger>(&path).do_write(values.clone())?;
    assert_eq!(
        EasyExcel::read_sync::<ArbitraryInteger>(&path).do_read_sync()?,
        values
    );
    Ok(())
}

#[test]
fn public_writer_accepts_every_supported_cell_variant() -> Result<()> {
    let directory = tempdir()?;
    EasyExcel::write::<EveryPublicCell>(directory.path().join("every-cell.xlsx"))
        .do_write([EveryPublicCell])?;
    Ok(())
}

#[test]
fn derive_uses_java_style_byte_array_and_file_image_converters() -> Result<()> {
    let directory = tempdir()?;
    let image_path = directory.path().join("source.png");
    let bytes = tiny_png();
    std::fs::write(&image_path, &bytes)?;
    let fixed_bytes: [u8; 70] = bytes.clone().try_into().expect("70-byte PNG fixture");
    let workbook_path = directory.path().join("image-converters.xlsx");

    EasyExcel::write::<ImageConverterRow>(&workbook_path).do_write([ImageConverterRow {
        primitive_bytes: bytes.clone(),
        boxed_bytes: bytes.clone().into_boxed_slice(),
        fixed_bytes,
        file: image_path.clone(),
        string_file: image_path.to_string_lossy().into_owned(),
    }])?;

    let mut archive = ZipArchive::new(File::open(&workbook_path)?)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let media_entries = (0..archive.len())
        .map(|index| {
            archive
                .by_index(index)
                .map(|entry| entry.name().to_owned())
                .map_err(|error| ExcelError::Format(error.to_string()))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|name| name.starts_with("xl/media/"))
        .count();
    assert_eq!(media_entries, 1);
    let mut drawing_xml = String::new();
    archive
        .by_name("xl/drawings/drawing1.xml")
        .map_err(|error| ExcelError::Format(error.to_string()))?
        .read_to_string(&mut drawing_xml)?;
    assert_eq!(drawing_xml.matches("<xdr:twoCellAnchor").count(), 5);
    Ok(())
}

#[test]
fn derive_uses_java_style_input_stream_and_url_image_converters() -> Result<()> {
    let bytes = tiny_png();
    let probe_stream = ImageInputStream::from(Cursor::new(bytes.clone()));
    assert_eq!(probe_stream.into_inner().into_inner(), bytes);
    let defaults = UrlImageConverter::default();
    assert_eq!(
        defaults.connect_timeout(),
        UrlImageConverter::DEFAULT_CONNECT_TIMEOUT
    );
    assert_eq!(
        defaults.read_timeout(),
        UrlImageConverter::DEFAULT_READ_TIMEOUT
    );
    let (url, server) = serve_image_once("200 OK", bytes.clone(), bytes.len())?;
    let directory = tempdir()?;
    let workbook_path = directory.path().join("stream-url-images.xlsx");

    EasyExcel::write::<StreamUrlImageRow>(&workbook_path).do_write([StreamUrlImageRow {
        stream: ImageInputStream::new(Cursor::new(bytes.clone())),
        url,
    }])?;
    server.join().expect("image server joins");

    let conversion = easyexcel::ConvertContext {
        sheet_name: "Images".to_owned(),
        row_index: 1,
        column_index: Some(1),
        field: "url",
        format: None,
        date_time_format: None,
        number_format: None,
        use_1904_windowing: false,
    };
    let (url, server) = serve_image_once("404 Not Found", Vec::new(), 0)?;
    assert!(url.to_excel_cell(&conversion).is_err());
    server.join().expect("image server joins");
    let (url, server) = serve_image_once("200 OK", bytes.clone(), bytes.len() + 1)?;
    assert!(url.to_excel_cell(&conversion).is_err());
    server.join().expect("image server joins");
    let column = ExcelColumn::new("stream", "InputStream", Some(0), 0, None);
    let read_context = ReadConverterContext::new(None, &column, &conversion);
    assert!(
        Converter::<ImageInputStream<Cursor<Vec<u8>>>>::convert_to_rust_data(
            &InputStreamImageConverter,
            &read_context,
        )
        .is_err()
    );

    let mut archive = ZipArchive::new(File::open(&workbook_path)?)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut drawing_xml = String::new();
    archive
        .by_name("xl/drawings/drawing1.xml")
        .map_err(|error| ExcelError::Format(error.to_string()))?
        .read_to_string(&mut drawing_xml)?;
    assert_eq!(drawing_xml.matches("<xdr:twoCellAnchor").count(), 2);
    Ok(())
}

#[test]
fn default_registry_writes_type_erased_input_stream_as_image() -> Result<()> {
    let bytes = tiny_png();
    let directory = tempdir()?;
    let workbook_path = directory.path().join("default-input-stream-image.xlsx");

    EasyExcel::write::<DefaultInputStreamImageRow>(&workbook_path).do_write([
        DefaultInputStreamImageRow {
            stream: ImageInputStream::boxed(Cursor::new(bytes.clone())),
        },
    ])?;

    let mut archive = ZipArchive::new(File::open(&workbook_path)?)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let media_name = (0..archive.len())
        .find_map(|index| {
            let entry = archive.by_index(index).ok()?;
            entry
                .name()
                .starts_with("xl/media/")
                .then(|| entry.name().to_owned())
        })
        .expect("default InputStream converter writes an XLSX media part");
    let mut embedded = Vec::new();
    archive
        .by_name(&media_name)
        .map_err(|error| ExcelError::Format(error.to_string()))?
        .read_to_end(&mut embedded)?;
    assert_eq!(embedded, bytes);

    let mut drawing_xml = String::new();
    archive
        .by_name("xl/drawings/drawing1.xml")
        .map_err(|error| ExcelError::Format(error.to_string()))?
        .read_to_string(&mut drawing_xml)?;
    assert_eq!(drawing_xml.matches("<xdr:twoCellAnchor").count(), 1);
    Ok(())
}

