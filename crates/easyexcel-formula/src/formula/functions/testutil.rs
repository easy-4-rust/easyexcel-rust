//! A lightweight [`Context`] for unit-testing function implementations in
//! isolation, backed by an optional small dense grid of cell values.

use std::collections::HashMap;

use crate::formula::context::{CellRef, Context};
use crate::formula::value::Value;
use easyexcel_model::dates::DateSystem;

/// A mock evaluation context. By default it is an empty single-sheet workbook;
/// use [`TestCtx::with_cells`] to seed cell values for range-based functions.
pub struct TestCtx {
    cells: HashMap<(usize, u32, u32), Value>,
    sheets: Vec<String>,
    current: CellRef,
    pub now: f64,
    pub today: f64,
}

impl TestCtx {
    pub fn new() -> Self {
        TestCtx {
            cells: HashMap::new(),
            sheets: vec!["Sheet1".to_string()],
            current: CellRef {
                sheet: 0,
                row: 0,
                col: 0,
            },
            now: 45000.5,
            today: 45000.0,
        }
    }

    /// Seed cells on sheet 0 from `(row, col, value)` triples.
    pub fn with_cells(cells: &[(u32, u32, Value)]) -> Self {
        let mut c = TestCtx::new();
        for (r, col, v) in cells {
            c.cells.insert((0, *r, *col), v.clone());
        }
        c
    }

    pub fn set(&mut self, row: u32, col: u32, v: Value) {
        self.cells.insert((0, row, col), v);
    }

    pub fn set_current(&mut self, row: u32, col: u32) {
        self.current = CellRef { sheet: 0, row, col };
    }
}

impl Default for TestCtx {
    fn default() -> Self {
        TestCtx::new()
    }
}

impl Context for TestCtx {
    fn date_system(&self) -> DateSystem {
        DateSystem::Date1900
    }
    fn current(&self) -> CellRef {
        self.current
    }
    fn sheet_count(&self) -> usize {
        self.sheets.len()
    }
    fn sheet_index(&self, name: &str) -> Option<usize> {
        self.sheets
            .iter()
            .position(|s| s.eq_ignore_ascii_case(name))
    }
    fn sheet_name(&self, idx: usize) -> Option<String> {
        self.sheets.get(idx).cloned()
    }
    fn cell(&mut self, sheet: usize, row: u32, col: u32) -> Value {
        self.cells
            .get(&(sheet, row, col))
            .cloned()
            .unwrap_or(Value::Empty)
    }
    fn now_serial(&self) -> f64 {
        self.now
    }
    fn today_serial(&self) -> f64 {
        self.today
    }
}

/// Helper to build a [`Value::Ref`] over sheet 0 for tests.
pub fn rng(r0: u32, c0: u32, r1: u32, c1: u32) -> Value {
    Value::Ref(crate::formula::value::RefRange {
        sheet: 0,
        start_row: r0,
        start_col: c0,
        end_row: r1,
        end_col: c1,
    })
}
