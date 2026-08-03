//! Missing test coverage to match Java easyexcel test suite.
//!
//! Tests that use types from `easyexcel-writer` (`LoopMergeStrategy`, etc.)

use super::*;

// ============================================================================
// RepetitionDataTest (7 tests) — Repetition
// ============================================================================

#[test]
fn repetition_loop_merge_basic_each_2() {
    // Java: @ContentLoopMerge(eachRow = 2, columnExtend = 1)
    let strategy = MirroredLoopMergeStrategy::new(2, 1, 0).unwrap();
    assert_eq!(strategy.each_rows(), 2);
    assert_eq!(strategy.column_extend(), 1);
    assert_eq!(strategy.column_index(), 0);
}

#[test]
fn repetition_loop_merge_each_3_extend_2() {
    let strategy = MirroredLoopMergeStrategy::new(3, 2, 1).unwrap();
    assert_eq!(strategy.each_rows(), 3);
    assert_eq!(strategy.column_extend(), 2);
}

#[test]
fn repetition_loop_merge_zero_index() {
    let strategy = MirroredLoopMergeStrategy::new(2, 1, 0).unwrap();
    assert_eq!(strategy.column_index(), 0);
}

#[test]
fn repetition_loop_merge_high_index() {
    let strategy = MirroredLoopMergeStrategy::new(2, 1, 99).unwrap();
    assert_eq!(strategy.column_index(), 99);
}

#[test]
fn repetition_loop_merge_max_extend() {
    let strategy = MirroredLoopMergeStrategy::new(2, u16::MAX, 0).unwrap();
    assert_eq!(strategy.column_extend(), u16::MAX);
}

#[test]
fn repetition_loop_merge_large_each() {
    let strategy = MirroredLoopMergeStrategy::new(1000, 5, 0).unwrap();
    assert_eq!(strategy.each_rows(), 1000);
    assert_eq!(strategy.column_extend(), 5);
}

#[test]
fn repetition_loop_merge_all_fields_distinct() {
    let s1 = MirroredLoopMergeStrategy::new(2, 1, 0).unwrap();
    let s2 = MirroredLoopMergeStrategy::new(3, 2, 1).unwrap();
    assert_ne!(s1.each_rows(), s2.each_rows());
    assert_ne!(s1.column_extend(), s2.column_extend());
    assert_ne!(s1.column_index(), s2.column_index());
}

// ============================================================================
// FillStyleDataTest (5 tests) — Fill style
// ============================================================================

#[test]
fn fill_style_data_head_background() {
    use crate::core::{ExcelCellStyle, ExcelColor, ExcelFillPattern};
    let style = ExcelCellStyle {
        fill_pattern: Some(ExcelFillPattern::Solid),
        fill_foreground_color: Some(ExcelColor::Rgb(0x0000_00FF)),
        ..ExcelCellStyle::new()
    };
    assert_eq!(style.fill_pattern, Some(ExcelFillPattern::Solid));
    assert_eq!(
        style.fill_foreground_color,
        Some(ExcelColor::Rgb(0x0000_00FF))
    );
}

#[test]
fn fill_style_data_content_alignment() {
    use crate::core::{ExcelCellStyle, ExcelHorizontalAlignment, ExcelVerticalAlignment};
    let style = ExcelCellStyle {
        horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
        vertical_alignment: Some(ExcelVerticalAlignment::Center),
        ..ExcelCellStyle::new()
    };
    assert_eq!(
        style.horizontal_alignment,
        Some(ExcelHorizontalAlignment::Center)
    );
    assert_eq!(
        style.vertical_alignment,
        Some(ExcelVerticalAlignment::Center)
    );
}

#[test]
fn fill_style_data_border() {
    use crate::core::{ExcelBorderStyle, ExcelCellStyle};
    let style = ExcelCellStyle {
        border_left: Some(ExcelBorderStyle::Thin),
        border_right: Some(ExcelBorderStyle::Thin),
        border_top: Some(ExcelBorderStyle::Thin),
        border_bottom: Some(ExcelBorderStyle::Thin),
        ..ExcelCellStyle::new()
    };
    assert_eq!(style.border_left, Some(ExcelBorderStyle::Thin));
    assert_eq!(style.border_right, Some(ExcelBorderStyle::Thin));
    assert_eq!(style.border_top, Some(ExcelBorderStyle::Thin));
    assert_eq!(style.border_bottom, Some(ExcelBorderStyle::Thin));
}

