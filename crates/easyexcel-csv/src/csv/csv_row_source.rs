use std::io::{BufRead, BufReader, Read};

use easyexcel_io::{Error, Result, RowSink, RowSource, StreamCell, StreamInfo};
use easyexcel_model::{CellValue, DateSystem};

use super::{CsvCharset, CsvReadOptions, decode_reader, detect_delimiter, infer_cell};

/// 对应 Java：无直接对应对象；Rust 架构扩展。将 CSV 增量记录公开为统一行源。
pub struct CsvRowSource<R: Read> {
    reader: Option<R>,
    options: CsvReadOptions,
    charset: CsvCharset,
}

impl<R: Read> CsvRowSource<R> {
    /// 创建 CSV 行源。
    #[must_use]
    pub fn new(reader: R, options: CsvReadOptions, charset: CsvCharset) -> Self {
        Self {
            reader: Some(reader),
            options,
            charset,
        }
    }
}

impl<R: Read> RowSource for CsvRowSource<R> {
    fn stream(&mut self, sink: &mut dyn RowSink) -> Result<()> {
        let reader = self.reader.take().ok_or_else(|| {
            Error::Unsupported("CSV row source can only be streamed once".to_owned())
        })?;
        let decoded = decode_reader(reader, &self.charset)?;
        let mut buffered = BufReader::new(decoded);
        let delimiter = match self.options.delimiter {
            Some(delimiter) => delimiter,
            None => detect_delimiter(&String::from_utf8_lossy(buffered.fill_buf()?)),
        };
        let mut csv = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(false)
            .flexible(true)
            .from_reader(buffered);

        sink.begin(&StreamInfo {
            sheet_name: self.options.sheet_name.clone(),
            date_system: DateSystem::Date1900,
        })?;
        for (row_index, record) in csv.records().enumerate() {
            let record = record.map_err(Error::from)?;
            let row = u32::try_from(row_index).map_err(|_| Error::ResourceLimit {
                resource: "rows",
                limit: u64::from(u32::MAX),
                actual: u64::try_from(row_index).unwrap_or(u64::MAX),
            })?;
            let cells = record
                .iter()
                .enumerate()
                .filter_map(|(column, field)| {
                    let value = if self.options.infer_types {
                        infer_cell(field).value()
                    } else if field.is_empty() {
                        CellValue::Empty
                    } else {
                        CellValue::Text(field.to_owned())
                    };
                    (!matches!(value, CellValue::Empty)).then(|| StreamCell {
                        col: u32::try_from(column).unwrap_or(u32::MAX),
                        value,
                        number_format: String::new(),
                    })
                })
                .collect::<Vec<_>>();
            sink.row(row, &cells)?;
        }
        sink.end()
    }
}
