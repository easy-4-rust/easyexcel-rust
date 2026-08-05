//! EasyExcel 写入语义到 `easyexcel-xls` BIFF8 引擎的适配层。
//!
//! OLE/CFB、BIFF 记录、公式、加密、工作簿与模板算法全部位于
//! `easyexcel-xls`；本模块只转换 EasyExcel 的 `CellValue` 与样式 metadata。

mod style;
mod template;

pub(crate) use easyexcel_xls::biff8::{
    Biff8Book, Biff8Cell, Biff8Merge, Biff8Sheet, Biff8StyleRequest, Biff8StyleTable, Biff8Value,
    date_to_excel_serial_with_windowing, datetime_to_excel_serial_with_windowing, looks_like_xls,
};
pub(crate) use style::{apply_excel_cell_style, apply_excel_font_style};
pub(crate) use template::Biff8TemplatePackage;