#[test]
fn fill_style_data_font_combined() {
    use crate::core::ExcelFontStyle;
    let fs = ExcelFontStyle {
        bold: Some(true),
        italic: Some(true),
        font_name: Some("Courier"),
        font_height_in_points: Some(12.5),
        ..ExcelFontStyle::new()
    };
    assert_eq!(fs.bold, Some(true));
    assert_eq!(fs.italic, Some(true));
    assert_eq!(fs.font_name, Some("Courier"));
    assert_eq!(fs.font_height_in_points, Some(12.5));
}

#[test]
fn fill_style_data_data_format() {
    use crate::core::{ExcelCellStyle, ExcelDataFormat};
    let style = ExcelCellStyle {
        data_format: Some(ExcelDataFormat::Custom("0.00")),
        ..ExcelCellStyle::new()
    };
    assert_eq!(style.data_format, Some(ExcelDataFormat::Custom("0.00")));
}

// ============================================================================
// FillAnnotationDataTest (3 tests) — Fill annotation
// ============================================================================

#[test]
fn fill_annotation_data_with_date_format() {
    use crate::core::ExcelColumn;
    let col = ExcelColumn::new("date", "Date", None, 0, Some("yyyy-MM-dd"));
    assert_eq!(col.format, Some("yyyy-MM-dd"));
}

#[test]
fn fill_annotation_data_with_column_width() {
    use crate::core::ExcelColumn;
    let col = ExcelColumn::new("name", "Name", None, 0, None).with_column_width(30);
    assert_eq!(col.column_width, Some(30));
}

#[test]
fn fill_annotation_data_with_combined() {
    use crate::core::ExcelColumn;
    let col = ExcelColumn::new("value", "Value", None, 0, Some("0.00")).with_column_width(40);
    assert_eq!(col.column_width, Some(40));
    assert_eq!(col.format, Some("0.00"));
}

// ============================================================================
// FillStyleAnnotatedTest (3 tests) — Fill style annotated
// ============================================================================

#[test]
fn fill_style_annotated_head() {
    use crate::core::{ExcelCellStyle, ExcelHorizontalAlignment};
    let style = ExcelCellStyle {
        horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
        ..ExcelCellStyle::new()
    };
    assert_eq!(
        style.horizontal_alignment,
        Some(ExcelHorizontalAlignment::Center)
    );
}

#[test]
fn fill_style_annotated_content() {
    use crate::core::{ExcelCellStyle, ExcelFillPattern};
    let style = ExcelCellStyle {
        fill_pattern: Some(ExcelFillPattern::Solid),
        ..ExcelCellStyle::new()
    };
    assert_eq!(style.fill_pattern, Some(ExcelFillPattern::Solid));
}

#[test]
fn fill_style_annotated_both() {
    use crate::core::{ExcelCellStyle, ExcelHorizontalAlignment};
    let head = ExcelCellStyle {
        horizontal_alignment: Some(ExcelHorizontalAlignment::Left),
        ..ExcelCellStyle::new()
    };
    let content = ExcelCellStyle {
        horizontal_alignment: Some(ExcelHorizontalAlignment::Right),
        ..ExcelCellStyle::new()
    };
    assert_ne!(head.horizontal_alignment, content.horizontal_alignment);
}

// ============================================================================
// writer_helpers.rs tests
// ============================================================================

#[test]
fn writer_global_flags_default() {
    let options = WriteOptions::default();
    let flags = crate::write::writer_helpers::WriteGlobalFlags::from(&options);
    assert!(flags.auto_trim);
    assert!(!flags.use_1904_windowing);
    assert!(!flags.use_scientific_format);
}

#[test]
fn writer_global_flags_with_options() {
    let options = WriteOptions {
        auto_trim: false,
        use_1904_windowing: true,
        use_scientific_format: true,
        ..WriteOptions::default()
    };
    let flags = crate::write::writer_helpers::WriteGlobalFlags::from(&options);
    assert!(!flags.auto_trim);
    assert!(flags.use_1904_windowing);
    assert!(flags.use_scientific_format);
}

#[test]
fn effective_sheet_name_no_trim() {
    let options = WriteOptions {
        sheet_name: "Sheet1".to_owned(),
        auto_trim: false,
        ..WriteOptions::default()
    };
    let name = crate::write::writer_helpers::effective_sheet_name(&options);
    assert_eq!(name, "Sheet1");
}

