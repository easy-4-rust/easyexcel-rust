//! 对应 Java：`com.alibaba.excel.metadata.property.ColumnWidthProperty`.

/// 对应 Java：`ColumnWidthProperty`. (Java `width: Integer`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnWidthProperty {
    /// Column width in Excel character units. (Java `getWidth()`)
    pub width: u16,
}

impl ColumnWidthProperty {
    /// Creates a `ColumnWidthProperty`. (Java constructor)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.ColumnWidthProperty。
    pub const fn new(width: u16) -> Self {
        Self { width }
    }
    /// Returns the width. (Java `getWidth()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.ColumnWidthProperty。
    pub const fn width(&self) -> u16 {
        self.width
    }
    #[must_use] pub const fn get_width(&self) -> u16 { self.width() }
    pub const fn set_width(&mut self, value: u16) { self.width = value; }
}
