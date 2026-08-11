//! 对应 Java：`com.alibaba.excel.read.metadata.ReadBasicParameter`
//!
//! Java 的 `ReadBasicParameter` 包含 `headRowNumber` 和 `customReadListenerList`。
//! Rust 版本中，`headRowNumber` 已合并到 `ReadOptions`（`read/read_options.rs`），
//! `customReadListenerList` 通过 `ExcelReaderBuilder` 的泛型 listener 参数实现。
//! 本文件保留类型定义以满足 1:1 文件对应要求。

use crate::metadata::BasicParameter;
use crate::read::ReadOptions;

/// 对应 Java：`ReadBasicParameter extends BasicParameter`
///
/// 读取基本参数，包含表头行数和自定义监听器列表（对应 Java `ReadBasicParameter`）。
/// 实际字段已分散到 `ReadOptions` 和 builder 中。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadBasicParameter {
    /// Java 父类 `BasicParameter` 的完整字段。
    pub basic_parameter: BasicParameter,
    /// 表头行数，默认 1。对应 Java `headRowNumber`。
    pub head_row_number: u32,
    /// Java 自定义监听器列表的后端中立注册名。
    pub custom_read_listener_list: Vec<String>,
}

impl ReadBasicParameter {
    /// 对应 Java：com.alibaba.excel.read.metadata.ReadBasicParameter。 创建默认参数。
    #[must_use]
    pub fn new() -> Self {
        Self {
            basic_parameter: BasicParameter::default(),
            head_row_number: 1,
            custom_read_listener_list: Vec::new(),
        }
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.ReadBasicParameter。 从 `ReadOptions` 构造。
    #[must_use]
    pub fn from_options(options: &ReadOptions) -> Self {
        Self {
            basic_parameter: BasicParameter {
                auto_trim: Some(options.auto_trim),
                use1904windowing: Some(options.use_1904_windowing),
                use_scientific_format: Some(options.scientific_format.is_enabled()),
                ..BasicParameter::default()
            },
            head_row_number: options.head_row_number,
            custom_read_listener_list: Vec::new(),
        }
    }

    /// Java `getHeadRowNumber`。
    #[must_use]
    pub const fn get_head_row_number(&self) -> u32 {
        self.head_row_number
    }
    /// Java `setHeadRowNumber`。
    pub const fn set_head_row_number(&mut self, value: u32) {
        self.head_row_number = value;
    }
    /// Java `getCustomReadListenerList` 的后端中立视图。
    #[must_use]
    pub fn get_custom_read_listener_list(&self) -> &[String] {
        &self.custom_read_listener_list
    }
    /// Java `setCustomReadListenerList` 的后端中立映射。
    pub fn set_custom_read_listener_list(&mut self, value: Vec<String>) {
        self.custom_read_listener_list = value;
    }
    /// 返回父级基础参数。
    #[must_use]
    pub const fn get_basic_parameter(&self) -> &BasicParameter {
        &self.basic_parameter
    }
    /// 返回可变父级基础参数。
    pub const fn get_basic_parameter_mut(&mut self) -> &mut BasicParameter {
        &mut self.basic_parameter
    }
}
