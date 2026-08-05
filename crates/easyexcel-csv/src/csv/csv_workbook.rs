//! CSV 逻辑工作簿模型。

use easyexcel_io::{Error, Result};
use easyexcel_model::CellValue as ModelCellValue;

use super::{CsvCellStyle, CsvCellValue, CsvCharset, CsvDataFormat, CsvSheet};

/// CSV 输出的单工作表逻辑工作簿。
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
    /// 使用全局渲染选项创建工作簿。
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

    /// 返回配置的区域标记。
    #[must_use]
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// 返回配置的字符集。
    #[must_use]
    pub const fn charset(&self) -> &CsvCharset {
        &self.charset
    }

    /// 返回是否写入 BOM。
    #[must_use]
    pub const fn with_bom(&self) -> bool {
        self.with_bom
    }

    /// 返回是否启用 1904 日期系统。
    #[must_use]
    pub const fn use_1904_windowing(&self) -> bool {
        self.use_1904_windowing
    }

    /// 返回是否使用科学计数法。
    #[must_use]
    pub const fn use_scientific_format(&self) -> bool {
        self.use_scientific_format
    }

    /// 返回已经创建的唯一工作表。
    #[must_use]
    pub const fn sheet(&self) -> Option<&CsvSheet<V>> {
        self.sheet.as_ref()
    }

    /// 返回工作簿局部的数据格式注册表。
    pub const fn data_format_mut(&mut self) -> &mut CsvDataFormat {
        &mut self.data_format
    }

    /// 创建并注册单元格样式。
    pub fn create_cell_style(&mut self) -> &mut CsvCellStyle {
        let index = i16::try_from(self.cell_styles.len()).unwrap_or(i16::MAX);
        self.cell_styles.push(CsvCellStyle::new(index));
        self.cell_styles
            .last_mut()
            .expect("cell style was just appended")
    }

    /// 按索引返回单元格样式。
    #[must_use]
    pub fn cell_style(&self, index: usize) -> Option<&CsvCellStyle> {
        self.cell_styles.get(index)
    }

    /// 创建并注册唯一的 CSV 工作表。
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
        Ok(self.sheet.as_mut().expect("sheet was just assigned"))
    }
}
