//! 对应 Java：`com.alibaba.excel.metadata.property.RowHeightProperty`.

/// 对应 Java：`RowHeightProperty`. (Java `height: Short`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowHeightProperty {
    /// Row height in points. (Java `getHeight()`)
    pub height: u16,
}

impl RowHeightProperty {
    /// Creates a `RowHeightProperty`. (Java constructor)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.RowHeightProperty。
    pub const fn new(height: u16) -> Self {
        Self { height }
    }
    /// Returns the height. (Java `getHeight()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.RowHeightProperty。
    pub const fn height(&self) -> u16 {
        self.height
    }
    /// Java `getHeight` 别名。运行期属性只保存已经通过注解 sentinel 校验的非负高度。
    #[must_use]
    pub const fn get_height(&self) -> u16 { self.height() }
    /// Java `setHeight` 的非空运行期映射。
    pub const fn set_height(&mut self, value: u16) { self.height = value; }
}
