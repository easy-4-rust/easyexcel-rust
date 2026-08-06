#[test]
fn stateful_sheet_state_construction() {
    use crate::core::ExcelWriteMetadata;
    use crate::write::metadata::write_sheet::WriteSheet as MirroredWriteSheet;
    let _sheet = MirroredWriteSheet::new();
    let options = WriteOptions::default();
    let metadata = ExcelWriteMetadata::default();
    let _state = StatefulSheetState {
        schema: &[],
        metadata,
        options,
        next_row: 0,
        next_data_index: 0,
    };
}

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
    let shared = share_handlers(vec![handler]);
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
    let shared = share_handlers(vec![handler]);
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
    let shared = share_handlers(vec![handler]);
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
    let shared = share_handlers(vec![handler]);
    let _ = shared[0].as_not_repeat_executor();
}

#[test]
fn stateful_sheet_state_struct_access() {
    use crate::core::ExcelWriteMetadata;
    let _state = StatefulSheetState {
        schema: &[],
        metadata: ExcelWriteMetadata::default(),
        options: WriteOptions::default(),
        next_row: 0,
        next_data_index: 0,
    };
}

