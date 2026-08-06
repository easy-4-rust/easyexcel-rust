/// 对应 Java：无直接对应对象；Rust 架构扩展。 FORMULA 记录中的缓存值。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Biff8FormulaCachedValue {
    /// 后续 STRING 记录承载文本。
    String,
    /// 数字缓存值。
    Number(f64),
    /// 布尔缓存值。
    Boolean(bool),
    /// 错误缓存值。
    Error,
    /// 空值或未定义缓存类型。
    Empty,
}

