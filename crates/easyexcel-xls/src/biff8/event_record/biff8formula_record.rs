/// 对应 Java：无直接对应对象；Rust 架构扩展。 BIFF8 FORMULA 记录。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Biff8FormulaRecord {
    /// 单元格公共头。
    pub header: Biff8CellHeader,
    /// 公式缓存值。
    pub cached_value: Biff8FormulaCachedValue,
}

