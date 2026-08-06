use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Cursor, Read as _, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ::bigdecimal::BigDecimal;
use chrono::NaiveDate;
use tempfile::tempdir;
use zip::ZipArchive;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use super::*;

/// Reads a ZIP entry from an XLSX package as UTF-8 text (integration asserts).
fn zip_entry_text(path: &Path, name: &str) -> Result<String> {
    let file = fs::File::open(path)?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut entry = archive
        .by_name(name)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut value = String::new();
    entry.read_to_string(&mut value)?;
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Value(String);

impl ExcelRow for Value {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
        COLUMNS
    }

    fn from_row(row: &RowData) -> Result<Self> {
        Ok(Self(
            row.cell(&Self::schema()[0])
                .map_or_else(String::new, CellValue::as_text),
        ))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![CellValue::String(self.0.clone())])
    }
}

include!("tests_cases/cases_01.rs");
include!("tests_cases/cases_02.rs");
include!("tests_cases/cases_03.rs");
include!("tests_cases/cases_04.rs");
include!("tests_cases/cases_05.rs");
