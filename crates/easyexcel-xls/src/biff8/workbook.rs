//! In-memory BIFF8 workbook model and OLE/CFB serialization.
//!
//! Java mapping: Alibaba `EasyExcel` `excelType(ExcelTypeEnum.XLS)` → POI HSSF.
//! This module is the single generated-workbook BIFF8 engine (not a full HSSF port):
//! - Supported: multi-sheet values/formulas, errors, rich SST, 1900/1904 date
//!   windows, visibility, dimensions, FONT/XF/fill/border, merges, hyperlinks,
//!   comments, charts, protection and workbook CryptoAPI encryption.
//! - Template scalar/collection fill and in-place OLE patching are implemented by
//!   the independent template package. URL hyperlinks, comments, rich text,
//!   borders, native Bar/Line/Pie charts, VBA preservation and `CryptoAPI`
//!   password encryption are consumed by both the stateful writer and the
//!   format-neutral `xls::write` adapter.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Cursor, Write};
use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use easyexcel_io::{Error as ExcelError, Result};

use super::cached::Biff8Cached;
use super::encode::{
    BIFF8_VERSION, BLANK, BOF, BOOLERR, BOUNDSHEET, CALCMODE, CODENAME, CODEPAGE, COLINFO,
    CONTINUE, DATEMODE, DBCELL, DEFAULTROWHEIGHT, DEFCOLWIDTH, DIMENSION, DT_GLOBALS, DT_WORKSHEET,
    EOF, EXTERNSHEET, EXTSST, FILEPASS, FONT, FORMAT, FORMULA, HYPERLINK, INDEX, INTERFACEEND,
    INTERFACEHDR, LABELSST, MAX_RECORD_DATA, MMS, MSODRAWING, MSODRAWINGGROUP, MULBLANK, MULRK,
    NOTE, NUMBER, OBJ, OBJECTPROTECT, PANE, PASSWORD, PROTECT, RK, ROW, SCENPROTECT, SST,
    STANDARDWIDTH, STRING, STYLE, SUPBOOK, TXO, WINDOW1, WINDOW2, WRITEACCESS, XF, XF_DATE,
    XF_DATETIME, XF_GENERAL, encode_rk, encode_short_unicode_string, encode_unicode_string,
    pack_colinfo_metadata, pack_default_row, pack_merge_range, pack_row_metadata, pack_window1,
    record, write_merge_cells, write_palette_record,
};
use super::style::Biff8StyleTable;

include!("workbook/biff8cell_to_write_bof.rs");
include!("workbook/biff8_sst_framer.rs");
include!("workbook/write_default_font_to_tests_extra.rs");

include!("workbook/write_comments.rs");
include!("workbook/write_charts.rs");
