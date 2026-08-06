//! 共享性能契约的反序列化类型。

use serde::Deserialize;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Java 与 Rust 共用的性能测试契约。
#[derive(Debug, Deserialize)]
pub(crate) struct BenchmarkSpec {
    pub(crate) schema_version: u32,
    pub(crate) suite_id: String,
    pub(crate) batch_size: usize,
    pub(crate) scenarios: Vec<ScenarioSpec>,
}

impl BenchmarkSpec {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按稳定标识查找场景。
    pub(crate) fn scenario(&self, id: &str) -> Option<&ScenarioSpec> {
        self.scenarios.iter().find(|scenario| scenario.id == id)
    }
}

include!("benchmark_spec/scenario_spec.rs");
