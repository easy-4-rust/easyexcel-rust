//! OOXML XLSX 工作簿读取、写入、流式读取和 `RoundTrip` 支持。

pub mod xlsx;

pub use xlsx::{
    OoxmlPackage, OoxmlZipEntry, ReadWriteSeek, TemplateCollectionFill, TemplateFillData,
    TemplateFillDirection, TemplateSheetSelector, append_rows_to_sheet, append_rows_to_xml,
    encrypt_package_to, has_template, load_template_bytes, looks_like_zip,
    normalize_workbook_target, read, read_path, read_path_with_password, read_with_password,
    render_typed_cell, replace_collection_fills_in_sheet, replace_scalar_cells,
    replace_scalar_cells_in_sheet, replace_scalar_cells_in_xml, resolve_sheet_target, stream,
    validate_xlsx_template_source, workbook_sheets, worksheet_path, write, write_path, xml_elements,
};