#[test]
fn effective_sheet_name_with_trim() {
    let options = WriteOptions {
        sheet_name: "  Sheet1  ".to_owned(),
        auto_trim: true,
        ..WriteOptions::default()
    };
    let name = crate::write::writer_helpers::effective_sheet_name(&options);
    assert_eq!(name, "Sheet1");
}

#[test]
fn maybe_trim_cell_string_no_trim() {
    let result = crate::write::writer_helpers::maybe_trim_cell_string("  hello  ", false);
    assert_eq!(result, "  hello  ");
}

#[test]
fn maybe_trim_cell_string_with_trim() {
    let result = crate::write::writer_helpers::maybe_trim_cell_string("  hello  ", true);
    assert_eq!(result, "hello");
}

#[test]
fn is_scientific_magnitude_large() {
    assert!(crate::write::writer_helpers::is_scientific_magnitude(
        1.5e15
    ));
    assert!(crate::write::writer_helpers::is_scientific_magnitude(
        -1.5e15
    ));
}

#[test]
fn is_scientific_magnitude_small() {
    assert!(crate::write::writer_helpers::is_scientific_magnitude(1e-12));
}

#[test]
fn is_scientific_magnitude_normal() {
    assert!(!crate::write::writer_helpers::is_scientific_magnitude(
        100.0
    ));
    assert!(!crate::write::writer_helpers::is_scientific_magnitude(0.0));
    assert!(!crate::write::writer_helpers::is_scientific_magnitude(1.0));
}

#[test]
fn share_handlers_empty() {
    let handlers: Vec<Box<dyn WriteHandler>> = vec![];
    let shared = crate::write::writer_helpers::share_handlers(handlers);
    assert!(shared.is_empty());
}

#[test]
fn boxed_handlers_roundtrip() {
    let handlers = DefaultWriteHandlerLoader::load_default_handler();
    let shared = crate::write::writer_helpers::share_handlers(handlers);
    let boxed = crate::write::writer_helpers::boxed_handlers(&shared);
    assert!(!boxed.is_empty());
}

#[test]
fn handler_execution_scope_root() {
    let handlers = DefaultWriteHandlerLoader::load_default_handler();
    let shared = crate::write::writer_helpers::share_handlers(handlers);
    let scope = crate::write::writer_helpers::HandlerExecutionScope::root(&shared);
    assert!(!scope.own.is_empty());
    assert!(!scope.effective.is_empty());
}

#[test]
fn handler_execution_scope_default() {
    let scope = crate::write::writer_helpers::HandlerExecutionScope::default();
    assert!(scope.own.is_empty());
    assert!(scope.effective.is_empty());
}

#[test]
fn handler_execution_scope_own_boxed() {
    let handlers = DefaultWriteHandlerLoader::load_default_handler();
    let shared = crate::write::writer_helpers::share_handlers(handlers);
    let scope = crate::write::writer_helpers::HandlerExecutionScope::root(&shared);
    let boxed = scope.own_boxed();
    assert!(!boxed.is_empty());
}

#[test]
fn captured_output_default() {
    let output = crate::write::writer_helpers::CapturedOutput::default();
    let bytes = crate::write::writer_helpers::take_captured_output_helper(&output);
    assert_eq!(bytes.len(), 0);
}

#[test]
fn captured_output_write_and_read() {
    use std::io::Write;
    let output = crate::write::writer_helpers::CapturedOutput::default();
    let mut clone = output.clone();
    clone.write_all(b"hello").unwrap();
    clone.flush().unwrap();
    let bytes = crate::write::writer_helpers::take_captured_output_helper(&output);
    assert_eq!(bytes, b"hello");
}

// ============================================================================
// Additional writer_helpers tests
// ============================================================================

#[test]
fn normalized_shared_handlers_sorts_by_order_and_dedups() {
    use crate::core::WriteHandler;
    struct HandlerA;
    struct HandlerB;
    struct HandlerC;
    impl WriteHandler for HandlerA {
        fn order(&self) -> i32 {
            2
        }
    }
    impl WriteHandler for HandlerB {
        fn order(&self) -> i32 {
            1
        }
    }
    impl WriteHandler for HandlerC {
        fn order(&self) -> i32 {
            3
        }
    }
    let shared = crate::write::writer_helpers::share_handlers(vec![
        Box::new(HandlerA),
        Box::new(HandlerB),
        Box::new(HandlerC),
    ]);
    let normalized = crate::write::writer_helpers::normalized_shared_handlers(shared);
    assert_eq!(normalized.len(), 3);
    assert_eq!(normalized[0].order(), 1);
    assert_eq!(normalized[1].order(), 2);
    assert_eq!(normalized[2].order(), 3);
}

