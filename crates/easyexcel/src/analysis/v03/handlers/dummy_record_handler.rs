//! 对应 Java：`com.alibaba.excel.analysis.v03.handlers.DummyRecordHandler`.
//!
//! Handles POI "dummy" records that mark end-of-row and missing cells.

use std::collections::{HashMap, HashSet};

use super::super::xls_record_handler::XlsRecordHandler;
use super::blank_record_handler::BlankCell;

include!("dummy_record_handler/dummy_record_event.rs");

/// 对应 Java：`DummyRecordHandler`.
#[derive(Debug, Default)]
pub struct DummyRecordHandler {
    occupied_columns: HashSet<usize>,
    last_event: Option<DummyRecordEvent>,
}

impl DummyRecordHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v03.handlers.DummyRecordHandler。 Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

    /// Records a real cell already present in the current Java `cellMap`.
    pub(crate) fn observe_cell(&mut self, column: usize) {
        self.occupied_columns.insert(column);
    }

    /// Takes the semantic event emitted by the most recent dummy record.
    pub(crate) fn take_event(&mut self) -> Option<DummyRecordEvent> {
        self.last_event.take()
    }
}

impl XlsRecordHandler for DummyRecordHandler {
    /// Java `DummyRecordHandler.processRecord` for POI-synthesised records.
    ///
    /// Dummy records have no physical BIFF body. The Rust dispatcher therefore
    /// uses a stable internal envelope after sid `0xFFFF`: tag `0` + row `u32`
    /// for `LastCellOfRowDummyRecord`, or tag `1` + row `u32` + column `u32`
    /// for `MissingCellDummyRecord`.
    fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        const DUMMY_RECORD_SID: u16 = u16::MAX;
        const END_ROW_TAG: u8 = 0;
        const MISSING_CELL_TAG: u8 = 1;

        self.last_event = None;
        if record_sid != DUMMY_RECORD_SID {
            return;
        }
        match data.first().copied() {
            Some(END_ROW_TAG) => {
                let Some(row) = decode_u32(data, 1) else {
                    return;
                };
                self.last_event = Some(Self::process_last_cell_of_row(row));
                self.occupied_columns.clear();
            }
            Some(MISSING_CELL_TAG) => {
                let (Some(row), Some(column)) = (decode_u32(data, 1), decode_u32(data, 5)) else {
                    return;
                };
                let Ok(column) = usize::try_from(column) else {
                    return;
                };
                if self.occupied_columns.insert(column) {
                    self.last_event =
                        Some(DummyRecordEvent::MissingCell(BlankCell { row, column }));
                }
            }
            _ => {}
        }
    }
}

fn decode_u32(data: &[u8], offset: usize) -> Option<u32> {
    data.get(offset..offset.saturating_add(4))
        .and_then(|bytes| bytes.try_into().ok())
        .map(u32::from_le_bytes)
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
    fn process_record_decodes_missing_cell_and_end_row() {
        // 对应 Java：MissingCellDummyRecord 使用 putIfAbsent，LastCell 清空当前行。
        let mut handler = DummyRecordHandler::new();
        handler.observe_cell(2);

        let mut occupied = vec![1];
        occupied.extend_from_slice(&3u32.to_le_bytes());
        occupied.extend_from_slice(&2u32.to_le_bytes());
        handler.process_record(u16::MAX, &occupied);
        assert!(handler.take_event().is_none());

        let mut missing = vec![1];
        missing.extend_from_slice(&3u32.to_le_bytes());
        missing.extend_from_slice(&4u32.to_le_bytes());
        handler.process_record(u16::MAX, &missing);
        assert_eq!(
            handler.take_event(),
            Some(DummyRecordEvent::MissingCell(BlankCell {
                row: 3,
                column: 4
            }))
        );

        let mut end_row = vec![0];
        end_row.extend_from_slice(&3u32.to_le_bytes());
        handler.process_record(u16::MAX, &end_row);
        assert_eq!(
            handler.take_event(),
            Some(DummyRecordEvent::EndRow { row: 3 })
        );

        handler.process_record(0x1234, &missing);
        assert!(handler.take_event().is_none());
    }
}
