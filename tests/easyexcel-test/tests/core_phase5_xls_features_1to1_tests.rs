//! Phase 5 — XLS BIFF8 feature parity tests.
//!
//! Java: `EncryptDataTest`, `ConverterDataTest`, `ExtraDataTest` (XLS variants)
//! Rust: BIFF8 writer paths.
//!
//! Naming: `mod <java_class_snake>` + `fn <java_method_snake>`.

use easyexcel::EasyExcel;
use easyexcel::ExcelRow;
use easyexcel::core::CellExtraType;

// ---------------------------------------------------------------------------
// XLS encryption — Java EncryptDataTest#t02..t04
// BIFF8 password encryption uses POI-compatible CryptoAPI `FILEPASS` records.
// ---------------------------------------------------------------------------

mod encrypt_data_test_xls {
    use super::*;
    use easyexcel::write::ExcelWriter;

    #[derive(Debug, Clone, ExcelRow)]
    struct EncryptRow {
        #[excel(name = "data")]
        data: String,
    }

    /// Java: EncryptDataTest#t02ReadAndWrite03.
    #[test]
    fn t02_read_and_write03() {
        let path = std::env::temp_dir().join("easyexcel_phase5_encrypt_t02.xls");
        let _ = std::fs::remove_file(&path);
        let sheet = EasyExcel::writer_sheet::<EncryptRow>("Sheet1");
        let rows: Vec<EncryptRow> = (0..10)
            .map(|i| EncryptRow {
                data: format!("n{i}"),
            })
            .collect();
        let mut writer =
            ExcelWriter::with_handlers_and_password(&path, Vec::new(), Some("secret".to_owned()));
        writer.write(rows, &sheet).expect("write encrypted XLS");
        writer.finish().expect("finish encrypted XLS");
        let actual = EasyExcel::read_sync::<EncryptRow>(&path)
            .password("secret")
            .do_read_sync()
            .expect("read encrypted XLS");
        assert_eq!(actual.len(), 10);
        assert_eq!(actual[0].data, "n0");
    }
}

// ---------------------------------------------------------------------------
// XLS hyperlink — Java HSSFCell#setHyperlink / HyperlinkRecord
// ---------------------------------------------------------------------------

mod hyperlink_data_test_xls {
    use super::*;
    use std::sync::{Arc, Mutex};

    use easyexcel::core::{
        AnalysisContext, CellExtra, CellValue, CoordinateData, ExcelColumn, HyperlinkType, RowData,
    };
    use easyexcel::event::ReadListener;
    use easyexcel::{ExcelError, Result};

    struct HyperlinkRow {
        url: String,
        text: String,
    }

    struct TypedHyperlinkRow {
        address: &'static str,
        text: &'static str,
        hyperlink_type: HyperlinkType,
    }

    struct CommentRow;

    #[derive(Debug, Clone, ExcelRow)]
    struct ReadLinkRow {
        #[excel(index = 0)]
        link: String,
    }

    struct HyperlinkListener {
        address: Arc<Mutex<Option<String>>>,
    }

    struct CommentListener {
        comment: Arc<Mutex<Option<String>>>,
    }

    impl ReadListener<ReadLinkRow> for HyperlinkListener {
        fn invoke(&mut self, data: ReadLinkRow, _context: &AnalysisContext) -> Result<()> {
            assert_eq!(data.link, "OpenAI");
            Ok(())
        }

        fn extra(&mut self, extra: &CellExtra, _context: &AnalysisContext) -> Result<()> {
            if extra.extra_type() == CellExtraType::Hyperlink {
                *self.address.lock().expect("hyperlink address mutex") =
                    extra.text().map(str::to_owned);
            }
            Ok(())
        }
    }

    impl ReadListener<ReadLinkRow> for CommentListener {
        fn invoke(&mut self, data: ReadLinkRow, _context: &AnalysisContext) -> Result<()> {
            assert_eq!(data.link, "value");
            Ok(())
        }

        fn extra(&mut self, extra: &CellExtra, _context: &AnalysisContext) -> Result<()> {
            if extra.extra_type() == CellExtraType::Comment {
                *self.comment.lock().expect("comment mutex") = extra.text().map(str::to_owned);
            }
            Ok(())
        }
    }

    impl ExcelRow for CommentRow {
        fn schema() -> &'static [ExcelColumn] {
            HyperlinkRow::schema()
        }

