//! 对应 Java：`com.alibaba.excel.context.AnalysisContext`。
//!
//! 拆分后仅保留 `AnalysisContext` 结构体；
//! `AnalysisContextLifecycle` trait 位于同级 `analysis_context/analysis_context_lifecycle.rs`。

use std::any::Any;

use crate::core::custom_read_object::CustomReadObject;
use crate::core::excel_error::ExcelError;

include!("analysis_context/analysis_context_lifecycle.rs");

/// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Read callback context equivalent to Java `AnalysisContext`。
///
/// Java exposes 14 methods plus several `@Deprecated` accessors. Rust keeps
/// only the methods actually consumed by `ReadListener` callbacks; legacy
/// getters (`getExcelType`, `getInputStream`, `getCurrentRowNum`, etc.) are
/// replaced by fields carried elsewhere in the read pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisContext {
    sheet_name: String,
    sheet_no: usize,
    row_index: u32,
    batch_index: usize,
    custom_object: Option<CustomReadObject>,
}

impl AnalysisContext {
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Creates a context. (Java `AnalysisContextImpl(ReadWorkbook, ExcelTypeEnum)` initial state)
    #[must_use]
    pub fn new(sheet_name: impl Into<String>, sheet_no: usize, row_index: u32) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            sheet_no,
            row_index,
            batch_index: 0,
            custom_object: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Returns the sheet name. (Java `AnalysisContext.readSheetHolder().getSheetName()`)
    #[must_use]
    pub fn sheet_name(&self) -> &str {
        &self.sheet_name
    }

    /// Returns the zero-based sheet index. (Java `AnalysisContext.readSheetHolder().getSheetNo()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。
    pub const fn sheet_no(&self) -> usize {
        self.sheet_no
    }

    /// Returns the zero-based physical row index. (Java `getCurrentRowNum()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }

    /// Returns the zero-based callback batch index.
    /// Rust extension tracking the page index in `PageReadListener`.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。
    pub const fn batch_index(&self) -> usize {
        self.batch_index
    }

    /// Returns the configured custom read object, if any.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。
    pub const fn custom_object(&self) -> Option<&CustomReadObject> {
        self.custom_object.as_ref()
    }

    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Returns the custom read object when its concrete type matches `T`.
    /// Mirrors `(T) AnalysisContext.getCustom()` after an explicit cast.
    #[must_use]
    pub fn custom<T: Any>(&self) -> Option<&T> {
        self.custom_object.as_ref()?.downcast_ref()
    }
    /// Java `getCustom()` 的类型安全兼容入口。
    #[must_use]
    pub fn get_custom<T: Any>(&self) -> Option<&T> {
        self.custom()
    }

    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Returns a context carrying the supplied custom read object.
    #[must_use]
    pub fn with_custom_object(mut self, custom_object: Option<CustomReadObject>) -> Self {
        self.custom_object = custom_object;
        self
    }

    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Returns a copy with a different batch index.
    #[must_use]
    pub fn with_batch_index(&self, batch_index: usize) -> Self {
        let mut context = self.clone();
        context.batch_index = batch_index;
        context
    }
}

include!("analysis_context/error_action.rs");

include!("analysis_context/result.rs");
