//! SXSSF `GZIPSheetDataWriter` equivalent: gzip-compressed row spill on disk.
//!
//! Java `SXSSFWorkbook.setCompressTempFiles(true)` routes sheet XML through
//! `GZIPSheetDataWriter`. `rust_xlsxwriter` constant-memory mode cannot gzip its
//! internal tempfile, so this module owns the durable spill while rows are
//! written; [`ExcelWriter`] materializes into a constant-memory worksheet only
//! at `finish` (stream decode → write → ZIP), keeping peak RAM bounded.

use std::path::Path;

use crate::core::{
    CellValue, CoordinateData, ExcelError, HyperlinkType, ImageData, Result, RichTextStringData,
};
#[cfg(test)]
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use easyexcel_io::{
    GzipCellSpillReader as EngineSpillReader, GzipCellSpillWriter as EngineSpillWriter,
    GzipCellValue,
};

pub use easyexcel_io::io::gzip_record::{GZIP_MAGIC, file_has_gzip_magic};

include!("gzip_spill/gzip_spill_snapshot.rs");

mod journal_cell_style;
pub(crate) use journal_cell_style::JournalCellStyle;

mod journal_cell;
pub(crate) use journal_cell::JournalCell;

mod journal_row;
pub(crate) use journal_row::JournalRow;

include!("gzip_spill/gzip_sheet_data_writer.rs");

include!("gzip_spill/gzip_spill_reader.rs");

fn to_spill_value(value: &CellValue) -> Result<GzipCellValue> {
    Ok(match value {
        CellValue::Empty => GzipCellValue::Empty,
        CellValue::String(text) => GzipCellValue::Text(text.clone()),
        CellValue::Bool(flag) => GzipCellValue::Bool(*flag),
        CellValue::Int(number) => GzipCellValue::Int(*number),
        CellValue::Float(number) => GzipCellValue::Float(*number),
        CellValue::Decimal(number) => GzipCellValue::Decimal(number.to_string()),
        CellValue::Date(date) => GzipCellValue::Date(date.format("%Y-%m-%d").to_string()),
        CellValue::DateTime(date_time) => {
            GzipCellValue::DateTime(date_time.format("%Y-%m-%d %H:%M:%S%.f").to_string())
        }
        CellValue::Error(text) => GzipCellValue::Error(text.clone()),
        CellValue::Formula(text) => GzipCellValue::Formula(text.clone()),
        CellValue::Hyperlink { url, text } => GzipCellValue::Hyperlink {
            url: url.clone(),
            text: text.clone(),
        },
        CellValue::HyperlinkWithMetadata {
            address,
            text,
            hyperlink_type,
            coordinates,
        } => GzipCellValue::TypedHyperlink {
            address: address.clone(),
            text: text.clone(),
            kind: hyperlink_type_to_spill(*hyperlink_type),
            first_row: coordinates.get_first_row_index(),
            first_col: coordinates.get_first_column_index(),
            last_row: coordinates.get_last_row_index(),
            last_col: coordinates.get_last_column_index(),
            relative_first_row: coordinates.get_relative_first_row_index(),
            relative_first_col: coordinates.get_relative_first_column_index(),
            relative_last_row: coordinates.get_relative_last_row_index(),
            relative_last_col: coordinates.get_relative_last_column_index(),
        },
        CellValue::Comment { value, text } => GzipCellValue::Comment {
            value: Box::new(to_spill_value(value)?),
            text: text.clone(),
        },
        CellValue::Image(bytes) => GzipCellValue::Image(bytes.clone()),
        CellValue::RichText(rich) => GzipCellValue::RichText(rich.text_string().to_owned()),
        CellValue::Images { value, images } => GzipCellValue::Images {
            value: Box::new(to_spill_value(value)?),
            images: images.iter().map(|image| image.image().to_vec()).collect(),
        },
    })
}

