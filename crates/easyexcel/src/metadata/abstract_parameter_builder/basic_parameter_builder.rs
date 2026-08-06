/// 对应 Java：无直接对应对象；Rust 架构扩展。 Minimal builder used by metadata tests and future reader/writer facades.
#[derive(Debug, Clone, Default)]
pub struct BasicParameterBuilder {
    parameter: BasicParameter,
}

impl BasicParameterBuilder {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates an empty builder. (Java builder entry point)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Builds the parameter bag. (Java `build()` parameter extraction)
    #[must_use]
    pub fn build(self) -> BasicParameter {
        self.parameter
    }
}

impl AbstractParameterBuilder for BasicParameterBuilder {
    fn parameter(&mut self) -> &mut BasicParameter {
        &mut self.parameter
    }
}

