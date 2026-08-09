//! 对应 Java：`com.alibaba.excel.read.metadata.holder.xlsx.XlsxReadWorkbookHolder`.

use crate::read::holder::read_workbook_holder::ReadWorkbookHolder;
use crate::read::metadata::holder::read_holder::delegate_read_holder_contract;
use std::collections::HashMap;
use crate::DataFormatData;
use std::ops::{Deref, DerefMut};

/// 对应 Java：`XlsxReadWorkbookHolder extends ReadWorkbookHolder`.
#[derive(Debug, Clone)]
pub struct XlsxReadWorkbookHolder {
    inner: ReadWorkbookHolder,
    data_format_data_cache: HashMap<i32, DataFormatData>,
    package_relationship_collection_map: HashMap<String, Vec<String>>,
    sax_parser_factory_name: Option<String>,
    opc_package: Option<Vec<u8>>,
    styles_table: Vec<DataFormatData>,
}

impl XlsxReadWorkbookHolder {
    /// 对应 Java：`XlsxReadWorkbookHolder(ReadWorkbook)`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: ReadWorkbookHolder::default(),
            data_format_data_cache: HashMap::new(),
            package_relationship_collection_map: HashMap::new(),
            sax_parser_factory_name: None,
            opc_package: None,
            styles_table: Vec::new(),
        }
    }

    /// 对应 Java：com.alibaba.excel.read.metadata.holder.xlsx.XlsxReadWorkbookHolder。 Creates the format-specific holder from resolved workbook options.
    #[must_use]
    pub fn from_options(options: &crate::ReadOptions) -> Self {
        Self {
            inner: ReadWorkbookHolder::from_options(options),
            data_format_data_cache: HashMap::new(),
            package_relationship_collection_map: HashMap::new(),
            sax_parser_factory_name: options.xlsx_sax_parser_factory_name.clone(),
            opc_package: None,
            styles_table: Vec::new(),
        }
    }

    /// Java `XlsxReadWorkbookHolder(ReadWorkbook)`。
    #[must_use]
    pub fn from_read_workbook(value: crate::ReadWorkbook) -> Self {
        let sax_parser_factory_name = value.get_xlsx_sax_parser_factory_name().map(str::to_owned);
        let mut holder = Self::new();
        holder.inner = ReadWorkbookHolder::from_read_workbook(value);
        holder.sax_parser_factory_name = sax_parser_factory_name;
        holder
    }

    /// Returns the inner holder.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.read.metadata.holder.xlsx.XlsxReadWorkbookHolder。
    pub const fn inner(&self) -> &ReadWorkbookHolder {
        &self.inner
    }
    pub const fn inner_mut(&mut self) -> &mut ReadWorkbookHolder { &mut self.inner }
    #[must_use] pub const fn get_data_format_data_cache(&self) -> &HashMap<i32, DataFormatData> {
        &self.data_format_data_cache
    }
    pub fn set_data_format_data_cache(&mut self, value: HashMap<i32, DataFormatData>) {
        self.data_format_data_cache = value;
    }
    pub fn data_format_data(&mut self, index: i32) -> &DataFormatData {
        self.data_format_data_cache.entry(index).or_insert_with(|| {
            let mut value = DataFormatData::new();
            value.set_index(i16::try_from(index).ok());
            value
        })
    }
    #[must_use] pub const fn get_package_relationship_collection_map(&self) -> &HashMap<String, Vec<String>> {
        &self.package_relationship_collection_map
    }
    pub fn set_package_relationship_collection_map(&mut self, value: HashMap<String, Vec<String>>) {
        self.package_relationship_collection_map = value;
    }
    #[must_use] pub fn get_sax_parser_factory_name(&self) -> Option<&str> {
        self.sax_parser_factory_name.as_deref()
    }
    pub fn set_sax_parser_factory_name(&mut self, value: Option<String>) {
        self.sax_parser_factory_name = value;
    }
    #[must_use] pub fn get_opc_package(&self) -> Option<&[u8]> { self.opc_package.as_deref() }
    pub fn set_opc_package(&mut self, value: Option<Vec<u8>>) { self.opc_package = value; }
    #[must_use] pub fn get_styles_table(&self) -> &[DataFormatData] { &self.styles_table }
    pub fn set_styles_table(&mut self, value: Vec<DataFormatData>) { self.styles_table = value; }
}

impl Deref for XlsxReadWorkbookHolder {
    type Target = ReadWorkbookHolder;
    fn deref(&self) -> &Self::Target { &self.inner }
}
impl DerefMut for XlsxReadWorkbookHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

delegate_read_holder_contract!(XlsxReadWorkbookHolder, inner);

impl Default for XlsxReadWorkbookHolder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xlsx_holder_constructors_and_inner_access() {
        // 对应 Java：XlsxReadWorkbookHolder 构造与 inner 访问器
        let holder = XlsxReadWorkbookHolder::new();
        assert!(
            !holder.inner().ignore_empty_row,
            "derive Default 初始为 false"
        );

        let options = crate::ReadOptions {
            ignore_empty_row: false,
            ..crate::ReadOptions::default()
        };
        let from_options = XlsxReadWorkbookHolder::from_options(&options);
        assert!(!from_options.inner().ignore_empty_row);
        assert_eq!(from_options.inner().charset, options.charset);
        let default_from_options =
            XlsxReadWorkbookHolder::from_options(&crate::ReadOptions::default());
        assert!(default_from_options.inner().ignore_empty_row);

        let defaulted = XlsxReadWorkbookHolder::default();
        assert!(defaulted.inner().auto_close_stream);
    }
}
