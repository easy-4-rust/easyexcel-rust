const FACADE_MANIFEST: &str = "crates/easyexcel/Cargo.toml";
const FACADE_LIB: &str = "crates/easyexcel/src/lib.rs";
const CACHE_ENGINE_MANIFEST: &str = "crates/easyexcel-cache/Cargo.toml";
const CACHE_ENGINE: &str = "crates/easyexcel-cache/src/cache/shared_string_cache.rs";
const CACHE_POLICY_ENGINE: &str = "crates/easyexcel-cache/src/cache/shared_string_cache_policy.rs";
const FACADE_CACHE_MOD: &str = "crates/easyexcel/src/cache/mod.rs";
const REMOVED_JAVA_CACHE_ADAPTER: &str = concat!("crates/easyexcel/src/cache/eh", "cache.rs");
const FILE_CACHE_ADAPTER: &str = "crates/easyexcel/src/cache/file_cache.rs";
const MOKA_ADAPTER: &str = "crates/easyexcel/src/cache/moka_cache.rs";
const OUTPUT_STREAM_COMPAT: &str = "crates/easyexcel/src/write/excel_output_stream.rs";
const IO_ROW_RANGE_ENGINE: &str = "crates/easyexcel-io/src/io/row_range.rs";
const IO_SHEET_SELECTION_ENGINE: &str = "crates/easyexcel-io/src/io/sheet_selection.rs";
const IO_FORMAT_ENGINE: &str = "crates/easyexcel-io/src/io/format.rs";
const IO_GZIP_CELL_ENGINE: &str = "crates/easyexcel-io/src/io/gzip_cell_record.rs";
const MODEL_STORED_ROW_ENGINE: &str = "crates/easyexcel-model/src/model/stored_row.rs";
const CSV_ENCODING_ADAPTER: &str = "crates/easyexcel/src/write/csv_encoding_writer.rs";
const EXCEL_TYPE_ADAPTER: &str = "crates/easyexcel/src/support/excel_type_enum.rs";
const XLSX_FACADE: &str = "crates/easyexcel/src/xlsx.rs";
const XLS_RECORD_DISPATCHER: &str = "crates/easyexcel/src/analysis/v03/xls_record_dispatcher.rs";
const XLS_SAX_ADAPTER: &str = "crates/easyexcel/src/analysis/v03/xls_sax_analyser.rs";
const XLSX_SAX_ADAPTER: &str = "crates/easyexcel/src/analysis/v07/xlsx_sax_analyser.rs";
const XLS_OBJ_HANDLER: &str = "crates/easyexcel/src/analysis/v03/handlers/obj_record_handler.rs";
const STYLE_UTIL_ADAPTER: &str = "crates/easyexcel/src/util/style_util.rs";
const FACADE_ERROR: &str = "crates/easyexcel/src/support/excel_error.rs";
const XLS_TEMPLATE_ADAPTER: &str = "crates/easyexcel/src/write/xls_adapter/template.rs";
const XLSX_TEMPLATE_ADAPTER: &str = "crates/easyexcel/src/template/template_writer.rs";
const XLSX_TEMPLATE_SELECTION_ENGINE: &str = "crates/easyexcel-xlsx/src/xlsx/template_source.rs";
const XLSX_EVENT_READER_ENGINE: &str = "crates/easyexcel-xlsx/src/xlsx/event_reader.rs";
const ROW_PROCESSING_ADAPTER: &str = "crates/easyexcel/src/read/row_processing.rs";
const TEMPLATE_WRITE_ADAPTER: &str = "crates/easyexcel/src/write/template_write.rs";
const READ_HELPERS_ADAPTER: &str = "crates/easyexcel/src/read/read_helpers.rs";
const EXCEL_WRITER_CORE: &str = "crates/easyexcel/src/write/excel_writer_core.rs";
const STRING_UTILS_ENGINE: &str = "crates/easyexcel-utils/src/utils/string_utils.rs";
const CLASS_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/class_utils.rs";
const FIELD_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/field_utils.rs";
const EXCEL_ANALYSER_ADAPTER: &str = "crates/easyexcel/src/analysis/excel_analyser_impl.rs";
const GZIP_SPILL_ADAPTER: &str = "crates/easyexcel/src/write/gzip_spill.rs";

