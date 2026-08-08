//! Minimal BIFF8 `.xls` template package (Java `withTemplate` / HSSF subset).
//!
//! # Approach
//!
//! Loads the OLE/CFB container, parses the `Workbook` stream into BIFF records,
//! and **preserves every untouched record byte-for-byte** (FONT / XF / SST /
//! MERGECELLS / existing cells). New values are inserted as inline `LABEL`
//! (0x0204) or `NUMBER` / `BOOLERR` / `BLANK` records immediately before the
//! target sheet's `EOF`, then `DIMENSION` and `BOUNDSHEET` stream offsets are
//! repaired. Other OLE streams (`SummaryInformation`, …) are kept by rewriting
//! only the `Workbook` / `Book` stream in place.
//!
//! # Java mapping
//!
//! | Java `EasyExcel` / POI | Rust |
//! |---|---|
//! | `EasyExcel.write(...).withTemplate(xls).sheet().doWrite(data)` | [`Biff8TemplatePackage`] + writer wiring |
//! | `HSSFWorkbook(templateStream)` | OLE open + Workbook record parse |
//! | `sheet.createRow(...).createCell(...).setCellValue(...)` | [`Biff8TemplatePackage::set_cell`] |
//! | POI keeps unedited records | unchanged BIFF records copied verbatim |
//!
//! Scalar/collection placeholders, `forceNewRow`, horizontal fill, style copy,
//! formula/range repair and call-scoped `CryptoAPI` password input/output are
//! implemented by this package. Unmodified BIFF records and OLE streams remain
//! byte-preserved.
//!
//! For `.xls` cell append (Java `withTemplate` + `doWrite`), use this package
//! via the writer facade instead of OOXML fill.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use cfb::CompoundFile;
use easyexcel_io::{Error as ExcelError, Result};

use super::encode::{
    BLANK, BOF, BOOLERR, BOUNDSHEET, DIMENSION, DT_WORKSHEET, EOF, FILEPASS, FONT, FORMULA, LABEL,
    LABELSST, MAX_RECORD_DATA, MERGECELLS, MSODRAWINGGROUP, NUMBER, RK, SST, SUPBOOK, XF,
    WINDOW2, XF_GENERAL, EXTERNSHEET, encode_rk,
    encode_unicode_string, pack_merge_range,
};
use super::record_sid::{
    CHART_AI_SID, CONDITIONAL_FORMATTING_HEADER_SID, CONDITIONAL_FORMATTING_RULE_SID,
    DATA_VALIDATION_SID, EXTERNAL_SHEET_SID, HYPERLINK_SID, MSO_DRAWING_SID, NAME_SID, NOTE_SID,
    OBJECT_PROTECT_SID, OBJ_SID, PASSWORD_SID, PROTECT_SID, RICH_STRING_SID, ROW_SID,
    SCENARIO_PROTECT_SID, SUP_BOOK_SID, TEXT_OBJECT_SID,
};
use super::{
    Biff8Cell, Biff8Chart, Biff8Comment, Biff8Hyperlink, Biff8HyperlinkKind, Biff8Merge, Biff8Value,
    decrypt_crypto_api_workbook_stream, legacy_password_hash,
    encrypt_crypto_api_workbook_stream, prepare_crypto_api_encryption,
};

include!("template/biff8_macro_policy.rs");
include!("template/rawrecord_to_scalar_placeholder_key.rs");
include!("template/collection_placeholder_key.rs");
include!("template/shift_formula_references.rs");
