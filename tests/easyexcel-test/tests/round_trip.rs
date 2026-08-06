//! End-to-end compatibility tests for the public facade.

use std::cell::RefCell;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

use chrono::NaiveDate;
use easyexcel::{
    AnalysisContext, AnchorType, BigInt, CellStyle, CellValue, ClientAnchorData, Converter,
    CoordinateData, EasyExcel, ExcelCellStyle, ExcelColor, ExcelColumn, ExcelError,
    ExcelFillPattern, ExcelFontScript, ExcelRow, ExcelUnderline, HorizontalAlignment, ImageData,
    ImageInputStream, InputStreamImageConverter, IntoExcelCell, LoopMergeProperty,
    OnceAbsoluteMergeProperty, PageReadListener, ReadConverterContext, ReadListener, Result,
    RichTextStringData, Url, UrlImageConverter, VerticalAlignment, WriteCellData,
    WriteConverterContext, WriteFont,
};
use tempfile::tempdir;
use zip::ZipArchive;

include!("round_trip_cases/user_to_default_registry_writes_type_erased_input_stream_as_image.rs");
include!(
    "round_trip_cases/public_facade_round_trips_scalar_write_cell_data_and_emits_multiple_imag.rs"
);
