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

#[cfg(test)]
mod generated_cell_value_tests {
    use super::*;

    #[test]
    fn blank_into_cell() {
        let cell = GeneratedBiff8CellValue::Blank.into_cell();
        assert!(matches!(cell.value, Biff8Value::Blank));
        assert_eq!(cell.xf, XF_GENERAL);
    }

    #[test]
    fn text_into_cell() {
        let cell = GeneratedBiff8CellValue::Text("hello".to_owned()).into_cell();
        match &cell.value {
            Biff8Value::Text(s) => assert_eq!(s, "hello"),
            _ => panic!("expected Text"),
        }
        assert_eq!(cell.xf, XF_GENERAL);
    }

    #[test]
    fn rich_text_into_cell() {
        let cell = GeneratedBiff8CellValue::RichText {
            text: "bold".to_owned(),
            runs: vec![(0, 1)],
        }
        .into_cell();
        match &cell.value {
            Biff8Value::RichText(rt) => assert_eq!(rt.text, "bold"),
            _ => panic!("expected RichText"),
        }
        assert_eq!(cell.xf, XF_GENERAL);
    }

    #[test]
    fn number_into_cell() {
        let cell = GeneratedBiff8CellValue::Number(42.5).into_cell();
        match cell.value {
            Biff8Value::Number(v) => assert!((v - 42.5).abs() < f64::EPSILON),
            _ => panic!("expected Number"),
        }
        assert_eq!(cell.xf, XF_GENERAL);
    }

    #[test]
    fn bool_into_cell() {
        let cell = GeneratedBiff8CellValue::Bool(true).into_cell();
        match cell.value {
            Biff8Value::Bool(v) => assert!(v),
            _ => panic!("expected Bool"),
        }
        assert_eq!(cell.xf, XF_GENERAL);
    }

    #[test]
    fn formula_into_cell() {
        let cell = GeneratedBiff8CellValue::Formula("SUM(A1:A10)".to_owned()).into_cell();
        match &cell.value {
            Biff8Value::Formula(f) => assert_eq!(f, "SUM(A1:A10)"),
            _ => panic!("expected Formula"),
        }
        assert_eq!(cell.xf, XF_GENERAL);
    }

    #[test]
    fn date_serial_into_cell() {
        let cell = GeneratedBiff8CellValue::DateSerial(44927.0).into_cell();
        match cell.value {
            Biff8Value::Number(v) => assert!((v - 44927.0).abs() < f64::EPSILON),
            _ => panic!("expected Number for date"),
        }
        assert_eq!(cell.xf, XF_DATE);
    }

    #[test]
    fn datetime_serial_into_cell() {
        let cell = GeneratedBiff8CellValue::DateTimeSerial(44927.5).into_cell();
        match cell.value {
            Biff8Value::Number(v) => assert!((v - 44927.5).abs() < f64::EPSILON),
            _ => panic!("expected Number for datetime"),
        }
        assert_eq!(cell.xf, XF_DATETIME);
    }

    #[test]
    fn debug_format() {
        let value = GeneratedBiff8CellValue::Blank;
        let debug = format!("{value:?}");
        assert!(debug.contains("Blank"));

        let value = GeneratedBiff8CellValue::Text("test".to_owned());
        let debug = format!("{value:?}");
        assert!(debug.contains("Text"));
    }

    #[test]
    fn clone_works() {
        let value = GeneratedBiff8CellValue::Number(1.0);
        let cloned = value.clone();
        let cell = cloned.into_cell();
        match cell.value {
            Biff8Value::Number(v) => assert!((v - 1.0).abs() < f64::EPSILON),
            _ => panic!("expected Number"),
        }
    }
}
