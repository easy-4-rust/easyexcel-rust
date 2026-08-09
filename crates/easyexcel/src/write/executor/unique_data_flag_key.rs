//! 对应 Java：`com.alibaba.excel.write.executor.ExcelWriteFillExecutor.UniqueDataFlagKey`。

/// 一次模板填充的数据域键。
///
/// Sheet 编号、Sheet 名和包装器名共同参与相等性与哈希计算，用于隔离不同模板数据域。
/// 对应 Java：`ExcelWriteFillExecutor.UniqueDataFlagKey`。
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct UniqueDataFlagKey {
    sheet_no: Option<i32>,
    sheet_name: Option<String>,
    wrapper_name: Option<String>,
}

impl UniqueDataFlagKey {
    /// 创建模板填充数据域键。
    #[must_use]
    pub fn new(
        sheet_no: Option<i32>,
        sheet_name: Option<String>,
        wrapper_name: Option<String>,
    ) -> Self {
        Self {
            sheet_no,
            sheet_name,
            wrapper_name,
        }
    }

    /// 返回 Sheet 编号。
    #[must_use]
    pub const fn get_sheet_no(&self) -> Option<i32> {
        self.sheet_no
    }

    /// 设置 Sheet 编号。
    pub const fn set_sheet_no(&mut self, value: Option<i32>) {
        self.sheet_no = value;
    }

    /// 返回 Sheet 名。
    #[must_use]
    pub fn get_sheet_name(&self) -> Option<&str> {
        self.sheet_name.as_deref()
    }

    /// 设置 Sheet 名。
    pub fn set_sheet_name(&mut self, value: Option<String>) {
        self.sheet_name = value;
    }

    /// 返回模板包装器名。
    #[must_use]
    pub fn get_wrapper_name(&self) -> Option<&str> {
        self.wrapper_name.as_deref()
    }

    /// 设置模板包装器名。
    pub fn set_wrapper_name(&mut self, value: Option<String>) {
        self.wrapper_name = value;
    }
}
