/// BIFF8 写入后端可直接编译的中立单元格值。
///
/// 对应 Java：无直接对应对象；Rust XLS 引擎扩展。门面负责 converter、Handler 和
/// Java 元数据合并，本类型负责把最终逻辑值编译为 `Biff8Cell/Biff8Value`。
#[derive(Debug, Clone)]
pub enum GeneratedBiff8CellValue {
    /// 空白单元格。
    Blank,
    /// 文本单元格。
    Text(String),
    /// 带 BIFF8 FONT 索引区间的富文本。
    RichText {
        /// 完整显示文本。
        text: String,
        /// `(UTF-16 起始位置, FONT 索引)` 列表。
        runs: Vec<(u16, u16)>,
    },
    /// IEEE-754 数字单元格。
    Number(f64),
    /// 布尔单元格。
    Bool(bool),
    /// 不含前导等号的公式表达式。
    Formula(String),
    /// 已按工作簿日期窗换算的日期序列值。
    DateSerial(f64),
    /// 已按工作簿日期窗换算的日期时间序列值。
    DateTimeSerial(f64),
}

impl GeneratedBiff8CellValue {
    /// 编译为 BIFF8 物理单元格。
    #[must_use]
    pub fn into_cell(self) -> Biff8Cell {
        match self {
            Self::Blank => Biff8Cell::general(Biff8Value::Blank),
            Self::Text(value) => Biff8Cell::general(Biff8Value::Text(value)),
            Self::RichText { text, runs } => Biff8Cell::general(Biff8Value::RichText(
                Biff8RichText::new(text, runs),
            )),
            Self::Number(value) => Biff8Cell::general(Biff8Value::Number(value)),
            Self::Bool(value) => Biff8Cell::general(Biff8Value::Bool(value)),
            Self::Formula(value) => Biff8Cell::general(Biff8Value::Formula(value)),
            Self::DateSerial(value) => Biff8Cell::date_serial(value),
            Self::DateTimeSerial(value) => Biff8Cell::datetime_serial(value),
        }
    }
}
