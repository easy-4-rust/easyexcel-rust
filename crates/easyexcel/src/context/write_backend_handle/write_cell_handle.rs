/// 对应 Java：无直接对应对象；Rust 架构扩展。 Backend-neutral equivalent of POI's mutable `Cell` callback object.
///
/// Mutations are recorded and committed by the active writer backend after
/// the logical callback chain. This never pretends to be an Apache POI cell.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteCellHandle {
    row_index: u32,
    column_index: u16,
    current_value: RefCell<CellValue>,
    requested_value: RefCell<Option<CellValue>>,
    requested_style: RefCell<Option<ExcelCellStyle>>,
    requested_skip: RefCell<Option<bool>>,
}

impl WriteCellHandle {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a handle for one physical cell.
    #[must_use]
    pub fn new(row_index: u32, column_index: u16, initial_value: CellValue) -> Self {
        Self {
            row_index,
            column_index,
            current_value: RefCell::new(initial_value),
            requested_value: RefCell::new(None),
            requested_style: RefCell::new(None),
            requested_skip: RefCell::new(None),
        }
    }

    /// Returns the zero-based physical row.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }

    /// Returns the zero-based physical column.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn column_index(&self) -> u16 {
        self.column_index
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the latest logical value visible to the callback chain.
    #[must_use]
    pub fn value(&self) -> CellValue {
        self.current_value.borrow().clone()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Requests a final cell value, including from `afterCellDispose`.
    pub fn set_value(&self, value: CellValue) {
        *self.current_value.borrow_mut() = value.clone();
        *self.requested_value.borrow_mut() = Some(value);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Synchronizes a value changed through the compatibility context field.
    ///
    /// 值未变化时跳过克隆：热路径上每单元格回调链结束时 `current_value` 与
    /// `context.value` 通常已经一致（构造值原样保留，或 `set_value` 已同时写入
    /// 两处），此时无条件克隆会为每个单元格额外分配一次 String；此处先比较再
    /// 决定是否需要克隆，语义与旧实现完全一致。
    pub fn sync_value(&self, value: &CellValue) {
        let mut current = self.current_value.borrow_mut();
        if *current != *value {
            *current = value.clone();
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Requests a final backend-neutral cell style.
    pub fn set_style(&self, style: ExcelCellStyle) {
        *self.requested_style.borrow_mut() = Some(style);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Requests that the physical cell be omitted or restored.
    pub fn set_skipped(&self, skipped: bool) {
        *self.requested_skip.borrow_mut() = Some(skipped);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the requested value override.
    #[must_use]
    pub fn requested_value(&self) -> Option<CellValue> {
        self.requested_value.borrow().clone()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the requested style override.
    #[must_use]
    pub fn requested_style(&self) -> Option<ExcelCellStyle> {
        *self.requested_style.borrow()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the requested skip override.
    #[must_use]
    pub fn requested_skip(&self) -> Option<bool> {
        *self.requested_skip.borrow()
    }
}

