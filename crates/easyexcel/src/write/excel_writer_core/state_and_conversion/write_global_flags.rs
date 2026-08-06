/// 对应 Java：无直接对应对象；Rust 架构扩展。 Global write flags copied from [`WriteOptions`] for cell emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct WriteGlobalFlags {
    /// Automatic trim for sheet names and string cells.
    auto_trim: bool,
    /// Whether Excel 1904 date windowing is enabled.
    use_1904_windowing: bool,
    /// Whether scientific notation is used for extreme General-format numbers.
    use_scientific_format: bool,
}

impl From<&WriteOptions> for WriteGlobalFlags {
    fn from(options: &WriteOptions) -> Self {
        Self {
            auto_trim: options.auto_trim,
            use_1904_windowing: options.use_1904_windowing,
            use_scientific_format: options.use_scientific_format,
        }
    }
}

