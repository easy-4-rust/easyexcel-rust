//! The tree-walking evaluator. Turns an [`Expr`] into a [`Value`] against a
//! workbook snapshot, handling operators, reference resolution, defined names,
//! and the lazy "special form" functions (IF, IFERROR, CHOOSE, …).

use std::rc::Rc;

use super::ast::{BinaryOp, Expr, Reference, SheetSpec, UnaryOp};
use super::coerce;
use super::context::{CellRef, Context};
use super::functions::Registry;
use super::value::{Array, Lambda, RefRange, Value};
use easyexcel_model::error::CellError;
use easyexcel_model::model::Workbook;

include!("eval/binding_to_wants_reference.rs");
include!("eval/evaluator_impl.rs");
include!("eval/is_arrayish_to_evaluator_impl.rs");
