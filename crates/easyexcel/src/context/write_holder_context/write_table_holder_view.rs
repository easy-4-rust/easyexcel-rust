/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read-only runtime view of Java `WriteTableHolder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteTableHolderView {
    table_no: i32,
    parent_sheet_name: String,
}

impl WriteTableHolderView {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a view for the active table and its parent sheet.
    #[must_use]
    pub fn new(table_no: i32, parent_sheet_name: impl Into<String>) -> Self {
        Self {
            table_no,
            parent_sheet_name: parent_sheet_name.into(),
        }
    }

    /// Returns the zero-based table number. (Java `WriteTableHolder.getTableNo()`)
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn table_no(&self) -> i32 {
        self.table_no
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the parent worksheet name.
    #[must_use]
    pub fn parent_sheet_name(&self) -> &str {
        &self.parent_sheet_name
    }
}

