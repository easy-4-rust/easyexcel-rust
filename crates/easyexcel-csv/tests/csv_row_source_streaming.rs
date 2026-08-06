//! CSV `RowSource` 必须在读完整个输入前向下游发出首行。

use std::io::{self, Read};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use easyexcel_csv::{CsvCharset, CsvReadOptions, CsvRowSource};
use easyexcel_io::{Error, Result, RowSink, RowSource, StreamCell, StreamInfo};

struct CountingReader {
    bytes: Vec<u8>,
    position: usize,
    consumed: Arc<AtomicUsize>,
}

impl Read for CountingReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let remaining = &self.bytes[self.position..];
        let count = remaining.len().min(buffer.len());
        buffer[..count].copy_from_slice(&remaining[..count]);
        self.position += count;
        self.consumed.fetch_add(count, Ordering::Relaxed);
        Ok(count)
    }
}

struct StopAfterFirstRow;

impl RowSink for StopAfterFirstRow {
    fn begin(&mut self, _info: &StreamInfo) -> Result<()> {
        Ok(())
    }

    fn row(&mut self, _row_index: u32, _cells: &[StreamCell]) -> Result<()> {
        Err(Error::Other("stop after first row".to_owned()))
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn csv_row_source_emits_rows_before_reading_the_entire_input() {
    let mut bytes = b"id,name\n1,Alice\n".to_vec();
    for index in 0..100_000 {
        bytes.extend_from_slice(format!("{index},row-{index}\n").as_bytes());
    }
    let total = bytes.len();
    let consumed = Arc::new(AtomicUsize::new(0));
    let reader = CountingReader {
        bytes,
        position: 0,
        consumed: Arc::clone(&consumed),
    };
    let mut source = CsvRowSource::new(reader, CsvReadOptions::default(), CsvCharset::default());
    let error = source
        .stream(&mut StopAfterFirstRow)
        .expect_err("sink stops the stream");
    assert!(matches!(error, Error::Other(_)));
    assert!(consumed.load(Ordering::Relaxed) < total);
}
