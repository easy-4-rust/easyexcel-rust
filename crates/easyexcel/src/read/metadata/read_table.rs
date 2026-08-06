//! 对应 Java：`com.alibaba.excel.read.metadata.ReadTable`.

/// 对应 Java：`ReadTable` — a thin marker carrying a single
/// `tableNo` field. Java has 4 members (tableNo, getTableNo,
/// setTableNo, equals/hashCode). Rust collapses the int field into
/// the `i32` representation that matches `ReadSheet.sheetNo`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadTable {
    /// Zero-based table index. (Java `ReadTable.tableNo`)
    pub table_no: i32,
}

impl ReadTable {
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadTable。 Creates a `ReadTable` with table no 0. (Java `new ReadTable()`)
    #[must_use]
    pub fn new() -> Self {
        Self { table_no: 0 }
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadTable。 Creates a `ReadTable` with the given table no.
    /// (Java `ReadTable(Integer tableNo)`)
    #[must_use]
    pub fn with_table_no(table_no: i32) -> Self {
        Self { table_no }
    }

    /// Returns the zero-based table index. (Java `getTableNo()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadTable。
    pub const fn table_no(&self) -> i32 {
        self.table_no
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadTable。 Sets the zero-based table index. (Java `setTableNo(Integer)`)
    pub fn set_table_no(&mut self, table_no: i32) -> &mut Self {
        self.table_no = table_no;
        self
    }
}