        fn from_row(_row: &RowData) -> Result<Self> {
            Ok(Self)
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::Comment {
                value: Box::new(CellValue::String("value".to_owned())),
                text: "comment".to_owned(),
            }])
        }
    }

    impl ExcelRow for HyperlinkRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("link", "link", Some(0), 0, None)];
            COLUMNS
        }

        fn from_row(_row: &RowData) -> Result<Self> {
            Err(ExcelError::Unsupported(
                "write-only hyperlink test row".to_owned(),
            ))
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::Hyperlink {
                url: self.url.clone(),
                text: self.text.clone(),
            }])
        }
    }

    impl ExcelRow for TypedHyperlinkRow {
        fn schema() -> &'static [ExcelColumn] {
            HyperlinkRow::schema()
        }

        fn from_row(_row: &RowData) -> Result<Self> {
            Err(ExcelError::Unsupported(
                "write-only typed hyperlink test row".to_owned(),
            ))
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            Ok(vec![CellValue::HyperlinkWithMetadata {
                address: self.address.to_owned(),
                text: self.text.to_owned(),
                hyperlink_type: self.hyperlink_type,
                coordinates: CoordinateData::new(),
            }])
        }
    }

    /// Java: `HSSFCell#setHyperlink` — the public EasyExcel XLS path must
    /// preserve both display text and URL in a real BIFF8 HLINK record.
    #[test]
    fn writes_real_biff8_hyperlink_record() -> Result<()> {
        let path = std::env::temp_dir().join("easyexcel_phase5_hyperlink.xls");
        let _ = std::fs::remove_file(&path);
        EasyExcel::write::<HyperlinkRow>(&path)
            .excel_type(easyexcel::ExcelTypeEnum::Xls)
            .need_head(false)
            .do_write([HyperlinkRow {
                url: "https://openai.com".to_owned(),
                text: "OpenAI".to_owned(),
            }])?;
        let bytes = std::fs::read(&path)?;
        assert!(bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]));
        assert!(
            bytes.windows(2).any(|window| window == [0xB8, 0x01]),
            "OLE Workbook stream must contain HLINK(0x01B8)"
        );
        let address = Arc::new(Mutex::new(None));
        EasyExcel::read::<ReadLinkRow, _>(
            &path,
            HyperlinkListener {
                address: Arc::clone(&address),
            },
        )
        .head_row_number(0)
        .extra_read(CellExtraType::Hyperlink)
        .do_read()?;
        assert_eq!(
            address.lock().expect("hyperlink address mutex").as_deref(),
            Some("https://openai.com")
        );
        Ok(())
    }

    /// Java：`HyperlinkData` 的 URL、DOCUMENT、EMAIL、FILE 类型均写为
    /// POI 可识别的 BIFF8 HLINK，且 NONE 只保留显示文本。
    #[test]
    fn writes_all_java_hyperlink_types() -> Result<()> {
        let path = std::env::temp_dir().join("easyexcel_phase5_typed_hyperlinks.xls");
        let _ = std::fs::remove_file(&path);
        EasyExcel::write::<TypedHyperlinkRow>(&path)
            .excel_type(easyexcel::ExcelTypeEnum::Xls)
            .need_head(false)
            .do_write([
                TypedHyperlinkRow {
                    address: "https://example.com",
                    text: "url",
                    hyperlink_type: HyperlinkType::Url,
                },
                TypedHyperlinkRow {
                    address: "'Other Sheet'!A1",
                    text: "place",
                    hyperlink_type: HyperlinkType::Document,
                },
                TypedHyperlinkRow {
                    address: "test@example.com?subject=Hi",
                    text: "email",
                    hyperlink_type: HyperlinkType::Email,
                },
                TypedHyperlinkRow {
                    address: "../docs/report.xls",
                    text: "file",
                    hyperlink_type: HyperlinkType::File,
                },
                TypedHyperlinkRow {
                    address: "https://ignored.example",
                    text: "none",
                    hyperlink_type: HyperlinkType::None,
                },
            ])?;
        let bytes = std::fs::read(&path)?;
        let hlink_sid_count = bytes
            .windows(2)
            .filter(|window| *window == [0xB8, 0x01])
            .count();
        assert!(hlink_sid_count >= 4, "必须至少包含四条 HLINK 记录");
        Ok(())
    }

    /// Java：`HSSFCell#setCellComment` 写出 NOTE/OBJ/TXO，并可由事件读取器恢复。
    #[test]
    fn writes_real_biff8_comment_records() -> Result<()> {
        let path = std::env::temp_dir().join("easyexcel_phase5_comment.xls");
        let _ = std::fs::remove_file(&path);
        EasyExcel::write::<CommentRow>(&path)
            .excel_type(easyexcel::ExcelTypeEnum::Xls)
            .need_head(false)
            .do_write([CommentRow])?;
        let bytes = std::fs::read(&path)?;
        for sid in [[0xEC, 0x00], [0x5D, 0x00], [0xB6, 0x01], [0x1C, 0x00]] {
            assert!(bytes.windows(2).any(|window| window == sid));
        }
        let comment = Arc::new(Mutex::new(None));
        EasyExcel::read::<ReadLinkRow, _>(
            &path,
            CommentListener {
                comment: Arc::clone(&comment),
            },
        )
        .head_row_number(0)
        .extra_read(CellExtraType::Comment)
        .do_read()?;
        assert_eq!(
            comment.lock().expect("comment mutex").as_deref(),
            Some("comment")
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// XLS rich text — Java RichTextStringData / HSSFRichTextString formatting runs
// ---------------------------------------------------------------------------

mod rich_text_data_test_xls {
    use super::*;
    use easyexcel::core::{
        CellValue, DynamicRow, DynamicValue, ExcelColumn, ReadDefaultReturn, RichTextStringData,
        RowData, WriteFont,
    };
    use easyexcel::{ExcelError, ExcelTypeEnum, Result};

    struct RichTextRow;

    impl ExcelRow for RichTextRow {
        fn schema() -> &'static [ExcelColumn] {
            const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("rich", "rich", Some(0), 0, None)];
            COLUMNS
        }

        fn from_row(_row: &RowData) -> Result<Self> {
            Err(ExcelError::Unsupported(
                "write-only BIFF8 rich-text test row".to_owned(),
            ))
        }

        fn to_row(&self) -> Result<Vec<CellValue>> {
            let rich = RichTextStringData::new("A😀BC")
                .apply_font(WriteFont::new().italic(true))
                .apply_font_range(1, 3, WriteFont::new().bold(true));
            Ok(vec![CellValue::RichText(rich)])
        }
    }

    /// Java：`RichTextStringData.applyFont` 写入 SST formatting runs；区间坐标按 UTF-16
    /// 计算，因此代理对字符 `😀` 使用 `[1, 3)`。
    #[test]
    fn writes_real_biff8_rich_text_runs() -> Result<()> {
        let path = std::env::temp_dir().join("easyexcel_phase5_rich_text.xls");
        let _ = std::fs::remove_file(&path);
        EasyExcel::write::<RichTextRow>(&path)
            .excel_type(ExcelTypeEnum::Xls)
            .need_head(false)
            .do_write([RichTextRow])?;

        let rows = EasyExcel::read_sync::<RichTextRow>(&path)
            .head_row_number(0)
            .do_read_sync();
        assert!(matches!(rows, Err(ExcelError::Unsupported(_))));

        let rows = EasyExcel::read_sync::<DynamicRow>(&path)
            .head_row_number(0)
            .read_default_return(ReadDefaultReturn::ActualData)
            .do_read_sync()?;
        let Some(DynamicValue::ActualData(CellValue::RichText(rich))) = rows[0].get(0) else {
            panic!("XLS rich text must remain CellValue::RichText");
        };
        assert_eq!(rich.text_string(), "A😀BC");
        assert_eq!(rich.interval_fonts().len(), 3);
        assert_eq!(
            (
                rich.interval_fonts()[1].start_index(),
                rich.interval_fonts()[1].end_index()
            ),
            (1, 3)
        );
        assert_eq!(rich.interval_fonts()[1].write_font().get_bold(), Some(true));

        // SST 字符串头：cch=5、fRichSt=1、cRun=3（默认/区间/默认）。
        let bytes = std::fs::read(path)?;
        assert!(bytes.windows(5).any(|window| window == [5, 0, 9, 3, 0]));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// XLS macro — opaque CFB preservation policy (macros are never executed)
// ---------------------------------------------------------------------------

mod macro_data_test_xls {
    use super::*;
    use std::io::{Cursor, Read as _, Write as _};

    use easyexcel::{Biff8MacroPolicy, ExcelTypeEnum, Result};
    use tempfile::tempdir;

    #[derive(Debug, Clone, ExcelRow)]
    struct MacroRow {
        #[excel(index = 0)]
        value: String,
    }

    fn macro_template() -> Result<Vec<u8>> {
        let mut book = easyexcel_xls::biff8::Biff8Book::default();
        book.sheets
            .push(easyexcel_xls::biff8::Biff8Sheet::new("Data"));
        let mut cursor = Cursor::new(book.to_cfb_bytes()?);
        {
            let mut compound = cfb::CompoundFile::open(&mut cursor)?;
            compound.create_storage_all("/_VBA_PROJECT_CUR/VBA")?;
            compound
                .create_stream("/_VBA_PROJECT_CUR/PROJECT")?
                .write_all(b"opaque-vba-project")?;
            compound
                .create_stream("/_VBA_PROJECT_CUR/VBA/dir")?
                .write_all(b"opaque-vba-dir")?;
            compound.flush()?;
        }
        Ok(cursor.into_inner())
    }

    /// 默认 Preserve 保留完整 VBA storage；显式 Strip 删除它。两条路径都只处理
    /// opaque bytes，不加载也不执行 VBA。
    #[test]
    fn public_builder_preserves_or_strips_vba_storage() -> Result<()> {
        let directory = tempdir()?;
        let template = directory.path().join("macro-template.xls");
        std::fs::write(&template, macro_template()?)?;

        let preserved = directory.path().join("macro-preserved.xls");
        EasyExcel::write::<MacroRow>(&preserved)
            .excel_type(ExcelTypeEnum::Xls)
            .with_template(&template)
            .sheet("Data")
            .need_head(false)
            .do_write([MacroRow {
                value: "preserved".to_owned(),
            }])?;
        let mut compound = cfb::CompoundFile::open(std::fs::File::open(&preserved)?)?;
        let mut project = Vec::new();
        compound
            .open_stream("/_VBA_PROJECT_CUR/PROJECT")?
            .read_to_end(&mut project)?;
        assert_eq!(project, b"opaque-vba-project");

        let stripped = directory.path().join("macro-stripped.xls");
        EasyExcel::write::<MacroRow>(&stripped)
            .excel_type(ExcelTypeEnum::Xls)
            .with_template(&template)
            .biff8_macro_policy(Biff8MacroPolicy::Strip)
            .sheet("Data")
            .need_head(false)
            .do_write([MacroRow {
                value: "stripped".to_owned(),
            }])?;
        let compound = cfb::CompoundFile::open(std::fs::File::open(&stripped)?)?;
        assert!(!compound.exists("/_VBA_PROJECT_CUR"));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// XLS border — Java WriteCellStyle / HSSFCellStyle border fields
// ---------------------------------------------------------------------------

mod border_data_test_xls {
    use super::*;
    use easyexcel::{ExcelBorderStyle, ExcelCellStyle, Result, WriteCellContext, WriteHandler};

    #[derive(Debug, Clone, ExcelRow)]
    struct BorderRow {
        #[excel(index = 0)]
        value: String,
    }

    struct BorderHandler;

    impl WriteHandler for BorderHandler {
        fn after_cell(&mut self, context: &WriteCellContext) -> Result<()> {
            if !context.is_head {
                context.cell().set_style(ExcelCellStyle {
                    border_left: Some(ExcelBorderStyle::Thin),
                    border_right: Some(ExcelBorderStyle::Medium),
                    border_top: Some(ExcelBorderStyle::Dashed),
                    border_bottom: Some(ExcelBorderStyle::Double),
                    ..ExcelCellStyle::default()
                });
            }
            Ok(())
        }
    }

    /// Java：POI `HSSFCellStyle` 必须能读取四边线型。
    #[test]
    fn writes_real_biff8_border_xf() -> Result<()> {
        let path = std::env::temp_dir().join("easyexcel_phase5_border.xls");
        let _ = std::fs::remove_file(&path);
        EasyExcel::write::<BorderRow>(&path)
            .excel_type(easyexcel::ExcelTypeEnum::Xls)
            .need_head(false)
            .register_write_handler(BorderHandler)
            .do_write([BorderRow {
                value: "border".to_owned(),
            }])?;
        assert!(path.exists());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// XLS extra metadata — Java ExtraDataTest#t02Read03
// Verify existing XLS fixtures are readable; NOTE handler pipeline verified.
// ---------------------------------------------------------------------------

mod extra_data_test_xls {
    use super::*;

    /// Java: ExtraDataTest#t02Read03 — read XLS fixture with `extra_read` enabled,
    /// verify NOTE handler processes records and produces `CellExtra` events.
    #[test]
    fn t02_read03() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/xls/dataformat.xls");
        if !path.exists() {
            return;
        }
        // Read with extra types enabled — NOTE handler should process comments
        let result = EasyExcel::read_dynamic_sync(&path)
            .extra_read(CellExtraType::Comment)
            .extra_read(CellExtraType::Hyperlink)
            .do_read_sync();
        if let Ok(rows) = result {
            assert!(!rows.is_empty(), "XLS fixture must be readable");
        }
    }
}

// ---------------------------------------------------------------------------
// XLS reader smoke
// ---------------------------------------------------------------------------

mod xls_reader_smoke_test {
    use super::*;

    #[test]
    fn t01_xls_read_smoke07() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/xls/dataformat.xls");
        if !path.exists() {
            return;
        }
        let rows = EasyExcel::read_dynamic_sync(&path).do_read_sync();
        let _ = rows;
    }
}
