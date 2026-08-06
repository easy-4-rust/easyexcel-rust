//! In-memory BIFF8 workbook model and OLE/CFB serialization.
//!
//! Java mapping: Alibaba `EasyExcel` `excelType(ExcelTypeEnum.XLS)` → POI HSSF.
//! This module is a **minimal** BIFF8 writer (not a full HSSF port):
//! - Supported: single/multi sheet, header + data rows, string / number / bool /
//!   date / datetime cells, SST shared strings, 1900 date system, column widths
//!   (COLINFO), row heights (ROW), basic FONT/XF (bold/italic/size/indexed or
//!   approximated RGB fill), MERGECELLS ranges.
//! - Template / scalar fill is implemented by the independent template package.
//!   (styles/merges not preserved). Collection fill and in-place OLE patching remain
//!   unsupported. Also unsupported: password encryption, images, true formula tokens,
//!   hyperlink/comment records, rich-text runs, borders, arbitrary custom number
//!   formats, charts, macros. Gaps fail visibly — never silently rewrite as XLSX.

use std::collections::{BTreeMap, HashMap};
use std::io::{Cursor, Write};
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use easyexcel_io::{Error as ExcelError, Result};

use super::cached::Biff8Cached;
use super::encode::{
    BIFF8_VERSION, BLANK, BOF, BOOLERR, BOUNDSHEET, CALCMODE, CODENAME, CODEPAGE, COLINFO,
    CONTINUE, DATEMODE, DIMENSION, DT_GLOBALS, DT_WORKSHEET, EOF, EXTSST, FONT, FORMAT, FORMULA,
    INTERFACEEND, INTERFACEHDR, LABELSST, MAX_RECORD_DATA, MMS, MSODRAWING, MULBLANK, MULRK,
    NUMBER, OBJ, PANE, RK, ROW, SST, STRING, STYLE, WINDOW2, WRITEACCESS, XF, XF_DATE, XF_DATETIME,
    XF_GENERAL, encode_rk, encode_short_unicode_string, encode_unicode_string, pack_colinfo,
    pack_merge_range, pack_row, record, write_merge_cells, write_palette_record,
};
use super::style::Biff8StyleTable;

include!("workbook/biff8cell_to_write_bof.rs");
include!("workbook/write_default_font_to_tests_extra.rs");
