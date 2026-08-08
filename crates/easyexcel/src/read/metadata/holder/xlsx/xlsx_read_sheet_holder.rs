//! 对应 Java：`com.alibaba.excel.read.metadata.holder.xlsx.XlsxReadSheetHolder`.

use crate::read::holder::read_sheet_holder::ReadSheetHolder;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

/// 对应 Java：`XlsxReadSheetHolder extends ReadSheetHolder`.
#[derive(Debug, Clone)]
pub struct XlsxReadSheetHolder {
    inner: ReadSheetHolder,
    column_index: Option<i32>,
    tag_deque: VecDeque<String>,
    temp_data: String,
    temp_formula: String,
    package_relationship_collection: Vec<String>,
}

impl Deref for XlsxReadSheetHolder {
    type Target = ReadSheetHolder;
    fn deref(&self) -> &Self::Target { &self.inner }
}
impl DerefMut for XlsxReadSheetHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl XlsxReadSheetHolder {
    /// 对应 Java： constructor.
    pub fn new(sheet_no: i32, sheet_name: impl Into<String>) -> Self {
        Self {
            inner: ReadSheetHolder::new(sheet_no, sheet_name),
            column_index: None,
            tag_deque: VecDeque::new(),
            temp_data: String::new(),
            temp_formula: String::new(),
            package_relationship_collection: Vec::new(),
        }
    }
    /// Returns the inner holder.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.holder.xlsx.XlsxReadSheetHolder。
    pub const fn inner(&self) -> &ReadSheetHolder {
        &self.inner
    }
    pub const fn inner_mut(&mut self) -> &mut ReadSheetHolder { &mut self.inner }
    #[must_use] pub const fn get_column_index(&self) -> Option<i32> { self.column_index }
    pub const fn set_column_index(&mut self, value: Option<i32>) { self.column_index = value; }
    #[must_use] pub const fn get_tag_deque(&self) -> &VecDeque<String> { &self.tag_deque }
    pub fn set_tag_deque(&mut self, value: VecDeque<String>) { self.tag_deque = value; }
    #[must_use] pub fn get_temp_data(&self) -> &str { &self.temp_data }
    pub fn set_temp_data(&mut self, value: impl Into<String>) { self.temp_data = value.into(); }
    #[must_use] pub fn get_temp_formula(&self) -> &str { &self.temp_formula }
    pub fn set_temp_formula(&mut self, value: impl Into<String>) { self.temp_formula = value.into(); }
    #[must_use] pub fn get_package_relationship_collection(&self) -> &[String] {
        &self.package_relationship_collection
    }
    pub fn set_package_relationship_collection(&mut self, value: Vec<String>) {
        self.package_relationship_collection = value;
    }
}