fn from_spill_value(value: GzipCellValue) -> Result<CellValue> {
    Ok(match value {
        GzipCellValue::Empty => CellValue::Empty,
        GzipCellValue::Text(text) => CellValue::String(text),
        GzipCellValue::Bool(flag) => CellValue::Bool(flag),
        GzipCellValue::Int(number) => CellValue::Int(number),
        GzipCellValue::Float(number) => CellValue::Float(number),
        GzipCellValue::Decimal(text) => CellValue::Decimal(
            text.parse()
                .map_err(|error| ExcelError::Format(format!("invalid decimal spill: {error}")))?,
        ),
        GzipCellValue::Date(text) => CellValue::Date(
            NaiveDate::parse_from_str(&text, "%Y-%m-%d")
                .map_err(|error| ExcelError::Format(format!("invalid date spill: {error}")))?,
        ),
        GzipCellValue::DateTime(text) => CellValue::DateTime(
            NaiveDateTime::parse_from_str(&text, "%Y-%m-%d %H:%M:%S%.f")
                .or_else(|_| NaiveDateTime::parse_from_str(&text, "%Y-%m-%d %H:%M:%S"))
                .map_err(|error| ExcelError::Format(format!("invalid datetime spill: {error}")))?,
        ),
        GzipCellValue::Error(text) => CellValue::Error(text),
        GzipCellValue::Formula(text) => CellValue::Formula(text),
        GzipCellValue::Hyperlink { url, text } => CellValue::Hyperlink { url, text },
        GzipCellValue::TypedHyperlink {
            address,
            text,
            kind,
            first_row,
            first_col,
            last_row,
            last_col,
            relative_first_row,
            relative_first_col,
            relative_last_row,
            relative_last_col,
        } => CellValue::HyperlinkWithMetadata {
            address,
            text,
            hyperlink_type: hyperlink_type_from_spill(kind)?,
            coordinates: coordinate_data_from_spill(
                first_row,
                first_col,
                last_row,
                last_col,
                relative_first_row,
                relative_first_col,
                relative_last_row,
                relative_last_col,
            ),
        },
        GzipCellValue::Comment { value, text } => CellValue::Comment {
            value: Box::new(from_spill_value(*value)?),
            text,
        },
        GzipCellValue::Image(bytes) => CellValue::Image(bytes),
        GzipCellValue::RichText(text) => CellValue::RichText(RichTextStringData::new(text)),
        GzipCellValue::Images { value, images } => CellValue::Images {
            value: Box::new(from_spill_value(*value)?),
            images: images.into_iter().map(ImageData::new).collect(),
        },
        GzipCellValue::Styled { .. } | GzipCellValue::JournalMetadata { .. } => {
            return Err(ExcelError::Format(
                "stateful journal metadata used as a scalar spill value".to_owned(),
            ));
        }
    })
}

const fn hyperlink_type_to_spill(value: HyperlinkType) -> u8 {
    match value {
        HyperlinkType::None => 0,
        HyperlinkType::Url => 1,
        HyperlinkType::Document => 2,
        HyperlinkType::Email => 3,
        HyperlinkType::File => 4,
    }
}

fn hyperlink_type_from_spill(value: u8) -> Result<HyperlinkType> {
    match value {
        0 => Ok(HyperlinkType::None),
        1 => Ok(HyperlinkType::Url),
        2 => Ok(HyperlinkType::Document),
        3 => Ok(HyperlinkType::Email),
        4 => Ok(HyperlinkType::File),
        other => Err(ExcelError::Format(format!(
            "invalid hyperlink type in gzip spill: {other}"
        ))),
    }
}

