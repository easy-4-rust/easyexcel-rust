//! The evaluation context: the interface a function implementation uses to reach
//! back into the workbook (read cells, resolve sheets, get the clock). The
//! evaluator implements this; functions are written against the trait so they
//! stay decoupled from the engine internals.

use super::value::{Array, RefRange, Value};
use crate::core::dates::DateSystem;

/// The cell currently being evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRef {
    pub sheet: usize,
    pub row: u32,
    pub col: u32,
}

/// Interface functions use to query the workbook during evaluation.
pub trait Context {
    /// The workbook's date system (1900 / 1904).
    fn date_system(&self) -> DateSystem;

    /// The cell whose formula is being evaluated (for relative refs, ROW(), …).
    fn current(&self) -> CellRef;

    fn sheet_count(&self) -> usize;

    /// Resolve a sheet name (case-insensitive) to its index.
    fn sheet_index(&self, name: &str) -> Option<usize>;

    /// The name of a sheet by index.
    fn sheet_name(&self, idx: usize) -> Option<String>;

    /// The scalar value at a concrete location. Out-of-range returns
    /// [`Value::Empty`]. May trigger lazy evaluation of dependencies.
    fn cell(&mut self, sheet: usize, row: u32, col: u32) -> Value;

    /// Current date+time as a serial number (volatile; used by NOW).
    fn now_serial(&self) -> f64;

    /// Current date as an integer serial number (volatile; used by TODAY).
    fn today_serial(&self) -> f64;

    /// Materialize a range reference into a dense array of scalars.
    fn ref_to_array(&mut self, r: RefRange) -> Array {
        let rows = r.rows() as usize;
        let cols = r.cols() as usize;
        let mut data = Vec::with_capacity(rows * cols);
        for (row, col) in r.iter() {
            data.push(self.cell(r.sheet, row, col));
        }
        Array { rows, cols, data }
    }

    /// Flatten any value (scalar/array/ref) into a list of scalar values, used by
    /// aggregate functions (SUM, COUNT, …). Errors are included verbatim so the
    /// caller can decide whether to propagate.
    fn flatten(&mut self, v: &Value) -> Vec<Value> {
        match v {
            Value::Array(a) => a.data.clone(),
            Value::Ref(r) => {
                let mut out = Vec::new();
                let rr = *r;
                for (row, col) in rr.iter() {
                    out.push(self.cell(rr.sheet, row, col));
                }
                out
            }
            other => vec![other.clone()],
        }
    }
}
