/// 对应 Java：无直接对应对象；Rust 架构扩展。 Logical cell payload (before SST / NUMBER framing).
#[derive(Debug, Clone)]
pub enum Biff8Value {
    /// Blank cell.
    Blank,
    /// Shared string (interned into SST on serialize).
    Text(String),
    /// 带 FONT run 的共享字符串。
    RichText(Biff8RichText),
    /// IEEE754 number (also used for Excel date serials).
    Number(f64),
    /// Boolean.
    Bool(bool),
    /// BIFF8 `BOOLERR` 错误码。
    Error(u8),
    /// Formula expression (without leading `=`), encoded as `BIFF8` `Ptg`
    /// tokens at serialization time. Cached result defaults to `0` so
    /// `Excel` / `LibreOffice` recalculate on load.
    Formula(String),
}
