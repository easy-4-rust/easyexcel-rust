//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.DummyRecordHandler`.
//!
//! Handles POI "dummy" records that mark end-of-row and missing cells.

use std::collections::HashMap;

use super::super::xls_record_handler::XlsRecordHandler;
use super::blank_record_handler::BlankCell;

include!("dummy_record_handler/dummy_record_event.rs");

/// 对应 Java：`DummyRecordHandler`.
#[derive(Debug, Default)]
pub struct DummyRecordHandler;

impl DummyRecordHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.DummyRecordHandler。 Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.DummyRecordHandler。 Java `LastCellOfRowDummyRecord` branch.
    #[must_use]
    pub fn process_last_cell_of_row(row: u32) -> DummyRecordEvent {
        DummyRecordEvent::EndRow { row }
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.DummyRecordHandler。 Java `MissingCellDummyRecord` branch — `putIfAbsent` semantics.
    ///
    /// Returns `Some(MissingCell)` only when the column is not already present
    /// (see `EasyExcel` issue #2236).
    #[must_use]
    // 对应 Java：参数 `Map<Long, ?>` 仅作“列是否已存在”的键集合使用，
    // 保留 `HashMap<usize, ()>` 形态以镜像 Java 侧容器类型。
    #[allow(clippy::zero_sized_map_values)]
    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.DummyRecordHandler。
    pub fn process_missing_cell(
        row: u32,
        column: usize,
        existing: &HashMap<usize, ()>,
    ) -> Option<DummyRecordEvent> {
        if existing.contains_key(&column) {
            return None;
        }
        Some(DummyRecordEvent::MissingCell(BlankCell { row, column }))
    }
}

impl XlsRecordHandler for DummyRecordHandler {
    /// Java `DummyRecordHandler.processRecord` — POI synthesised dummy records
    /// are not true BIFF sids; use [`Self::process_last_cell_of_row`] /
    /// [`Self::process_missing_cell`].
    fn process_record(&mut self, _record_sid: u16, _data: &[u8]) {
        // No-op by design (matches Java's instanceof branches on DummyRecord).
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // 对应 Java：测试构造 `HashMap<usize, ()>` 与 `process_missing_cell` 参数
    // 形态保持一致（见上 allow 说明）。
    #[allow(clippy::zero_sized_map_values)]
    fn missing_cell_skips_existing_columns() {
        let mut map = HashMap::new();
        map.insert(1usize, ());
        assert!(DummyRecordHandler::process_missing_cell(0, 1, &map).is_none());
        assert!(DummyRecordHandler::process_missing_cell(0, 2, &map).is_some());
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn last_cell_of_row_event() {
        // 对应 Java：LastCellOfRowDummyRecord 分支
        assert_eq!(
            DummyRecordHandler::process_last_cell_of_row(4),
            DummyRecordEvent::EndRow { row: 4 }
        );
    }

    #[test]
    fn process_record_is_noop_by_design() {
        // 对应 Java：DummyRecord 非真实 BIFF sid，processRecord 空实现
        let mut handler = DummyRecordHandler::new();
        handler.process_record(u16::MAX, &[]);
        handler.process_record(0xFFFF, &[1, 2]);
    }
}
