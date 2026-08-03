//! 对应 Java：`com.alibaba.excel.write.style.column.LongestMatchColumnWidthStyleStrategy`.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::{CellDataType, CellValue, Result, WriteCellContext, WriteHandler};

use crate::writer::style::column::abstract_head_column_width_style_strategy::AbstractHeadColumnWidthStyleStrategy;

/// Maximum Excel column width in character units. (Java `MAX_COLUMN_WIDTH = 255`)
const MAX_COLUMN_WIDTH: u16 = 255;

/// 对应 Java：`LongestMatchColumnWidthStyleStrategy`.
///
/// Java walks rendered cell content after each cell write, measures
/// `String.getBytes().length`, and calls `Sheet.setColumnWidth(col, len * 256)`
/// when a longer value appears. The Rust port:
/// - records UTF-8 byte lengths in [`WriteHandler::after_cell`]
/// - exposes the running max via [`WriteHandler::style_column_width`]
/// - the XLSX write path reapplies those widths after the sheet finishes
///
/// Optional [`Self::with_autofit_fallback`] keeps `worksheet.autofit()` as a
/// secondary path (disabled by default).
pub struct LongestMatchColumnWidthStyleStrategy {
    /// Per-column maximum content length. (Java `cache` / `maxColumnWidthMap`)
    cache: Mutex<HashMap<usize, u16>>,
    /// When true, also request autofit after the sheet write.
    autofit_fallback: bool,
}

impl LongestMatchColumnWidthStyleStrategy {
    /// Creates the strategy with length-based widths only.
    /// (Java `LongestMatchColumnWidthStyleStrategy()`)
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            autofit_fallback: false,
        }
    }

    /// Enables or disables autofit as an optional fallback after length widths.
    #[must_use]
    pub fn with_autofit_fallback(mut self, enabled: bool) -> Self {
        self.autofit_fallback = enabled;
        self
    }

    /// Returns whether autofit fallback is enabled.
    #[must_use]
    pub const fn autofit_fallback(&self) -> bool {
        self.autofit_fallback
    }

    /// Updates the cached max width for one cell. (Java `setColumnWidth` body)
    fn observe_cell(&self, context: &WriteCellContext) {
        let Some(column_width) = data_length(context) else {
            return;
        };
        let column_width = column_width.min(MAX_COLUMN_WIDTH);
        let column_index = usize::from(context.column_index);
        let Ok(mut cache) = self.cache.lock() else {
            return;
        };
        let entry = cache.entry(column_index).or_insert(0);
        if column_width > *entry {
            *entry = column_width;
        }
    }
}

impl Default for LongestMatchColumnWidthStyleStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteHandler for LongestMatchColumnWidthStyleStrategy {
    fn order(&self) -> i32 {
        // Mirror Java `OrderConstant.DEFINE_STYLE` / late column-width apply.
        -50_000
    }

    fn after_cell(&mut self, context: &WriteCellContext) -> Result<()> {
        // Java `AbstractColumnWidthStyleStrategy.afterCellDispose` → `setColumnWidth`
        self.observe_cell(context);
        Ok(())
    }

    fn style_column_width(&self, column_index: usize) -> Option<u16> {
        self.cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(&column_index).copied())
            .filter(|width| *width > 0)
    }

    fn style_auto_column_width(&self) -> bool {
        self.autofit_fallback
    }
}

impl AbstractHeadColumnWidthStyleStrategy for LongestMatchColumnWidthStyleStrategy {
    fn head_column_width(&self, column_index: usize) -> Option<u16> {
        self.style_column_width(column_index)
    }
}

/// Computes the Java-compatible content length for longest-match column width.
///
/// Head cells always use the string/text form. Content cells only measure
/// STRING / BOOLEAN / NUMBER (Java `dataLength` switch); other types return
/// `None` (Java `-1`).
fn data_length(context: &WriteCellContext) -> Option<u16> {
    if context.is_head {
        return byte_len(&context.value.as_text());
    }
    // Java unwraps WriteCellData list; Images/Comment wrap a scalar value.
    let value = match &context.value {
        CellValue::Comment { value, .. } | CellValue::Images { value, .. } => value.as_ref(),
        other => other,
    };
    match value.data_type() {
        CellDataType::String | CellDataType::Boolean | CellDataType::Number => {
            byte_len(&value.as_text())
        }
        _ => None,
    }
}

/// UTF-8 byte length capped to `u16`, approximating Java `String.getBytes().length`.
fn byte_len(text: &str) -> Option<u16> {
    u16::try_from(text.len()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::{CellValue, WriteCellContext, WriteHandler};
    use chrono::NaiveDate;

    fn context(value: CellValue, is_head: bool) -> WriteCellContext {
        let mut context = WriteCellContext::new("S", 0, 0, value);
        context.is_head = is_head;
        context
    }

    #[test]
    fn longest_match_observes_widths_and_reports_them() {
        let mut strategy = LongestMatchColumnWidthStyleStrategy::new().with_autofit_fallback(true);
        assert!(strategy.autofit_fallback());
        strategy
            .after_cell(&context(CellValue::String("hello".to_owned()), false))
            .unwrap();
        strategy
            .after_cell(&context(CellValue::Bool(true), false))
            .unwrap();
        strategy
            .after_cell(&context(CellValue::Int(12345), false))
            .unwrap();
        strategy
            .after_cell(&context(
                CellValue::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                false,
            ))
            .unwrap();
        strategy
            .after_cell(&context(
                CellValue::Comment {
                    value: Box::new(CellValue::String("note".to_owned())),
                    text: "note".to_owned(),
                },
                false,
            ))
            .unwrap();
        strategy
            .after_cell(&context(
                CellValue::Images {
                    value: Box::new(CellValue::Int(1)),
                    images: Vec::new(),
                },
                false,
            ))
            .unwrap();
        strategy
            .after_cell(&context(CellValue::Empty, true))
            .unwrap();
        let width = strategy.style_column_width(0);
        assert!(width.is_some_and(|w| w > 0));
        assert_eq!(strategy.head_column_width(0), width);
        assert!(strategy.style_auto_column_width());
    }

    #[test]
    fn longest_match_poisoned_cache_returns_early() {
        let mut strategy = LongestMatchColumnWidthStyleStrategy::new();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = strategy.cache.lock().expect("lock");
            panic!("poison the cache");
        }));
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::String("x".to_owned()));
        context.is_head = false;
        let _ = strategy.after_cell(&context);
        assert_eq!(strategy.style_column_width(0), None);
    }

    #[test]
    fn longest_match_default_has_no_autofit_fallback() {
        let strategy = LongestMatchColumnWidthStyleStrategy::default();
        assert!(!strategy.autofit_fallback());
        assert!(!strategy.style_auto_column_width());
        assert_eq!(strategy.order(), -50_000);
        assert_eq!(strategy.style_column_width(99), None);
    }
}