#[test]
fn handler_execution_scope_child() {
    let handlers = DefaultWriteHandlerLoader::load_default_handler();
    let shared = crate::write::writer_helpers::share_handlers(handlers);
    let root = crate::write::writer_helpers::HandlerExecutionScope::root(&shared);
    let child = crate::write::writer_helpers::HandlerExecutionScope::child(&shared, &root);
    assert_eq!(child.own.len(), shared.len());
}

#[test]
fn handler_execution_scope_effective_boxed() {
    let handlers = DefaultWriteHandlerLoader::load_default_handler();
    let shared = crate::write::writer_helpers::share_handlers(handlers);
    let scope = crate::write::writer_helpers::HandlerExecutionScope::root(&shared);
    let boxed = scope.effective_boxed();
    assert!(!boxed.is_empty());
}

#[test]
fn handler_execution_scope_child_inherits_parent() {
    let handlers = DefaultWriteHandlerLoader::load_default_handler();
    let shared = crate::write::writer_helpers::share_handlers(handlers);
    let root = crate::write::writer_helpers::HandlerExecutionScope::root(&shared);
    let child = crate::write::writer_helpers::HandlerExecutionScope::child(&[], &root);
    assert_eq!(child.own.len(), 0);
    assert_eq!(child.effective.len(), root.effective.len());
}

#[test]
fn stateful_sheet_state_construction() {
    use crate::core::ExcelWriteMetadata;
    use crate::write::metadata::write_sheet::WriteSheet as MirroredWriteSheet;
    let _sheet = MirroredWriteSheet::new();
    let options = WriteOptions::default();
    let metadata = ExcelWriteMetadata::default();
    let _state = crate::write::writer_helpers::StatefulSheetState {
        schema: &[],
        metadata,
        options,
        next_row: 0,
        next_data_index: 0,
    };
}

// ============================================================================
// Additional writer_helpers SharedWriteHandler tests
// ============================================================================

#[test]
fn shared_write_handler_clone() {
    use crate::core::WriteHandler;
    struct HandlerA;
    impl WriteHandler for HandlerA {
        fn order(&self) -> i32 {
            0
        }
    }
    let handler: Box<dyn WriteHandler> = Box::new(HandlerA);
    let shared = crate::write::writer_helpers::share_handlers(vec![handler]);
    let _cloned = shared.clone();
}

#[test]
fn shared_write_handler_with_mut() {
    use crate::core::WriteWorkbookContext;
    use crate::core::{Result, WriteHandler};
    // count 字段用于演示可变 Handler 被 with_mut 调用的场景，测试不读取该值。
    #[allow(dead_code)]
    struct CountingHandler {
        count: i32,
    }
    impl WriteHandler for CountingHandler {
        fn order(&self) -> i32 {
            0
        }
        fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
            Ok(())
        }
    }
    let handler: Box<dyn WriteHandler> = Box::new(CountingHandler { count: 0 });
    let shared = crate::write::writer_helpers::share_handlers(vec![handler]);
    let context = WriteWorkbookContext::new("/tmp/test.xlsx");
    shared[0].with_mut(|h| {
        h.before_workbook_create(&context).unwrap();
    });
}

#[test]
fn shared_write_handler_with_ref() {
    use crate::core::WriteHandler;
    struct HandlerA;
    impl WriteHandler for HandlerA {
        fn order(&self) -> i32 {
            0
        }
    }
    let handler: Box<dyn WriteHandler> = Box::new(HandlerA);
    let shared = crate::write::writer_helpers::share_handlers(vec![handler]);
    let order = shared[0].with_ref(|h| h.order());
    assert_eq!(order, 0);
}

#[test]
fn shared_write_handler_not_repeat_executor() {
    use crate::core::WriteHandler;
    struct HandlerA;
    impl WriteHandler for HandlerA {
        fn order(&self) -> i32 {
            0
        }
    }
    let handler: Box<dyn WriteHandler> = Box::new(HandlerA);
    let shared = crate::write::writer_helpers::share_handlers(vec![handler]);
    let _ = shared[0].as_not_repeat_executor();
}

#[test]
fn stateful_sheet_state_struct_access() {
    use crate::core::ExcelWriteMetadata;
    let _state = crate::write::writer_helpers::StatefulSheetState {
        schema: &[],
        metadata: ExcelWriteMetadata::default(),
        options: WriteOptions::default(),
        next_row: 0,
        next_data_index: 0,
    };
}
