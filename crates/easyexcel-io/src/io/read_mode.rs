/// 工作簿读取模式。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ReadMode {
    /// 按行推送事件，适合大文件。
    #[default]
    Event,
    /// 将完整工作簿加载到内存，适合查询和修改。
    Workbook,
}
