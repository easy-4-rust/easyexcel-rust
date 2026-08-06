//! `easyexcel` 门面与基础引擎 crate 的依赖边界审计。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::TaskResult;

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

/// 校验门面只依赖基础引擎，不直接依赖格式、压缩、加密或缓存实现库。
pub(crate) fn audit() -> TaskResult {
    let manifest = read(FACADE_MANIFEST)?;
    let dependencies = dependency_names(&manifest);

    let missing = REQUIRED_ENGINE_DEPENDENCIES
        .iter()
        .copied()
        .filter(|name| !dependencies.contains(*name))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "facade is missing required engine dependencies: {}",
            missing.join(", ")
        )
        .into());
    }

    let forbidden = FORBIDDEN_FACADE_DEPENDENCIES
        .iter()
        .copied()
        .filter(|name| dependencies.contains(*name))
        .collect::<Vec<_>>();
    if !forbidden.is_empty() {
        return Err(format!(
            "facade directly depends on low-level implementation crates: {}",
            forbidden.join(", ")
        )
        .into());
    }

    let cache_manifest = read(CACHE_ENGINE_MANIFEST)?;
    let cache_dependencies = dependency_names(&cache_manifest);
    for dependency in ["moka", "tempfile"] {
        if !cache_dependencies.contains(dependency) {
            return Err(format!(
                "cache engine is missing required implementation dependency: {dependency}"
            )
            .into());
        }
    }

    let cache_engine = read(CACHE_ENGINE)?;
    for (needle, purpose) in [
        ("use moka::sync::Cache;", "Moka cache implementation"),
        ("struct MokaSharedStringCache", "Moka write-phase cache"),
        ("objects: Cache<usize, Arc<str>>", "Moka object store"),
        (
            "objects: Cache::builder().build()",
            "unbounded Moka construction",
        ),
        ("self.objects.insert(index", "Moka object insertion"),
        ("struct MokaSharedStringReader", "Moka read-phase cache"),
        ("struct FileSharedStringCache", "file-cache write phase"),
        ("struct FileSharedStringReader", "file-cache read phase"),
        ("temporary_file: NamedTempFile", "file-cache lifetime guard"),
    ] {
        require_contains(CACHE_ENGINE, &cache_engine, needle, purpose)?;
    }
    for forbidden in ["max_capacity(", "time_to_live(", "time_to_idle("] {
        require_absent(
            CACHE_ENGINE,
            &cache_engine,
            forbidden,
            "Moka entry eviction",
        )?;
    }

    let cache_policy_engine = read(CACHE_POLICY_ENGINE)?;
    require_contains(
        CACHE_POLICY_ENGINE,
        &cache_policy_engine,
        "create_cache(ReadCacheMode::File, shared_strings_xml_size)",
        "bounded-memory file-cache policy",
    )?;
    for forbidden in ["max_active_", "weighted", "batches"] {
        require_absent(
            CACHE_POLICY_ENGINE,
            &cache_policy_engine,
            forbidden,
            "Moka eviction policy",
        )?;
    }

    let xlsx_event_reader_engine = read(XLSX_EVENT_READER_ENGINE)?;
    for (needle, purpose) in [
        (
            "create_cache(cache_mode, xml_size)",
            "cache selection in XLSX SAX metadata",
        ),
        ("BufReader::new(file)", "buffered XLSX part streaming"),
        ("read_event_into", "incremental XML event reading"),
        ("parse_shared_strings", "streamed shared-string decoding"),
    ] {
        require_contains(
            XLSX_EVENT_READER_ENGINE,
            &xlsx_event_reader_engine,
            needle,
            purpose,
        )?;
    }

    let facade = read(FACADE_LIB)?;
    for module in [
        "csv", "formula", "format", "io", "model", "tabular", "xls", "xlsx",
    ] {
        require_contains(
            FACADE_LIB,
            &facade,
            &format!("pub mod {module};"),
            "foundation API facade module",
        )?;

        let module_path = format!("crates/easyexcel/src/{module}.rs");
        let module_source = read(&module_path)?;
        require_no_wildcard_imports(&module_path, &module_source)?;
    }
    for path in FOUNDATION_ADAPTERS {
        let source = read(path)?;
        require_no_wildcard_imports(path, &source)?;
    }
    for path in JAVA_TRIM_ADAPTERS {
        let source = read(path)?;
        require_contains(
            path,
            &source,
            "easyexcel_utils::string_utils::java_trim",
            "shared Java-compatible trimming",
        )?;
        require_absent(
            path,
            &source,
            ".trim()",
            "Rust Unicode trim in Java adapter",
        )?;
    }

    require_path_absent(REMOVED_JAVA_CACHE_ADAPTER, "removed Java cache adapter")?;
    require_tree_absent_case_insensitive(
        "crates/easyexcel/src",
        concat!("eh", "cache"),
        "removed Java cache vocabulary",
    )?;
    let facade_cache_mod = read(FACADE_CACHE_MOD)?;
    require_absent(
        FACADE_CACHE_MOD,
        &facade_cache_mod,
        concat!("eh", "cache"),
        "removed Java cache vocabulary",
    )?;

    let moka_adapter = read(MOKA_ADAPTER)?;
    require_contains(
        MOKA_ADAPTER,
        &moka_adapter,
        "easyexcel_cache::create_moka_cache()",
        "engine-owned Moka object cache",
    )?;
    require_absent(
        MOKA_ADAPTER,
        &moka_adapter,
        "moka::",
        "direct Moka implementation",
    )?;
    for forbidden in ["max_capacity", "max_active", "megabytes", "batches"] {
        require_absent(
            MOKA_ADAPTER,
            &moka_adapter,
            forbidden,
            "Moka eviction configuration",
        )?;
    }

    let file_cache_adapter = read(FILE_CACHE_ADAPTER)?;
    require_contains(
        FILE_CACHE_ADAPTER,
        &file_cache_adapter,
        "easyexcel_cache::create_file_cache()?",
        "engine-owned file cache",
    )?;
    require_absent(
        FILE_CACHE_ADAPTER,
        &file_cache_adapter,
        "tempfile::",
        "facade-owned temporary-file implementation",
    )?;

    let output_stream = read(OUTPUT_STREAM_COMPAT)?;
    require_contains(
        OUTPUT_STREAM_COMPAT,
        &output_stream,
        "easyexcel_io::CloseableOutputStream<W>",
        "engine-owned output stream",
    )?;
    require_absent(
        OUTPUT_STREAM_COMPAT,
        &output_stream,
        "Arc<Mutex",
        "shared output implementation",
    )?;

    let csv_encoding_adapter = read(CSV_ENCODING_ADAPTER)?;
    for (needle, purpose) in [
        (
            "inner: easyexcel_csv::CsvEncodingWriter",
            "CSV-engine-owned encoder",
        ),
        (
            "easyexcel_csv::CsvEncodingWriter::encode_utf16",
            "CSV-engine-owned UTF-16 encoding",
        ),
        (
            "easyexcel_csv::csv_encoding",
            "CSV-engine-owned charset lookup",
        ),
        ("easyexcel_csv::csv_bom", "CSV-engine-owned BOM selection"),
    ] {
        require_contains(CSV_ENCODING_ADAPTER, &csv_encoding_adapter, needle, purpose)?;
    }
    for forbidden in ["encoding_rs::", "encode_utf16().flat_map", "to_le_bytes()"] {
        require_absent(
            CSV_ENCODING_ADAPTER,
            &csv_encoding_adapter,
            forbidden,
            "facade-owned CSV encoding",
        )?;
    }

    let excel_type_adapter = read(EXCEL_TYPE_ADAPTER)?;
    for needle in [
        "easyexcel_io::Format::from_magic(bytes)",
        "easyexcel_io::Format::from_extension(extension)",
    ] {
        require_contains(
            EXCEL_TYPE_ADAPTER,
            &excel_type_adapter,
            needle,
            "I/O-owned format recognition",
        )?;
    }
    for forbidden in ["bytes.starts_with", "path.extension()"] {
        require_absent(
            EXCEL_TYPE_ADAPTER,
            &excel_type_adapter,
            forbidden,
            "facade-owned format recognition",
        )?;
    }

    let model_stored_row_engine = read(MODEL_STORED_ROW_ENGINE)?;
    require_contains(
        MODEL_STORED_ROW_ENGINE,
        &model_stored_row_engine,
        "self.stored_range()",
        "engine-owned sparse model bounds",
    )?;

    let xlsx_facade = read(XLSX_FACADE)?;
    for symbol in [
        "XlsxSource",
        "XlsxCellEventReader",
        "OoxmlTemplatePackage",
        "materialize_excel_input",
        "template_xml",
    ] {
        require_contains(
            XLSX_FACADE,
            &xlsx_facade,
            symbol,
            "XLSX engine API facade export",
        )?;
    }

    let xls_record_dispatcher = read(XLS_RECORD_DISPATCHER)?;
    require_contains(
        XLS_RECORD_DISPATCHER,
        &xls_record_dispatcher,
        "record_sid::is_skippable_event_record(record_sid)",
        "engine-owned BIFF event-record classification",
    )?;
    require_absent(
        XLS_RECORD_DISPATCHER,
        &xls_record_dispatcher,
        "fn is_ignorable_sid",
        "facade-owned BIFF SID classification",
    )?;
    require_contains(
        XLS_RECORD_DISPATCHER,
        &xls_record_dispatcher,
        ".as_engine_selection().matches(",
        "I/O-owned event sheet selection",
    )?;
    require_absent(
        XLS_RECORD_DISPATCHER,
        &xls_record_dispatcher,
        "SheetSelector::First => index == 0",
        "facade-owned event sheet selection",
    )?;

    let xls_obj_handler = read(XLS_OBJ_HANDLER)?;
    require_contains(
        XLS_OBJ_HANDLER,
        &xls_obj_handler,
        "easyexcel_xls::biff8::event_record::decode_obj_common_data(data)",
        "engine-owned BIFF OBJ decoding",
    )?;
    require_absent(
        XLS_OBJ_HANDLER,
        &xls_obj_handler,
        "parse is deferred",
        "deferred BIFF OBJ parsing stub",
    )?;

    for (path, decoder) in XLS_RECORD_DECODER_ADAPTERS {
        let source = read(path)?;
        let production = production_prefix(&source);
        require_contains(path, production, decoder, "XLS-engine-owned BIFF decoding")?;
        for forbidden in [
            "from_le_bytes",
            "from_be_bytes",
            "std::str::from_utf8",
            "cfb::",
            "data[",
        ] {
            require_absent(path, production, forbidden, "facade-owned BIFF decoding")?;
        }
    }

    let xls_sax_adapter = read(XLS_SAX_ADAPTER)?;
    let xls_sax_production = production_prefix(&xls_sax_adapter);
    for (needle, purpose) in [
        (
            "record_stream::read_workbook_stream(&self.path)",
            "XLS-engine-owned OLE workbook extraction",
        ),
        (
            "record_stream::walk_biff_records(&workbook",
            "XLS-engine-owned BIFF record traversal",
        ),
    ] {
        require_contains(XLS_SAX_ADAPTER, xls_sax_production, needle, purpose)?;
    }
    for forbidden in ["cfb::", "from_le_bytes", "File::open", "read_to_end"] {
        require_absent(
            XLS_SAX_ADAPTER,
            xls_sax_production,
            forbidden,
            "facade-owned XLS binary/OLE processing",
        )?;
    }

    let xlsx_sax_adapter = read(XLSX_SAX_ADAPTER)?;
    let xlsx_sax_production = production_prefix(&xlsx_sax_adapter);
    for needle in ["list_xlsx_sheets(&path, &options)", "read_xlsx::<T, L>("] {
        require_contains(
            XLSX_SAX_ADAPTER,
            xlsx_sax_production,
            needle,
            "XLSX-engine-backed facade analysis",
        )?;
    }
    for forbidden in ["quick_xml::", "zip::", "ZipArchive", "from_utf8"] {
        require_absent(
            XLSX_SAX_ADAPTER,
            xlsx_sax_production,
            forbidden,
            "facade-owned XML/ZIP processing",
        )?;
    }

    for (path, engine_call) in XLSX_HANDLER_ADAPTERS {
        let source = read(path)?;
        let production = production_prefix(&source);
        require_contains(
            path,
            production,
            engine_call,
            "XLSX-engine-owned reusable parsing",
        )?;
        for forbidden in ["quick_xml::", "from_utf8", "split_once(':')"] {
            require_absent(path, production, forbidden, "facade-owned OOXML parsing")?;
        }
    }

    let style_util_adapter = read(STYLE_UTIL_ADAPTER)?;
    require_contains(
        STYLE_UTIL_ADAPTER,
        &style_util_adapter,
        "DataFormatData::resolve(data_format_data)",
        "model-owned data-format normalization",
    )?;
    require_absent(
        STYLE_UTIL_ADAPTER,
        &style_util_adapter,
        "fn general_data_format",
        "facade-owned General data-format construction",
    )?;

    let facade_error = read(FACADE_ERROR)?;
    require_contains(
        FACADE_ERROR,
        &facade_error,
        "easyexcel_io::Error::SheetNotFound(sheet) => Self::SheetNotFound(sheet)",
        "typed engine sheet-not-found mapping",
    )?;
    for path in [XLS_TEMPLATE_ADAPTER, XLSX_TEMPLATE_ADAPTER] {
        let source = read(path)?;
        require_absent(
            path,
            &source,
            "strip_prefix(\"worksheet not found: \")",
            "string-parsed sheet-not-found error",
        )?;
    }
    let xlsx_template_adapter = read(XLSX_TEMPLATE_ADAPTER)?;
    require_contains(
        XLSX_TEMPLATE_ADAPTER,
        &xlsx_template_adapter,
        ".equivalent(right.as_engine_selector())",
        "XLSX-engine-owned template sheet equivalence",
    )?;
    require_absent(
        XLSX_TEMPLATE_ADAPTER,
        &xlsx_template_adapter,
        "TemplateSheet::First | TemplateSheet::Index(0)",
        "facade-owned template sheet equivalence",
    )?;
    let xlsx_template_selection_engine = read(XLSX_TEMPLATE_SELECTION_ENGINE)?;
    require_contains(
        XLSX_TEMPLATE_SELECTION_ENGINE,
        &xlsx_template_selection_engine,
        "pub fn equivalent<'b>(self, other: TemplateSheetSelector<'b>) -> bool",
        "template sheet equivalence algorithm",
    )?;

    let row_processing_adapter = read(ROW_PROCESSING_ADAPTER)?;
    require_contains(
        ROW_PROCESSING_ADAPTER,
        &row_processing_adapter,
        "easyexcel_io::select_sheet_names(names, selector.as_engine_selection(), auto_trim)",
        "I/O-owned sheet selection",
    )?;
    require_absent(
        ROW_PROCESSING_ADAPTER,
        &row_processing_adapter,
        "names.first()",
        "facade-owned first-sheet selection",
    )?;
    require_contains(
        ROW_PROCESSING_ADAPTER,
        &row_processing_adapter,
        "sheet.stored_rows()",
        "model-owned stored-row traversal",
    )?;
    require_absent(
        ROW_PROCESSING_ADAPTER,
        &row_processing_adapter,
        ".cells\n            .range",
        "facade-owned sparse model traversal",
    )?;
    require_contains(
        ROW_PROCESSING_ADAPTER,
        &row_processing_adapter,
        "easyexcel_io::row_is_selected(",
        "I/O-owned row filtering",
    )?;
    require_absent(
        ROW_PROCESSING_ADAPTER,
        &row_processing_adapter,
        "row_index >= options.head_row_number",
        "facade-owned row filtering",
    )?;

    let io_sheet_selection_engine = read(IO_SHEET_SELECTION_ENGINE)?;
    require_contains(
        IO_SHEET_SELECTION_ENGINE,
        &io_sheet_selection_engine,
        "pub fn matches(self, index: usize, name: Option<&str>, auto_trim: bool) -> bool",
        "streaming sheet-selection predicate",
    )?;

    let io_row_range_engine = read(IO_ROW_RANGE_ENGINE)?;
    require_contains(
        IO_ROW_RANGE_ENGINE,
        &io_row_range_engine,
        "pub fn row_is_selected(",
        "shared row-selection predicate",
    )?;

    let io_format_engine = read(IO_FORMAT_ENGINE)?;
    require_contains(
        IO_FORMAT_ENGINE,
        &io_format_engine,
        "pub fn detect_path(path: &Path) -> Result<Self>",
        "path extension and magic format detection",
    )?;
    let excel_analyser_adapter = read(EXCEL_ANALYSER_ADAPTER)?;
    require_contains(
        EXCEL_ANALYSER_ADAPTER,
        &excel_analyser_adapter,
        "easyexcel_io::Format::detect_path(path)",
        "I/O-owned workbook format detection",
    )?;
    require_absent(
        EXCEL_ANALYSER_ADAPTER,
        &excel_analyser_adapter,
        "path.extension()",
        "facade-owned extension fallback",
    )?;

    let io_gzip_cell_engine = read(IO_GZIP_CELL_ENGINE)?;
    for symbol in [
        "pub struct GzipCellSpillSnapshot",
        "pub struct GzipCellSpillWriter",
        "pub struct GzipCellSpillReader",
    ] {
        require_contains(
            IO_GZIP_CELL_ENGINE,
            &io_gzip_cell_engine,
            symbol,
            "engine-owned gzip sheet spill lifecycle",
        )?;
    }
    let gzip_spill_adapter = read(GZIP_SPILL_ADAPTER)?;
    require_contains(
        GZIP_SPILL_ADAPTER,
        &gzip_spill_adapter,
        "GzipCellSpillWriter as EngineSpillWriter",
        "I/O-owned gzip spill writer",
    )?;
    require_contains(
        GZIP_SPILL_ADAPTER,
        &gzip_spill_adapter,
        "pub type GzipSpillSnapshot = easyexcel_io::GzipCellSpillSnapshot",
        "I/O-owned gzip spill snapshot",
    )?;
    for forbidden in [
        "pub struct GzipSpillSnapshot",
        "GzipCellRecordWriter",
        "GzipCellRecordReader",
    ] {
        require_absent(
            GZIP_SPILL_ADAPTER,
            &gzip_spill_adapter,
            forbidden,
            "facade-owned gzip spill lifecycle",
        )?;
    }

    let template_write_adapter = read(TEMPLATE_WRITE_ADAPTER)?;
    require_absent(
        TEMPLATE_WRITE_ADAPTER,
        &template_write_adapter,
        "if !self.sheet_names()?.iter().any",
        "duplicated facade sheet-existence scan",
    )?;
    require_absent(
        TEMPLATE_WRITE_ADAPTER,
        &template_write_adapter,
        "if index >= self.sheet_names()?.len()",
        "duplicated facade sheet-index bounds check",
    )?;

    let xlsx_template_adapter = read(XLSX_TEMPLATE_ADAPTER)?;
    require_contains(
        XLSX_TEMPLATE_ADAPTER,
        &xlsx_template_adapter,
        "easyexcel_xlsx::OoxmlPackage::from_entries(entries.to_vec()).to_bytes()",
        "XLSX-engine-owned OOXML ZIP encoding",
    )?;

    let read_helpers_adapter = read(READ_HELPERS_ADAPTER)?;
    require_contains(
        READ_HELPERS_ADAPTER,
        &read_helpers_adapter,
        "easyexcel_io::validate_row_range(options.start_row, options.end_row)",
        "I/O-owned row-range validation",
    )?;
    require_absent(
        READ_HELPERS_ADAPTER,
        &read_helpers_adapter,
        "start > end",
        "facade-owned row-range validation",
    )?;

    let excel_writer_core = read(EXCEL_WRITER_CORE)?;
    require_absent(
        EXCEL_WRITER_CORE,
        &excel_writer_core,
        "fn validate_xls_options",
        "no-op XLS validation stub",
    )?;
    require_contains(
        EXCEL_WRITER_CORE,
        &excel_writer_core,
        "easyexcel_utils::string_utils::java_trim(&options.sheet_name)",
        "shared Java-compatible sheet-name trimming",
    )?;

    let string_utils_engine = read(STRING_UTILS_ENGINE)?;
    require_contains(
        STRING_UTILS_ENGINE,
        &string_utils_engine,
        "Cow::Borrowed(java_trim(value))",
        "allocation-free Java-compatible optional trimming",
    )?;
    require_contains(
        STRING_UTILS_ENGINE,
        &string_utils_engine,
        "pub fn resolve_cglib_field_name(value: &str)",
        "shared Java field-name normalization",
    )?;

    let class_utils_adapter = read(CLASS_UTILS_ADAPTER)?;
    require_contains(
        CLASS_UTILS_ADAPTER,
        &class_utils_adapter,
        "T::schema()",
        "derive-owned field metadata",
    )?;
    require_absent(
        CLASS_UTILS_ADAPTER,
        &class_utils_adapter,
        "placeholder",
        "reflection placeholder API",
    )?;

    let field_utils_adapter = read(FIELD_UTILS_ADAPTER)?;
    require_contains(
        FIELD_UTILS_ADAPTER,
        &field_utils_adapter,
        "easyexcel_utils::string_utils::resolve_cglib_field_name(name)",
        "foundation-owned field-name normalization",
    )?;
    require_contains(
        FIELD_UTILS_ADAPTER,
        &field_utils_adapter,
        "T::schema().iter().find",
        "derive-owned field lookup",
    )?;
    for obsolete in [
        "easyexcel/src",
        "crates/easyexcel/src/read/read.rs",
        "crates/easyexcel/src/read/read/metadata.rs",
        "crates/easyexcel/src/util/member_utils.rs",
    ] {
        require_path_absent(obsolete, "obsolete facade placeholder")?;
    }

    println!(
        "facade-boundary-audit ok: {} engine dependencies, no low-level direct dependencies",
        REQUIRED_ENGINE_DEPENDENCIES.len()
    );
    Ok(())
}

