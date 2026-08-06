/// 对应 Java：无直接对应对象；Rust 架构扩展。 工作簿写入模式。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum WriteMode {
    /// 生成新文件。
    #[default]
    Generate,
    /// 修改现有文件并尽量保留未知部件。
    RoundTrip,
}
