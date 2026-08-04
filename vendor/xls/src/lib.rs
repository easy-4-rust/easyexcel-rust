// Vendored fork of zemse/xls (see Cargo.toml header). Keep the code as close
// to upstream as possible for easy syncs: silence style-only Clippy lints
// instead of editing fork logic.
#![allow(clippy::byte_char_slices, clippy::question_mark)]

//! `xls` — a terminal spreadsheet that reads and writes XLS (BIFF8), XLSX and
//! CSV, with a built-in formula engine.
//!
//! The library surface is the [`core`] module: the data model ([`core::Workbook`]),
//! the formula engine ([`core::formula`]), and the file readers/writers. The CLI
//! and TUI front-ends are compiled only when their respective Cargo features are
//! enabled and are not part of the library API.
//!
//! ```
//! use xls::core::{Workbook, Cell};
//! use xls::core::formula::Engine;
//!
//! let mut wb = Workbook::new();
//! let s = wb.sheet_mut(0).unwrap();
//! s.set_a1("A1", Cell::Number(2.0));
//! s.set_a1("A2", Cell::Number(3.0));
//! s.set_a1("A3", Cell::Formula { expr: "=A1+A2".into(), cached: Default::default() });
//! Engine::new().recalc(&mut wb);
//! assert_eq!(wb.display_cell(0, 2, 0), "5");
//! ```

pub mod core;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "tui")]
pub mod tui;

/// Re-export of the most-used public types.
pub use crate::core::{Cell, CellError, CellValue, Sheet, Workbook};
