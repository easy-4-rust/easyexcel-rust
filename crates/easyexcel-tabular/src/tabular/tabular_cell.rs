use easyexcel_model::CellValue;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 中立表格中的一个单元格。
#[derive(Debug, Clone, PartialEq)]
pub struct TabularCell {
    value: CellValue,
    header: bool,
}

impl TabularCell {
    /// 创建普通单元格。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(value: CellValue) -> Self {
        Self {
            value,
            header: false,
        }
    }

    /// 创建表头单元格。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn header(value: CellValue) -> Self {
        Self {
            value,
            header: true,
        }
    }

    /// 返回单元格值。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn value(&self) -> &CellValue {
        &self.value
    }

    /// 返回该单元格是否来自表头。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn is_header(&self) -> bool {
        self.header
    }
}
