//! Backend-neutral row/cell handles exposed to write handlers.

use std::cell::RefCell;

use crate::{CellValue, ExcelCellStyle};

include!("write_backend_handle/write_cell_handle.rs");

include!("write_backend_handle/write_row_handle.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_handles_record_mutations_without_fake_backend_objects() {
        let row = WriteRowHandle::new(4);
        row.set_height(27);
        assert_eq!(row.row_index(), 4);
        assert_eq!(row.requested_height(), Some(27));

        let cell = WriteCellHandle::new(4, 2, CellValue::String("source".to_owned()));
        cell.set_value(CellValue::String("changed".to_owned()));
        cell.set_style(ExcelCellStyle {
            hidden: Some(true),
            ..ExcelCellStyle::new()
        });
        cell.set_skipped(false);
        assert_eq!(cell.row_index(), 4);
        assert_eq!(cell.column_index(), 2);
        assert_eq!(
            cell.requested_value(),
            Some(CellValue::String("changed".to_owned()))
        );
        assert_eq!(cell.value(), CellValue::String("changed".to_owned()));
        assert_eq!(
            cell.requested_style().and_then(|style| style.hidden),
            Some(true)
        );
        assert_eq!(cell.requested_skip(), Some(false));
    }

    #[test]
    fn sync_value_reflects_direct_mutation_and_keeps_requested() {
        // 对应 Java：context.value 被 handler 直接修改后经 sync 同步到 handle
        let cell = WriteCellHandle::new(0, 0, CellValue::String("initial".to_owned()));
        // 值未变化时同步不改变 value()（优化：跳过冗余克隆）
        cell.sync_value(&CellValue::String("initial".to_owned()));
        assert_eq!(cell.value(), CellValue::String("initial".to_owned()));
        // 直接变更后同步使 value() 可见
        cell.sync_value(&CellValue::String("direct".to_owned()));
        assert_eq!(cell.value(), CellValue::String("direct".to_owned()));
        // sync 不写入 requested（apply_cell_mutations 不重复提交）
        assert_eq!(cell.requested_value(), None);
        // set_value 后再次同步同值，value() 不变且 requested 保留
        cell.set_value(CellValue::String("mutated".to_owned()));
        cell.sync_value(&CellValue::String("mutated".to_owned()));
        assert_eq!(cell.value(), CellValue::String("mutated".to_owned()));
        assert_eq!(
            cell.requested_value(),
            Some(CellValue::String("mutated".to_owned()))
        );
    }
}
