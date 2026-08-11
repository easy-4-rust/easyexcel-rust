//! CSV 逻辑工作簿模型。

use easyexcel_io::{Error, Result};
use easyexcel_model::CellValue as ModelCellValue;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{CsvCellStyle, CsvCellValue, CsvCharset, CsvDataFormat, CsvSheet};

static NEXT_CSV_WORKBOOK_ID: AtomicUsize = AtomicUsize::new(1);

/// 对应 Java：com.alibaba.excel.metadata.csv.CsvWorkbook。 CSV 输出的单工作表逻辑工作簿。
#[derive(Debug, Clone, PartialEq)]
pub struct CsvWorkbook<V: CsvCellValue = ModelCellValue> {
    identity: usize,
    out: String,
    locale: String,
    use_1904_windowing: bool,
    use_scientific_format: bool,
    charset: CsvCharset,
    with_bom: bool,
    sheet: Option<CsvSheet<V>>,
    data_format: CsvDataFormat,
    cell_styles: Vec<CsvCellStyle>,
}

impl<V: CsvCellValue> CsvWorkbook<V> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用全局渲染选项创建工作簿。
    #[must_use]
    pub fn new(
        locale: impl Into<String>,
        use_1904_windowing: bool,
        use_scientific_format: bool,
        charset: CsvCharset,
        with_bom: bool,
    ) -> Self {
        Self {
            identity: NEXT_CSV_WORKBOOK_ID.fetch_add(1, Ordering::Relaxed),
            out: String::new(),
            locale: locale.into(),
            use_1904_windowing,
            use_scientific_format,
            charset,
            with_bom,
            sheet: None,
            data_format: CsvDataFormat::new(),
            cell_styles: Vec::new(),
        }
    }

    /// 返回工作簿稳定身份，供组合模型替代 Java 父对象引用。
    #[must_use]
    pub const fn identity(&self) -> usize {
        self.identity
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回配置的区域标记。
    #[must_use]
    pub fn locale(&self) -> &str {
        &self.locale
    }
    pub fn get_locale(&self) -> &str { self.locale() }
    /// 返回后端中立输出缓冲。对应 Java Lombok `getOut`。
    pub fn get_out(&self) -> &str { &self.out }
    /// 替换后端中立输出缓冲。对应 Java Lombok `setOut`。
    pub fn set_out(&mut self, value: impl Into<String>) { self.out = value.into(); }

    /// 设置 Java Lombok `setLocale` 对应的区域标记。
    pub fn set_locale(&mut self, locale: impl Into<String>) {
        self.locale = locale.into();
    }

    /// 返回配置的字符集。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn charset(&self) -> &CsvCharset {
        &self.charset
    }
    pub const fn get_charset(&self) -> &CsvCharset { self.charset() }

    /// 设置 Java Lombok `setCharset` 对应的字符集。
    pub fn set_charset(&mut self, charset: CsvCharset) {
        self.charset = charset;
    }

    /// 返回是否写入 BOM。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn with_bom(&self) -> bool {
        self.with_bom
    }
    pub const fn get_with_bom(&self) -> bool { self.with_bom() }

    /// 设置 Java Lombok `setWithBom` 对应的 BOM 开关。
    pub const fn set_with_bom(&mut self, with_bom: bool) {
        self.with_bom = with_bom;
    }

    /// 返回是否启用 1904 日期系统。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn use_1904_windowing(&self) -> bool {
        self.use_1904_windowing
    }
    pub const fn get_use_1904_windowing(&self) -> bool { self.use_1904_windowing() }
    /// Java Lombok 原始字段拼写兼容入口。
    pub const fn get_use1904windowing(&self) -> bool { self.use_1904_windowing() }

    /// 设置 Java Lombok `setUse1904windowing` 对应的日期系统。
    pub const fn set_use_1904_windowing(&mut self, use_1904_windowing: bool) {
        self.use_1904_windowing = use_1904_windowing;
    }
    /// Java Lombok 原始字段拼写兼容入口。
    pub const fn set_use1904windowing(&mut self, value: bool) { self.use_1904_windowing = value; }

    /// 返回是否使用科学计数法。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn use_scientific_format(&self) -> bool {
        self.use_scientific_format
    }
    pub const fn get_use_scientific_format(&self) -> bool { self.use_scientific_format() }

    /// 设置 Java Lombok `setUseScientificFormat` 对应的数字输出策略。
    pub const fn set_use_scientific_format(&mut self, use_scientific_format: bool) {
        self.use_scientific_format = use_scientific_format;
    }

    /// 返回已经创建的唯一工作表。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn sheet(&self) -> Option<&CsvSheet<V>> {
        self.sheet.as_ref()
    }
    pub const fn get_csv_sheet(&self) -> Option<&CsvSheet<V>> { self.sheet() }

    /// 返回唯一工作表的可变引用。
    pub const fn sheet_mut(&mut self) -> Option<&mut CsvSheet<V>> {
        self.sheet.as_mut()
    }

    /// 返回实际存在的逻辑工作表数量。
    #[must_use]
    pub const fn number_of_sheets(&self) -> usize {
        if self.sheet.is_some() { 1 } else { 0 }
    }
    pub const fn get_number_of_sheets(&self) -> usize { self.number_of_sheets() }

    /// 按唯一索引查询工作表，对齐 Java `getSheetAt` 的单 Sheet 约束。
    pub fn sheet_at(&self, index: usize) -> Result<&CsvSheet<V>> {
        if index != 0 {
            return Err(Error::Unsupported(
                "CSV exists only in one sheet".to_owned(),
            ));
        }
        self.sheet
            .as_ref()
            .ok_or_else(|| Error::Csv("CSV sheet has not been created".to_owned()))
    }
    pub fn get_sheet_at(&self, index: usize) -> Result<&CsvSheet<V>> { self.sheet_at(index) }

    /// 按名称查询唯一工作表。
    #[must_use]
    pub fn sheet_by_name(&self, name: &str) -> Option<&CsvSheet<V>> {
        self.sheet
            .as_ref()
            .filter(|sheet| sheet.name() == name)
    }
    pub fn get_sheet(&self, name: &str) -> Option<&CsvSheet<V>> { self.sheet_by_name(name) }

    /// 返回单工作表迭代器，对齐 Java `sheetIterator` / `iterator`。
    pub fn sheets(&self) -> impl Iterator<Item = &CsvSheet<V>> {
        self.sheet.iter()
    }

    /// 返回工作簿局部的数据格式注册表。
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn data_format_mut(&mut self) -> &mut CsvDataFormat {
        &mut self.data_format
    }
    pub const fn get_csv_data_format(&self) -> &CsvDataFormat { self.data_format() }

    /// 返回工作簿局部数据格式注册表。
    #[must_use]
    pub const fn data_format(&self) -> &CsvDataFormat {
        &self.data_format
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建并注册单元格样式。
    ///
    /// # Panics
    ///
    /// 仅当内部 `Vec::push` 未保留刚插入的样式时 panic；正常 Rust 内存模型下不会发生。
    pub fn create_cell_style(&mut self) -> &mut CsvCellStyle {
        let index = i16::try_from(self.cell_styles.len()).unwrap_or(i16::MAX);
        self.cell_styles.push(CsvCellStyle::new(index));
        self.cell_styles
            .last_mut()
            .expect("cell style was just appended")
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按索引返回单元格样式。
    #[must_use]
    pub fn cell_style(&self, index: usize) -> Option<&CsvCellStyle> {
        self.cell_styles.get(index)
    }

    /// 返回 Java `getNumCellStyles` 对应的样式数量。
    #[must_use]
    pub fn number_of_cell_styles(&self) -> usize {
        self.cell_styles.len()
    }
    pub fn get_num_cell_styles(&self) -> usize { self.number_of_cell_styles() }

    /// 返回样式集合，语义对应 Java Lombok `getCsvCellStyleList`。
    #[must_use]
    pub fn cell_styles(&self) -> &[CsvCellStyle] {
        &self.cell_styles
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建并注册唯一的 CSV 工作表。
    ///
    /// # Errors
    ///
    /// 已经存在工作表时返回不支持错误。
    pub fn try_create_sheet(&mut self, sheet_name: &str) -> Result<&mut CsvSheet<V>> {
        if self.sheet.is_some() {
            return Err(Error::Unsupported(
                "CSV repeat sheet creation is not allowed".to_owned(),
            ));
        }
        let mut sheet = CsvSheet::new(sheet_name);
        sheet.set_csv_workbook(Some(self.identity));
        self.sheet = Some(sheet);
        self.sheet
            .as_mut()
            .ok_or_else(|| Error::Csv("CSV sheet assignment produced no sheet".to_owned()))
    }

    /// 创建默认命名的唯一 CSV 工作表。
    pub fn create_sheet(&mut self) -> Result<&mut CsvSheet<V>> {
        self.try_create_sheet("Sheet1")
    }

    /// 创建指定名称的唯一 CSV 工作表。
    pub fn create_sheet_named(&mut self, sheet_name: &str) -> Result<&mut CsvSheet<V>> {
        self.try_create_sheet(sheet_name)
    }

    /// Java `getCellStyleAt`。
    #[must_use]
    pub fn get_cell_style_at(&self, index: usize) -> Option<&CsvCellStyle> {
        self.cell_style(index)
    }
    #[must_use] pub fn get_csv_cell_style_list(&self) -> &[CsvCellStyle] { self.cell_styles() }
    pub fn set_csv_cell_style_list(&mut self, value: Vec<CsvCellStyle>) { self.cell_styles = value; }
    pub fn set_csv_data_format(&mut self, value: CsvDataFormat) { self.data_format = value; }
    pub fn set_csv_sheet(&mut self, mut value: Option<CsvSheet<V>>) {
        if let Some(sheet) = value.as_mut() {
            sheet.set_csv_workbook(Some(self.identity));
        }
        self.sheet = value;
    }

    /// 删除唯一工作表；越界与 Java 一样产生可见错误。
    pub fn remove_sheet_at(&mut self, index: usize) -> Result<()> {
        if index != 0 || self.sheet.is_none() {
            return Err(Error::Unsupported("CSV exists only in one sheet".to_owned()));
        }
        self.sheet = None;
        Ok(())
    }

    pub fn clone_sheet(&mut self, _index: usize) -> Result<&mut CsvSheet<V>> {
        Err(Error::Unsupported("CSV cannot clone sheet".to_owned()))
    }
    pub fn add_picture(&mut self, _data: &[u8], _picture_type: i32) -> Result<usize> {
        Err(Error::Unsupported("CSV cannot add picture".to_owned()))
    }
    pub fn add_ole_package(&mut self, _data: &[u8], _label: &str) -> Result<()> {
        Err(Error::Unsupported("CSV cannot add OLE package".to_owned()))
    }
    pub fn create_name(&mut self, _name: &str) -> Result<()> {
        Err(Error::Unsupported("CSV cannot create workbook names".to_owned()))
    }
    pub fn link_external_workbook(&mut self, _name: &str) -> Result<usize> {
        Err(Error::Unsupported("CSV cannot link external workbook".to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_model::CellValue as ModelCellValue;

    type TestWorkbook = CsvWorkbook<ModelCellValue>;

    fn make_wb() -> TestWorkbook {
        CsvWorkbook::new("", false, false, CsvCharset::utf8(), false)
    }

    #[test]
    fn new_workbook_stores_settings() {
        let wb: TestWorkbook = CsvWorkbook::new("zh-CN", true, false, CsvCharset::utf8(), true);
        assert_eq!(wb.locale(), "zh-CN");
        assert!(wb.use_1904_windowing());
        assert!(!wb.use_scientific_format());
        assert_eq!(wb.charset(), &CsvCharset::utf8());
        assert!(wb.with_bom());
    }

    #[test]
    fn identity_is_unique() {
        let wb1 = make_wb();
        let wb2 = make_wb();
        assert_ne!(wb1.identity(), wb2.identity());
    }

    #[test]
    fn locale_setter_and_getter() {
        let mut wb: TestWorkbook = CsvWorkbook::new("en", false, false, CsvCharset::utf8(), false);
        wb.set_locale("fr-FR");
        assert_eq!(wb.locale(), "fr-FR");
        assert_eq!(wb.get_locale(), "fr-FR");
    }

    #[test]
    fn out_buffer() {
        let mut wb = make_wb();
        assert_eq!(wb.get_out(), "");
        wb.set_out("data");
        assert_eq!(wb.get_out(), "data");
    }

    #[test]
    fn charset_setter() {
        let mut wb = make_wb();
        let gbk = CsvCharset::new("GBK");
        wb.set_charset(gbk.clone());
        assert_eq!(wb.charset(), &gbk);
        assert_eq!(wb.get_charset(), &gbk);
    }

    #[test]
    fn with_bom_setter() {
        let mut wb = make_wb();
        assert!(!wb.with_bom());
        wb.set_with_bom(true);
        assert!(wb.with_bom());
        assert!(wb.get_with_bom());
    }

    #[test]
    fn use_1904_windowing_setter() {
        let mut wb = make_wb();
        wb.set_use_1904_windowing(true);
        assert!(wb.use_1904_windowing());
        assert!(wb.get_use_1904_windowing());
        assert!(wb.get_use1904windowing());
        wb.set_use1904windowing(false);
        assert!(!wb.use_1904_windowing());
    }

    #[test]
    fn use_scientific_format_setter() {
        let mut wb = make_wb();
        wb.set_use_scientific_format(true);
        assert!(wb.use_scientific_format());
        assert!(wb.get_use_scientific_format());
    }

    #[test]
    fn no_sheet_initially() {
        let mut wb = make_wb();
        assert!(wb.sheet().is_none());
        assert!(wb.get_csv_sheet().is_none());
        assert!(wb.sheet_mut().is_none());
        assert_eq!(wb.number_of_sheets(), 0);
        assert_eq!(wb.get_number_of_sheets(), 0);
    }

    #[test]
    fn try_create_sheet_success() {
        let mut wb = make_wb();
        wb.try_create_sheet("Data").unwrap();
        assert!(wb.sheet().is_some());
        assert_eq!(wb.sheet().unwrap().name(), "Data");
        assert_eq!(wb.number_of_sheets(), 1);
    }

    #[test]
    fn try_create_sheet_duplicate_errors() {
        let mut wb = make_wb();
        wb.try_create_sheet("S1").unwrap();
        assert!(wb.try_create_sheet("S2").is_err());
    }

    #[test]
    fn create_sheet_default_name() {
        let mut wb = make_wb();
        wb.create_sheet().unwrap();
        assert_eq!(wb.sheet().unwrap().name(), "Sheet1");
    }

    #[test]
    fn create_sheet_named() {
        let mut wb = make_wb();
        wb.create_sheet_named("Custom").unwrap();
        assert_eq!(wb.sheet().unwrap().name(), "Custom");
    }

    #[test]
    fn sheet_at_index_0_ok() {
        let mut wb = make_wb();
        wb.create_sheet().unwrap();
        assert!(wb.sheet_at(0).is_ok());
        assert!(wb.get_sheet_at(0).is_ok());
    }

    #[test]
    fn sheet_at_nonzero_errors() {
        let mut wb = make_wb();
        wb.create_sheet().unwrap();
        assert!(wb.sheet_at(1).is_err());
    }

    #[test]
    fn sheet_at_no_sheet_errors() {
        let wb = make_wb();
        assert!(wb.sheet_at(0).is_err());
    }

    #[test]
    fn sheet_by_name() {
        let mut wb = make_wb();
        wb.create_sheet_named("Test").unwrap();
        assert!(wb.sheet_by_name("Test").is_some());
        assert!(wb.sheet_by_name("Other").is_none());
        assert!(wb.get_sheet("Test").is_some());
    }

    #[test]
    fn sheets_iterator() {
        let mut wb = make_wb();
        assert_eq!(wb.sheets().count(), 0);
        wb.create_sheet().unwrap();
        assert_eq!(wb.sheets().count(), 1);
    }

    #[test]
    fn remove_sheet_at_success() {
        let mut wb = make_wb();
        wb.create_sheet().unwrap();
        wb.remove_sheet_at(0).unwrap();
        assert!(wb.sheet().is_none());
    }

    #[test]
    fn remove_sheet_at_no_sheet_errors() {
        let mut wb = make_wb();
        assert!(wb.remove_sheet_at(0).is_err());
    }

    #[test]
    fn remove_sheet_at_nonzero_errors() {
        let mut wb = make_wb();
        wb.create_sheet().unwrap();
        assert!(wb.remove_sheet_at(1).is_err());
    }

    #[test]
    fn create_cell_style() {
        let mut wb = make_wb();
        let style = wb.create_cell_style();
        assert_eq!(style.index(), 0);
        assert_eq!(wb.number_of_cell_styles(), 1);
        assert_eq!(wb.get_num_cell_styles(), 1);
    }

    #[test]
    fn cell_style_at_index() {
        let mut wb = make_wb();
        wb.create_cell_style();
        assert!(wb.cell_style(0).is_some());
        assert!(wb.cell_style(99).is_none());
        assert!(wb.get_cell_style_at(0).is_some());
    }

    #[test]
    fn cell_styles_list() {
        let mut wb = make_wb();
        wb.create_cell_style();
        wb.create_cell_style();
        assert_eq!(wb.cell_styles().len(), 2);
        assert_eq!(wb.get_csv_cell_style_list().len(), 2);
    }

    #[test]
    fn data_format() {
        let mut wb = make_wb();
        let _ = wb.data_format();
        let _ = wb.data_format_mut();
        let _ = wb.get_csv_data_format();
    }

    #[test]
    fn set_csv_cell_style_list() {
        let mut wb = make_wb();
        wb.set_csv_cell_style_list(vec![CsvCellStyle::new(0), CsvCellStyle::new(1)]);
        assert_eq!(wb.number_of_cell_styles(), 2);
    }

    #[test]
    fn set_csv_data_format() {
        let mut wb = make_wb();
        let fmt = CsvDataFormat::new();
        wb.set_csv_data_format(fmt);
    }

    #[test]
    fn set_csv_sheet_propagates_workbook_id() {
        let mut wb = make_wb();
        let mut sheet = CsvSheet::new("Test");
        assert!(sheet.get_csv_workbook().is_none());
        wb.set_csv_sheet(Some(sheet));
        assert!(wb.sheet().is_some());
        assert!(wb.sheet().unwrap().get_csv_workbook().is_some());
    }

    #[test]
    fn set_csv_sheet_none() {
        let mut wb = make_wb();
        wb.create_sheet().unwrap();
        wb.set_csv_sheet(None);
        assert!(wb.sheet().is_none());
    }

    // ── 不支持的操作返回错误 ──

    #[test]
    fn clone_sheet_unsupported() {
        let mut wb = make_wb();
        assert!(wb.clone_sheet(0).is_err());
    }

    #[test]
    fn add_picture_unsupported() {
        let mut wb = make_wb();
        assert!(wb.add_picture(&[0], 0).is_err());
    }

    #[test]
    fn add_ole_package_unsupported() {
        let mut wb = make_wb();
        assert!(wb.add_ole_package(&[0], "label").is_err());
    }

    #[test]
    fn create_name_unsupported() {
        let mut wb = make_wb();
        assert!(wb.create_name("name").is_err());
    }

    #[test]
    fn link_external_workbook_unsupported() {
        let mut wb = make_wb();
        assert!(wb.link_external_workbook("name").is_err());
    }
}
