/// 对应 Java：无直接对应对象；Rust 架构扩展。 可复现运行所需的环境标识。
#[derive(Debug, Serialize)]
pub(crate) struct EnvironmentResult {
    pub(crate) git_sha: String,
    pub(crate) runtime: String,
    pub(crate) os: &'static str,
    pub(crate) arch: &'static str,
    pub(crate) spec_sha256: String,
}

