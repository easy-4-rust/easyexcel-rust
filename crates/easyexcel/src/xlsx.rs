//! OOXML XLSX 基础工作簿引擎门面。

pub use easyexcel_xlsx::{
    AnchorCoordinate, ImageAnchorSpec, LegacyTemplateSheet, MAX_XLSX_COLUMN_NUMBER,
    MAX_XLSX_ROW_NUMBER, OoxmlPackage, OoxmlTemplatePackage, OoxmlZipEntry, ReadSeek,
    ReadWriteSeek, ResolvedImageAnchor, RichTextSegment, TemplateCollectionFill, TemplateFillData,
    TemplateFillDirection, TemplateSheetSelector, XlsxCellEvent, XlsxCellEventReader,
    XlsxCellValue, XlsxDisplayOptions, XlsxEventMetadata, XlsxExtra, XlsxExtraKind, XlsxInput,
    XlsxNumberFormat, XlsxPackageReader, XlsxSource, append_rows_to_sheet, append_rows_to_xml,
    decode_ooxml_escape, decrypt_file, dimension_last_row, encrypt_package_to,
    excel_input_suffix, has_template, is_compound_document, is_encrypted_ooxml,
    load_legacy_template_sheets, load_template_bytes, local_tag_name, looks_like_zip,
    materialize_excel_input, normalize_workbook_target, parse_a1_cell_range,
    parse_a1_cell_reference, parse_attribute_pairs, parse_xlsx_index, parse_xlsx_row_number, read,
    read_path, read_path_with_password, read_with_password, render_typed_cell,
    replace_collection_fills_in_sheet, replace_scalar_cells, replace_scalar_cells_in_sheet,
    replace_scalar_cells_in_xml, resolve_image_anchor, resolve_sheet_target,
    seed_legacy_template_workbook, segment_utf16_text, stream, validate_xlsx_template_source,
    workbook_sheets, worksheet_path, write, write_path, xml_elements,
};

pub use easyexcel_xlsx::xlsx::{
    generation, ooxml_constants, package, template_fill, template_styles, template_xml,
};
