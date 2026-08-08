//! 对应 Java：`com.alibaba.excel.analysis.v03.IgnorableXlsRecordHandler`.

use super::xls_record_handler::XlsRecordHandler;

/// 对应 Java：`IgnorableXlsRecordHandler extends XlsRecordHandler`.
///
/// Java marks handlers whose records belong to a worksheet and may be skipped
/// while the current worksheet is not selected.
pub trait IgnorableXlsRecordHandler: XlsRecordHandler {
    /// 返回该处理器是否带有 Java `IgnorableXlsRecordHandler` 标记。
    ///
    /// Java 使用空 marker interface；Rust 暴露只读查询，便于在不依赖
    /// 运行时反射的情况下验证并使用相同标记语义。
    #[must_use]
    fn is_ignorable(&self) -> bool {
        true
    }
}

macro_rules! impl_ignorable {
    ($($handler:path),+ $(,)?) => {
        $(impl IgnorableXlsRecordHandler for $handler {})+
    };
}

impl_ignorable!(
    super::handlers::blank_record_handler::BlankRecordHandler,
    super::handlers::bool_err_record_handler::BoolErrRecordHandler,
    super::handlers::bound_sheet_record_handler::BoundSheetRecordHandler,
    super::handlers::dummy_record_handler::DummyRecordHandler,
    super::handlers::formula_record_handler::FormulaRecordHandler,
    super::handlers::hyperlink_record_handler::HyperlinkRecordHandler,
    super::handlers::index_record_handler::IndexRecordHandler,
    super::handlers::label_record_handler::LabelRecordHandler,
    super::handlers::label_sst_record_handler::LabelSstRecordHandler,
    super::handlers::merge_cells_record_handler::MergeCellsRecordHandler,
    super::handlers::note_record_handler::NoteRecordHandler,
    super::handlers::number_record_handler::NumberRecordHandler,
    super::handlers::obj_record_handler::ObjRecordHandler,
    super::handlers::rk_record_handler::RkRecordHandler,
    super::handlers::sst_record_handler::SstRecordHandler,
    super::handlers::string_record_handler::StringRecordHandler,
    super::handlers::text_object_record_handler::TextObjectRecordHandler,
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::v03::handlers::blank_record_handler::BlankRecordHandler;

    fn assert_java_marker<T: IgnorableXlsRecordHandler>() {}

    #[test]
    fn blank_handler_implements_java_marker_interface() {
        assert_java_marker::<BlankRecordHandler>();
        assert!(BlankRecordHandler::new().is_ignorable());
    }
}
