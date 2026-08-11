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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_all_some_values() {
        let key =
            UniqueDataFlagKey::new(Some(1), Some("Sheet1".to_owned()), Some("list".to_owned()));
        assert_eq!(key.get_sheet_no(), Some(1));
        assert_eq!(key.get_sheet_name(), Some("Sheet1"));
        assert_eq!(key.get_wrapper_name(), Some("list"));
    }

    #[test]
    fn new_with_all_none_values() {
        let key = UniqueDataFlagKey::new(None, None, None);
        assert!(key.get_sheet_no().is_none());
        assert!(key.get_sheet_name().is_none());
        assert!(key.get_wrapper_name().is_none());
    }

    #[test]
    fn setters_mutable() {
        let mut key = UniqueDataFlagKey::default();
        key.set_sheet_no(Some(42));
        assert_eq!(key.get_sheet_no(), Some(42));
        key.set_sheet_name(Some("Data".to_owned()));
        assert_eq!(key.get_sheet_name(), Some("Data"));
        key.set_wrapper_name(Some("scalar".to_owned()));
        assert_eq!(key.get_wrapper_name(), Some("scalar"));
        // 重置为 None
        key.set_sheet_no(None);
        assert!(key.get_sheet_no().is_none());
    }

    #[test]
    fn equality_and_hash() {
        let a = UniqueDataFlagKey::new(Some(1), Some("S".to_owned()), None);
        let b = UniqueDataFlagKey::new(Some(1), Some("S".to_owned()), None);
        assert_eq!(a, b);
        let c = UniqueDataFlagKey::new(Some(2), Some("S".to_owned()), None);
        assert_ne!(a, c);
    }

    #[test]
    fn default_is_all_none() {
        let key = UniqueDataFlagKey::default();
        assert_eq!(key, UniqueDataFlagKey::new(None, None, None));
    }

    #[test]
    fn debug_fmt() {
        let key = UniqueDataFlagKey::new(Some(0), Some("S".to_owned()), Some("w".to_owned()));
        let text = format!("{:?}", key);
        assert!(text.contains("UniqueDataFlagKey"));
    }
}
