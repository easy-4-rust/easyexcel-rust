/// A parsed criterion used by the `*IF`/`*IFS` family.
///
/// Excel criteria are a string like `">5"`, `"<=10"`, `"<>x"`, `"apple"`, or a
/// bare value. Text comparisons are case-insensitive and support `*`/`?`
/// wildcards.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct Criteria {
    op: CritOp,
    /// Numeric comparison target, if the criterion value parses as a number.
    num: Option<f64>,
    /// Text comparison target (lower-cased).
    text: String,
    /// Original (non-lowercased) text.
    raw: String,
}

impl Criteria {
    /// Build a criterion from a criteria value (number, bool or text).
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn parse(v: &Value) -> Criteria {
        let s = match v {
            Value::Number(n) => easyexcel_model::value::format_number_general(*n),
            Value::Bool(b) => {
                if *b {
                    "TRUE".into()
                } else {
                    "FALSE".into()
                }
            }
            Value::Text(s) => s.clone(),
            _ => String::new(),
        };
        let (op, rest) = if let Some(r) = s.strip_prefix("<=") {
            (CritOp::Le, r)
        } else if let Some(r) = s.strip_prefix(">=") {
            (CritOp::Ge, r)
        } else if let Some(r) = s.strip_prefix("<>") {
            (CritOp::Ne, r)
        } else if let Some(r) = s.strip_prefix('<') {
            (CritOp::Lt, r)
        } else if let Some(r) = s.strip_prefix('>') {
            (CritOp::Gt, r)
        } else if let Some(r) = s.strip_prefix('=') {
            (CritOp::Eq, r)
        } else {
            (CritOp::Eq, s.as_str())
        };
        let num = super::coerce::parse_number_text(rest);
        Criteria {
            op,
            num,
            text: rest.to_lowercase(),
            raw: rest.to_string(),
        }
    }

    /// Test a candidate value against the criterion.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn matches(&self, v: &Value) -> bool {
        // Numeric comparison when both sides are numeric.
        if let Some(target) = self.num {
            if let Some(n) = numeric_of(v) {
                return match self.op {
                    CritOp::Eq => n == target,
                    CritOp::Ne => n != target,
                    CritOp::Lt => n < target,
                    CritOp::Le => n <= target,
                    CritOp::Gt => n > target,
                    CritOp::Ge => n >= target,
                };
            }
            // criterion numeric but cell not numeric: only <> matches
            return self.op == CritOp::Ne;
        }
        // Text comparison.
        let cell_text = match v {
            Value::Text(s) => s.to_lowercase(),
            Value::Bool(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Value::Empty => String::new(),
            Value::Number(n) => easyexcel_model::value::format_number_general(*n).to_lowercase(),
            _ => return false,
        };
        match self.op {
            CritOp::Eq => {
                if self.raw.is_empty() {
                    matches!(v, Value::Empty)
                } else {
                    wildcard_match(&self.text, &cell_text)
                }
            }
            CritOp::Ne => !wildcard_match(&self.text, &cell_text),
            CritOp::Lt => cell_text < self.text,
            CritOp::Le => cell_text <= self.text,
            CritOp::Gt => cell_text > self.text,
            CritOp::Ge => cell_text >= self.text,
        }
    }
}

