//! Documented stubs for functions we deliberately do not support: cube (`CUBE*`),
//! web (`WEBSERVICE`, `FILTERXML`, `ENCODEURL`), RTD, and pivot-only functions.
//! These return `#N/A` (or `#NAME?` where Excel itself would not know them) so
//! that opening a workbook that uses them degrades gracefully rather than
//! mis-evaluating.

use super::Registry;
use crate::formula::context::Context;
use crate::formula::value::Value;
use easyexcel_model::error::CellError;

pub fn register(r: &mut Registry) {
    for name in EXT_FUNCTIONS {
        r.add(name, 0, super::VARIADIC, false, unsupported);
    }
}

fn unsupported(_: &mut dyn Context, _: &[Value]) -> Value {
    // PARITY: external-data/cube/RTD functions cannot be computed offline.
    Value::Error(CellError::NA)
}

/// Functions that require external data, a cube/OLAP connection, RTD, or pivot
/// cache and therefore cannot be evaluated by an offline engine.
pub const EXT_FUNCTIONS: &[&str] = &[
    "CUBEKPIMEMBER",
    "CUBEMEMBER",
    "CUBEMEMBERPROPERTY",
    "CUBERANKEDMEMBER",
    "CUBESET",
    "CUBESETCOUNT",
    "CUBEVALUE",
    "RTD",
    "WEBSERVICE",
    "FILTERXML",
    "ENCODEURL",
    "GETPIVOTDATA",
    // Locale / East-Asian text and phonetic functions (need locale/phonetic data).
    "ASC",
    "DBCS",
    "PHONETIC",
    "BAHTTEXT",
    "DETECTLANGUAGE",
    "TRANSLATE",
    // Pivot / image / stock / macro functions (need a host application or service).
    "GROUPBY",
    "PIVOTBY",
    "IMAGE",
    "STOCKHISTORY",
    "CALL",
    "REGISTER.ID",
];