const REQUIRED_ENGINE_DEPENDENCIES: &[&str] = &[
    "easyexcel-cache",
    "easyexcel-csv",
    "easyexcel-format",
    "easyexcel-formula",
    "easyexcel-io",
    "easyexcel-model",
    "easyexcel-tabular",
    "easyexcel-utils",
    "easyexcel-xls",
    "easyexcel-xlsx",
];

const FORBIDDEN_FACADE_DEPENDENCIES: &[&str] = &[
    "aes",
    "calamine",
    "cfb",
    "csv",
    "encoding_rs",
    "encoding_rs_io",
    "flate2",
    "md-5",
    "moka",
    "ms-offcrypto-writer",
    "office-crypto",
    "quick-xml",
    "rand",
    "rust_xlsxwriter",
    "sha1",
    "sha2",
    "ssfmt",
    "tempfile",
    "zip",
];

const FOUNDATION_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/constant/excel_xml_constants.rs",
    "crates/easyexcel/src/constant/mod.rs",
    "crates/easyexcel/src/metadata/format/mod.rs",
    "crates/easyexcel/src/util/map_utils.rs",
    "crates/easyexcel/src/util/string_utils.rs",
    "crates/easyexcel/src/util/boolean_utils.rs",
    "crates/easyexcel/src/util/int_utils.rs",
    "crates/easyexcel/src/util/list_utils.rs",
    "crates/easyexcel/src/util/sheet_utils.rs",
    "crates/easyexcel/src/util/mod.rs",
];

const JAVA_TRIM_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/write/excel_builder_impl.rs",
    "crates/easyexcel/src/analysis/v03/handlers/label_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/label_sst_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/string_record_handler.rs",
    "crates/easyexcel/src/util/cell_editor.rs",
];

const XLS_RECORD_DECODER_ADAPTERS: &[(&str, &str)] = &[
    (
        "crates/easyexcel/src/analysis/v03/handlers/blank_record_handler.rs",
        "event_record::decode_cell_header(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/bof_record_handler.rs",
        "event_record::decode_bof_type(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/bool_err_record_handler.rs",
        "event_record::decode_bool_err_record(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/bound_sheet_record_handler.rs",
        "event_record::decode_bound_sheet_record(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/formula_record_handler.rs",
        "event_record::decode_formula_record(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/hyperlink_record_handler.rs",
        "event_record::decode_cell_range(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/index_record_handler.rs",
        "event_record::decode_index_last_row(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/label_record_handler.rs",
        "event_record::decode_label_record_position(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/label_sst_record_handler.rs",
        "event_record::decode_label_sst_record(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/merge_cells_record_handler.rs",
        "event_record::decode_merge_ranges(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/note_record_handler.rs",
        "event_record::decode_note_record_position(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/number_record_handler.rs",
        "event_record::decode_number_record(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/obj_record_handler.rs",
        "event_record::decode_obj_common_data(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/rk_record_handler.rs",
        "event_record::decode_cell_position(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/sst_record_handler.rs",
        "event_record::decode_sst_unique_count(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/string_record_handler.rs",
        "biff8::string::decode_unicode_string_record(data)",
    ),
    (
        "crates/easyexcel/src/analysis/v03/handlers/text_object_record_handler.rs",
        "event_record::decode_text_object_fragment(",
    ),
];

const XLSX_HANDLER_ADAPTERS: &[(&str, &str)] = &[
    (
        "crates/easyexcel/src/analysis/v07/handlers/cell_tag_handler.rs",
        "easyexcel_xlsx::parse_a1_cell_reference(reference)",
    ),
    (
        "crates/easyexcel/src/analysis/v07/handlers/count_tag_handler.rs",
        "easyexcel_xlsx::dimension_last_row(ref_attr)",
    ),
    (
        "crates/easyexcel/src/analysis/v07/handlers/merge_cell_tag_handler.rs",
        "easyexcel_xlsx::parse_a1_cell_range(reference)",
    ),
    (
        "crates/easyexcel/src/analysis/v07/handlers/row_tag_handler.rs",
        "easyexcel_xlsx::parse_xlsx_row_number(value)",
    ),
    (
        "crates/easyexcel/src/analysis/v07/handlers/sax/shared_strings_table_handler.rs",
        "easyexcel_xlsx::decode_ooxml_escape(value)",
    ),
];

