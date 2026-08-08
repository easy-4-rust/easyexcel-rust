//! 对应 Java：`com.alibaba.excel.read.metadata.holder.xls.XlsReadSheetHolder`.

use crate::read::holder::read_sheet_holder::ReadSheetHolder;
use std::collections::HashMap;
use crate::RowTypeEnum;
use std::ops::{Deref, DerefMut};

/// 对应 Java：`XlsReadSheetHolder extends ReadSheetHolder`.
#[derive(Debug, Clone)]
pub struct XlsReadSheetHolder {
    inner: ReadSheetHolder,
    object_cache_map: HashMap<i32, String>,
    temp_object_index: Option<i32>,
    temp_row_type: Option<RowTypeEnum>,
}

impl Deref for XlsReadSheetHolder {
    type Target = ReadSheetHolder;
    fn deref(&self) -> &Self::Target { &self.inner }
}
impl DerefMut for XlsReadSheetHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl XlsReadSheetHolder {
    /// 对应 Java： constructor.
    pub fn new(sheet_no: i32, sheet_name: impl Into<String>) -> Self {
        Self {
            inner: ReadSheetHolder::new(sheet_no, sheet_name),
            object_cache_map: HashMap::new(),
            temp_object_index: None,
            temp_row_type: None,
        }
    }
    /// Returns the inner holder.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.holder.xls.XlsReadSheetHolder。
    pub const fn inner(&self) -> &ReadSheetHolder {
        &self.inner
    }
    #[must_use] pub const fn get_object_cache_map(&self) -> &HashMap<i32, String> { &self.object_cache_map }
    pub fn set_object_cache_map(&mut self, value: HashMap<i32, String>) { self.object_cache_map = value; }
    #[must_use] pub const fn get_temp_object_index(&self) -> Option<i32> { self.temp_object_index }
    pub const fn set_temp_object_index(&mut self, value: Option<i32>) { self.temp_object_index = value; }
    #[must_use] pub const fn get_temp_row_type(&self) -> Option<RowTypeEnum> { self.temp_row_type }
    pub const fn set_temp_row_type(&mut self, value: Option<RowTypeEnum>) { self.temp_row_type = value; }
}
