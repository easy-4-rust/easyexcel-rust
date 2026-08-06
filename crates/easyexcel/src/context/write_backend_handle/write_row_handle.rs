/// 对应 Java：无直接对应对象；Rust 架构扩展。 Backend-neutral equivalent of POI's mutable `Row` callback object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteRowHandle {
    row_index: u32,
    requested_height: RefCell<Option<u16>>,
}

impl WriteRowHandle {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a handle for one physical row.
    #[must_use]
    pub fn new(row_index: u32) -> Self {
        Self {
            row_index,
            requested_height: RefCell::new(None),
        }
    }

    /// Returns the zero-based physical row.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Requests a final row height in points.
    pub fn set_height(&self, height: u16) {
        *self.requested_height.borrow_mut() = Some(height);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the requested final row height.
    #[must_use]
    pub fn requested_height(&self) -> Option<u16> {
        *self.requested_height.borrow()
    }
}

