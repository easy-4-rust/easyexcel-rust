/// 对应 Java：无直接对应对象；Rust 架构扩展。 单个可执行性能场景。
#[derive(Debug, Deserialize)]
pub(crate) struct ScenarioSpec {
    pub(crate) id: String,
    pub(crate) format: String,
    pub(crate) operation: String,
    pub(crate) mode: String,
    pub(crate) memory: String,
}

