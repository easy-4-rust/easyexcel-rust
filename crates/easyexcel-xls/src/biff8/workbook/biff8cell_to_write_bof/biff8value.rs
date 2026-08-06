/// 对应 Java：无直接对应对象；Rust 架构扩展。 Logical cell payload (before SST / NUMBER framing).
#[derive(Debug, Clone)]
pub enum Biff8Value {
    /// Blank cell.
    Blank,
    /// Shared string (interned into SST on serialize).
    Text(String),
    /// IEEE754 number (also used for Excel date serials).
    Number(f64),
    /// Boolean.
    Bool(bool),
    /// Formula expression (without leading `=`), encoded as `BIFF8` `Ptg`
    /// tokens at serialization time. Cached result defaults to `0` so
    /// `Excel` / `LibreOffice` recalculate on load.
    Formula(String),
}

