//! CSV 逻辑工作簿模型。

use easyexcel_io::{Error, Result};
use easyexcel_model::CellValue as ModelCellValue;

use super::{CsvCellStyle, CsvCellValue, CsvCharset, CsvDataFormat, CsvSheet};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 CSV 输出的单工作表逻辑工作簿。
#[derive(Debug, Clone, PartialEq)]
pub struct CsvWorkbook<V: CsvCellValue = ModelCellValue> {
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回配置的区域标记。
    #[must_use]
    pub fn locale(&self) -> &str {
        &self.locale
    }
    pub fn get_locale(&self) -> &str { self.locale() }

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

    /// 设置 Java Lombok `setUse1904windowing` 对应的日期系统。
    pub const fn set_use_1904_windowing(&mut self, use_1904_windowing: bool) {
        self.use_1904_windowing = use_1904_windowing;
    }

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
        self.sheet = Some(CsvSheet::new(sheet_name));
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

    /// CSV 没有字体表，对齐 Java 的固定返回值。
    #[must_use]
    pub const fn number_of_fonts(&self) -> usize {
        0
    }
    pub const fn get_number_of_fonts(&self) -> usize { self.number_of_fonts() }
    pub const fn get_number_of_fonts_as_int(&self) -> usize { self.number_of_fonts() }

    /// CSV 没有名称表，对齐 Java 的固定返回值。
    #[must_use]
    pub const fn number_of_names(&self) -> usize {
        0
    }
    pub const fn get_number_of_names(&self) -> usize { self.number_of_names() }

    /// CSV 工作簿本身不持久化隐藏状态。
    #[must_use]
    pub const fn is_hidden(&self) -> bool {
        false
    }

    /// CSV 工作簿本身不执行公式重算。
    #[must_use]
    pub const fn force_formula_recalculation(&self) -> bool {
        false
    }
    pub const fn get_force_formula_recalculation(&self) -> bool {
        self.force_formula_recalculation()
    }

    /// Java CSV Workbook 对 POI 非 CSV 能力的确定性默认语义。
    pub const fn get_active_sheet_index(&self) -> usize { 0 }
    pub const fn get_first_visible_tab(&self) -> usize { 0 }
    pub const fn get_sheet_index(&self, _name: &str) -> usize { 0 }
    pub const fn get_missing_cell_policy(&self) -> u8 { 0 }
    pub const fn is_sheet_hidden(&self, _index: usize) -> bool { false }
    pub const fn is_sheet_very_hidden(&self, _index: usize) -> bool { false }
    pub const fn set_active_sheet(&mut self, _index: usize) {}
    pub const fn set_first_visible_tab(&mut self, _index: usize) {}
    pub const fn set_selected_tab(&mut self, _index: usize) {}
    pub const fn set_sheet_order(&mut self, _name: &str, _index: usize) {}
    pub const fn set_sheet_name(&mut self, _index: usize, _name: &str) {}
    pub const fn set_hidden(&mut self, _hidden: bool) {}
    pub const fn set_sheet_hidden(&mut self, _index: usize, _hidden: bool) {}
    pub const fn set_force_formula_recalculation(&mut self, _value: bool) {}
    pub fn iterator(&self) -> impl Iterator<Item = &CsvSheet<V>> { self.sheets() }
}
