//! EasyExcel 写入语义到 `easyexcel-xls` BIFF8 引擎的适配层。
//!
//! OLE/CFB、BIFF 记录、公式、加密、工作簿与模板算法全部位于
//! `easyexcel-xls`；本模块只转换 EasyExcel 的 `CellValue` 与样式 metadata。

mod style;
mod template;

pub(crate) use easyexcel_xls::biff8::{
    Biff8Book, Biff8Cell, Biff8Color, Biff8Comment, Biff8FillPattern, Biff8HyperlinkKind,
    Biff8Merge, Biff8Sheet, Biff8StyleRequest, Biff8StyleTable,
    GeneratedBiff8CellValue, date_to_excel_serial, date_to_excel_serial_with_windowing,
    datetime_to_excel_serial, datetime_to_excel_serial_with_windowing, looks_like_xls,
};
#[allow(unused_imports)]
pub(crate) use style::{
    apply_excel_cell_style, apply_excel_font_style, apply_write_font, writer_horizontal_alignment,
    writer_vertical_alignment,
};
pub(crate) use template::Biff8TemplatePackage;
