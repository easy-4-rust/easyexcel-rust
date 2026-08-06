/// 对应 Java：无直接对应对象；Rust 架构扩展。 Value accepted by [`TemplateData`] placeholder insertion methods.
pub trait IntoTemplateValue {
    /// Converts the value to its typed template representation.
    fn into_template_value(self) -> CellValue;
}

impl IntoTemplateValue for CellValue {
    fn into_template_value(self) -> CellValue {
        self
    }
}

impl IntoTemplateValue for String {
    fn into_template_value(self) -> CellValue {
        CellValue::String(self)
    }
}

impl IntoTemplateValue for &str {
    fn into_template_value(self) -> CellValue {
        CellValue::String(self.to_owned())
    }
}

impl IntoTemplateValue for &String {
    fn into_template_value(self) -> CellValue {
        CellValue::String(self.clone())
    }
}

impl IntoTemplateValue for bool {
    fn into_template_value(self) -> CellValue {
        CellValue::Bool(self)
    }
}

impl IntoTemplateValue for isize {
    fn into_template_value(self) -> CellValue {
        CellValue::Int(i64::try_from(self).expect("Rust isize is at most 64 bits"))
    }
}

impl IntoTemplateValue for usize {
    fn into_template_value(self) -> CellValue {
        CellValue::Decimal(BigDecimal::from(
            u64::try_from(self).expect("Rust usize is at most 64 bits"),
        ))
    }
}

impl IntoTemplateValue for BigInt {
    fn into_template_value(self) -> CellValue {
        CellValue::Decimal(BigDecimal::from(self))
    }
}

impl IntoTemplateValue for f32 {
    fn into_template_value(self) -> CellValue {
        CellValue::Float(f64::from(self))
    }
}

impl IntoTemplateValue for f64 {
    fn into_template_value(self) -> CellValue {
        CellValue::Float(self)
    }
}

impl IntoTemplateValue for BigDecimal {
    fn into_template_value(self) -> CellValue {
        CellValue::Decimal(self)
    }
}

impl IntoTemplateValue for NaiveDate {
    fn into_template_value(self) -> CellValue {
        CellValue::Date(self)
    }
}

impl IntoTemplateValue for NaiveDateTime {
    fn into_template_value(self) -> CellValue {
        CellValue::DateTime(self)
    }
}

impl<T> IntoTemplateValue for Option<T>
where
    T: IntoTemplateValue,
{
    fn into_template_value(self) -> CellValue {
        self.map_or(CellValue::Empty, IntoTemplateValue::into_template_value)
    }
}

