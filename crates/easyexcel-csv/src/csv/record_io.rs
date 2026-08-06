//! CSV 增量记录读写后端。

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use easyexcel_io::{Error, Result};
use encoding_rs_io::DecodeReaderBytes;

use super::{CsvCharset, CsvEncodingWriter, csv_bom, csv_encoding, decode_reader};

include!("record_io/csv_record_writer.rs");

include!("record_io/csv_record_reader.rs");
