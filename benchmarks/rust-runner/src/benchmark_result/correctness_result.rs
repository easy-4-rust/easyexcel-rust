/// 对应 Java：无直接对应对象；Rust 架构扩展。 单次运行的可观察正确性结果。
#[derive(Debug, Serialize)]
pub(crate) struct CorrectnessResult {
    pub(crate) observed_rows: u64,
    pub(crate) checksum: String,
    pub(crate) rereadable: bool,
}

