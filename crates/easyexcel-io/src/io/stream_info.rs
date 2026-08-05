use easyexcel_model::DateSystem;

/// 流式读取开始前提供的工作表元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamInfo {
    /// 工作表名称。
    pub sheet_name: String,
    /// 工作簿日期系统。
    pub date_system: DateSystem,
}
