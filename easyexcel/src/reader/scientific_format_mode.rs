//! Scientific-notation rendering mode for extreme General-format numbers.

/// Controls how General-format extreme numbers are displayed while reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScientificFormatMode {
    /// Match Java `EasyExcel`'s default and avoid scientific notation.
    #[default]
    Plain,
    /// Use Java `EasyExcel`'s `0.#####E0` scientific representation.
    Scientific,
}

impl ScientificFormatMode {
    pub(crate) const fn is_enabled(self) -> bool {
        matches!(self, Self::Scientific)
    }
}