#[allow(clippy::too_many_arguments)]
const fn coordinate_data_from_spill(
    first_row: Option<u32>,
    first_col: Option<u16>,
    last_row: Option<u32>,
    last_col: Option<u16>,
    relative_first_row: Option<i32>,
    relative_first_col: Option<i32>,
    relative_last_row: Option<i32>,
    relative_last_col: Option<i32>,
) -> CoordinateData {
    let mut value = CoordinateData::new();
    if let Some(index) = first_row {
        value = value.first_row_index(index);
    }
    if let Some(index) = first_col {
        value = value.first_column_index(index);
    }
    if let Some(index) = last_row {
        value = value.last_row_index(index);
    }
    if let Some(index) = last_col {
        value = value.last_column_index(index);
    }
    if let Some(index) = relative_first_row {
        value = value.relative_first_row_index(index);
    }
    if let Some(index) = relative_first_col {
        value = value.relative_first_column_index(index);
    }
    if let Some(index) = relative_last_row {
        value = value.relative_last_row_index(index);
    }
    if let Some(index) = relative_last_col {
        value = value.relative_last_column_index(index);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn gzip_spill_round_trips_cells_and_exposes_magic() {
        let mut writer = GzipSheetDataWriter::create_owned("Sheet1").expect("create");
        let date = NaiveDate::from_ymd_opt(2020, 1, 1).expect("date");
        writer
            .write_row(&[
                CellValue::String("字符串0".to_owned()),
                CellValue::Date(date),
                CellValue::Float(0.56),
                CellValue::Int(42),
                CellValue::Bool(true),
                CellValue::Empty,
            ])
            .expect("write");
        let snap = writer.snapshot().expect("snapshot");
        assert!(snap.is_gzip, "spill must start with gzip magic");
        assert!(snap.uncompressed_len > 0);
        assert!(snap.compressed_len > 0);
        // Highly repetitive / small payloads may not shrink, but magic must be present.
        assert_eq!(&snap.path.extension().and_then(|e| e.to_str()), &Some("gz"));

        let mut reader = writer.finish().expect("finish");
        let row = reader.next_row().expect("decode").expect("one row");
        assert_eq!(row[0], CellValue::String("字符串0".to_owned()));
        assert_eq!(row[1], CellValue::Date(date));
        assert!(matches!(row[2], CellValue::Float(v) if (v - 0.56).abs() < f64::EPSILON));
        assert_eq!(row[3], CellValue::Int(42));
        assert_eq!(row[4], CellValue::Bool(true));
        assert_eq!(row[5], CellValue::Empty);
        assert!(reader.next_row().expect("eof").is_none());
        assert!(reader.snapshot().is_gzip);
    }

    #[test]
    fn gzip_spill_round_trips_every_cell_value_variant() {
        let mut writer = GzipSheetDataWriter::create_owned("All").expect("create");
        let date = NaiveDate::from_ymd_opt(2020, 1, 1).expect("date");
        let datetime = date.and_hms_opt(12, 30, 0).expect("datetime");
        let datetime_nanos = date
            .and_hms_nano_opt(12, 30, 0, 123_000_000)
            .expect("datetime nanos");
        let rows = vec![
            CellValue::Empty,
            CellValue::String("s".to_owned()),
            CellValue::Bool(false),
            CellValue::Int(-7),
            CellValue::Float(1.25),
            CellValue::Decimal(BigDecimal::from(3)),
            CellValue::Date(date),
            CellValue::DateTime(datetime),
            CellValue::DateTime(datetime_nanos),
            CellValue::Error("#DIV/0!".to_owned()),
            CellValue::Formula("B2+C2".to_owned()),
            CellValue::Hyperlink {
                url: "https://x".to_owned(),
                text: "link".to_owned(),
            },
            CellValue::HyperlinkWithMetadata {
                address: "'Other Sheet'!A1".to_owned(),
                text: "place".to_owned(),
                hyperlink_type: HyperlinkType::Document,
                coordinates: CoordinateData::new()
                    .first_row_index(4)
                    .relative_last_column_index(2),
            },
            CellValue::Comment {
                value: Box::new(CellValue::Bool(true)),
                text: "note".to_owned(),
            },
            CellValue::Image(vec![1, 2, 3]),
            CellValue::RichText(RichTextStringData::new("rich")),
            CellValue::Images {
                value: Box::new(CellValue::Int(5)),
                images: vec![ImageData::new(vec![9, 8])],
            },
        ];
        writer.write_row(&rows).expect("write");
        let mut reader = writer.finish().expect("finish");
        let decoded = reader.next_row().expect("decode").expect("row");
        assert_eq!(decoded, rows);
        assert!(reader.next_row().expect("eof").is_none());
    }

    #[test]
    fn spill_value_adapter_reports_invalid_typed_text() {
        let decimal_err = from_spill_value(GzipCellValue::Decimal("abc".to_owned()))
            .expect_err("invalid decimal must fail");
        assert!(matches!(decimal_err, ExcelError::Format(_)));

        let date_err = from_spill_value(GzipCellValue::Date("bad".to_owned()))
            .expect_err("invalid date must fail");
        assert!(matches!(date_err, ExcelError::Format(_)));

        let datetime_err = from_spill_value(GzipCellValue::DateTime("bad".to_owned()))
            .expect_err("invalid datetime must fail");
        assert!(matches!(datetime_err, ExcelError::Format(_)));
    }

    #[test]
    fn gzip_spill_reader_reports_non_eof_stream_errors() {
        let directory = tempfile::tempdir().expect("tempdir");
        let bad_path = directory.path().join("bad.gz");
        std::fs::write(&bad_path, b"not a gzip stream").expect("write");
        let mut reader = GzipSpillReader {
            inner: EngineSpillReader::open_path(bad_path, "Sheet1").expect("open"),
            styles: Vec::new(),
        };
        let error = reader.next_row().expect_err("corrupt stream must fail");
        assert!(matches!(error, ExcelError::Io(_)));
    }

    #[test]
    fn file_has_gzip_magic_missing_file_returns_false() {
        assert!(!file_has_gzip_magic(std::path::Path::new(
            "/nonexistent/path/definitely-missing.gz"
        )));
    }
}
