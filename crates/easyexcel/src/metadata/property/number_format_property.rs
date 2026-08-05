//! 对应 Java：`com.alibaba.excel.metadata.property.NumberFormatProperty`.

pub use easyexcel_format::NumberRoundingMode;

/// Number format metadata from `@NumberFormat`.
///
/// Rust port of Java `NumberFormatProperty`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFormatProperty {
    /// Format pattern. (Java `format`)
    pub format: String,
    /// Rounding mode. (Java `roundingMode`)
    pub rounding_mode: NumberRoundingMode,
}

impl NumberFormatProperty {
    /// Creates a number format property. (Java constructor)
    #[must_use]
    pub fn new(format: impl Into<String>, rounding_mode: impl Into<NumberRoundingMode>) -> Self {
        Self {
            format: format.into(),
            rounding_mode: rounding_mode.into(),
        }
    }

    /// Builds from annotation values. (Java `build(NumberFormat)`)
    #[must_use]
    pub fn build(format: Option<&str>, rounding_mode: Option<NumberRoundingMode>) -> Option<Self> {
        format.map(|format| Self {
            format: format.to_owned(),
            rounding_mode: rounding_mode.unwrap_or_default(),
        })
    }

    /// Returns the format pattern. (Java `getFormat()`)
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Returns the rounding mode. (Java `getRoundingMode()`)
    #[must_use]
    pub const fn rounding_mode(&self) -> NumberRoundingMode {
        self.rounding_mode
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use bigdecimal::RoundingMode;

    #[test]
    fn bigdecimal_mapping_covers_every_mode() {
        // 对应 Java：RoundingMode 与 NumberFormatProperty.roundingMode 双向映射
        assert_eq!(NumberRoundingMode::Up.bigdecimal(), Some(RoundingMode::Up));
        assert_eq!(
            NumberRoundingMode::Down.bigdecimal(),
            Some(RoundingMode::Down)
        );
        assert_eq!(
            NumberRoundingMode::Ceiling.bigdecimal(),
            Some(RoundingMode::Ceiling)
        );
        assert_eq!(
            NumberRoundingMode::Floor.bigdecimal(),
            Some(RoundingMode::Floor)
        );
        assert_eq!(
            NumberRoundingMode::HalfUp.bigdecimal(),
            Some(RoundingMode::HalfUp)
        );
        assert_eq!(
            NumberRoundingMode::HalfDown.bigdecimal(),
            Some(RoundingMode::HalfDown)
        );
        assert_eq!(
            NumberRoundingMode::HalfEven.bigdecimal(),
            Some(RoundingMode::HalfEven)
        );
        assert_eq!(NumberRoundingMode::Unnecessary.bigdecimal(), None);

        for (mode, bigdecimal_mode) in [
            (NumberRoundingMode::Up, RoundingMode::Up),
            (NumberRoundingMode::Down, RoundingMode::Down),
            (NumberRoundingMode::Ceiling, RoundingMode::Ceiling),
            (NumberRoundingMode::Floor, RoundingMode::Floor),
            (NumberRoundingMode::HalfUp, RoundingMode::HalfUp),
            (NumberRoundingMode::HalfDown, RoundingMode::HalfDown),
            (NumberRoundingMode::HalfEven, RoundingMode::HalfEven),
        ] {
            assert_eq!(NumberRoundingMode::from(bigdecimal_mode), mode);
        }
        // 默认模式
        assert_eq!(NumberRoundingMode::default(), NumberRoundingMode::HalfUp);
    }

    #[test]
    fn new_build_and_accessors() {
        // 对应 Java：NumberFormatProperty 构造与 getter
        let property = NumberFormatProperty::new("0.00", NumberRoundingMode::HalfDown);
        assert_eq!(property.format(), "0.00");
        assert_eq!(property.rounding_mode(), NumberRoundingMode::HalfDown);
        assert_eq!(property.format, "0.00");

        // build 未指定格式返回 None
        assert!(NumberFormatProperty::build(None, Some(NumberRoundingMode::Up)).is_none());
        // build 未指定舍入模式使用默认
        let built = NumberFormatProperty::build(Some("0"), None).expect("built");
        assert_eq!(built.rounding_mode(), NumberRoundingMode::default());
        let built = NumberFormatProperty::build(Some("0.0"), Some(NumberRoundingMode::Floor))
            .expect("built");
        assert_eq!(built.rounding_mode(), NumberRoundingMode::Floor);
        assert_eq!(built.format(), "0.0");
    }
}