fn read(path: &str) -> TaskResult<String> {
    if !Path::new(path).is_file() {
        return Err(format!("missing {path}").into());
    }
    Ok(fs::read_to_string(path)?)
}

fn dependency_names(manifest: &str) -> BTreeSet<&str> {
    let mut dependencies = BTreeSet::new();
    let mut in_dependencies = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        if !in_dependencies || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            let name = name.trim();
            dependencies.insert(name.strip_suffix(".workspace").unwrap_or(name));
        }
    }
    dependencies
}

fn production_prefix(source: &str) -> &str {
    source
        .split_once("#[cfg(test)]")
        .map_or(source, |(production, _)| production)
}

fn require_contains(path: &str, source: &str, needle: &str, purpose: &str) -> TaskResult {
    if source.contains(needle) {
        return Ok(());
    }
    Err(format!("{path} must contain {needle:?} ({purpose})").into())
}

fn require_absent(path: &str, source: &str, needle: &str, purpose: &str) -> TaskResult {
    if !source.contains(needle) {
        return Ok(());
    }
    Err(format!("{path} must not contain {needle:?} ({purpose})").into())
}

fn require_no_wildcard_imports(path: &str, source: &str) -> TaskResult {
    let wildcard_import = source.lines().map(str::trim).find(|line| {
        (line.starts_with("use ") || line.starts_with("pub use ")) && line.contains("::*")
    });
    if wildcard_import.is_none() {
        return Ok(());
    }
    Err(format!(
        "{path} must not contain wildcard imports: {}",
        wildcard_import.unwrap_or_default()
    )
    .into())
}

fn require_path_absent(path: &str, purpose: &str) -> TaskResult {
    if !Path::new(path).exists() {
        return Ok(());
    }
    Err(format!("{path} must not exist ({purpose})").into())
}

fn require_tree_absent_case_insensitive(root: &str, needle: &str, purpose: &str) -> TaskResult {
    let needle = needle.to_ascii_lowercase();
    let mut pending = vec![Path::new(root).to_path_buf()];
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                pending.push(entry?.path());
            }
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path)?;
        if source.to_ascii_lowercase().contains(&needle) {
            return Err(
                format!("{} must not contain {needle:?} ({purpose})", path.display()).into(),
            );
        }
    }
    Ok(())
}
