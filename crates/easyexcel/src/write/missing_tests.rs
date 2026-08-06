//! Missing test coverage to match Java easyexcel test suite.
//!
//! Tests that use types from `easyexcel-writer` (`LoopMergeStrategy`, etc.)

use super::*;
use crate::write::handler_execution_scope::HandlerExecutionScope;
use crate::write::shared_write_handler::{
    StatefulSheetState, boxed_handlers, normalized_shared_handlers, share_handlers,
};

// ============================================================================
// RepetitionDataTest (7 tests) — Repetition
// ============================================================================

include!("missing_tests_split/chunk_01.rs");

// ============================================================================
// FillStyleDataTest (5 tests) — Fill style
// ============================================================================

// ============================================================================
// FillAnnotationDataTest (3 tests) — Fill annotation
// ============================================================================

// ============================================================================
// FillStyleAnnotatedTest (3 tests) — Fill style annotated
// ============================================================================

// ============================================================================
// Canonical writer helper tests
// ============================================================================

// ============================================================================
// Additional canonical writer helper tests
// ============================================================================

include!("missing_tests_split/chunk_02.rs");

// ============================================================================
// Additional canonical SharedWriteHandler tests
// ============================================================================
