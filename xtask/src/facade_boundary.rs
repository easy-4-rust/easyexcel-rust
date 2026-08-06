//! `easyexcel` 门面与基础引擎 crate 的依赖边界审计。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::TaskResult;

const FACADE_MANIFEST: &str = "crates/easyexcel/Cargo.toml";
const FACADE_LIB: &str = "crates/easyexcel/src/lib.rs";
const EHCACHE_COMPAT: &str = "crates/easyexcel/src/cache/ehcache.rs";
const MOKA_ADAPTER: &str = "crates/easyexcel/src/cache/moka_cache.rs";
const OUTPUT_STREAM_COMPAT: &str = "crates/easyexcel/src/write/excel_output_stream.rs";
const IO_ROW_RANGE_ENGINE: &str = "crates/easyexcel-io/src/io/row_range.rs";
const IO_SHEET_SELECTION_ENGINE: &str = "crates/easyexcel-io/src/io/sheet_selection.rs";
const IO_FORMAT_ENGINE: &str = "crates/easyexcel-io/src/io/format.rs";
const IO_GZIP_CELL_ENGINE: &str = "crates/easyexcel-io/src/io/gzip_cell_record.rs";
const MODEL_STORED_ROW_ENGINE: &str = "crates/easyexcel-model/src/model/stored_row.rs";
const XLSX_FACADE: &str = "crates/easyexcel/src/xlsx.rs";
const XLS_RECORD_DISPATCHER: &str = "crates/easyexcel/src/analysis/v03/xls_record_dispatcher.rs";
const XLS_OBJ_HANDLER: &str = "crates/easyexcel/src/analysis/v03/handlers/obj_record_handler.rs";
const STYLE_UTIL_ADAPTER: &str = "crates/easyexcel/src/util/style_util.rs";
const FACADE_ERROR: &str = "crates/easyexcel/src/support/excel_error.rs";
const XLS_TEMPLATE_ADAPTER: &str = "crates/easyexcel/src/write/xls_adapter/template.rs";
const XLSX_TEMPLATE_ADAPTER: &str = "crates/easyexcel/src/template/template_writer.rs";
const XLSX_TEMPLATE_SELECTION_ENGINE: &str = "crates/easyexcel-xlsx/src/xlsx/template_source.rs";
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

    let ehcache = read(EHCACHE_COMPAT)?;
    require_contains(
        EHCACHE_COMPAT,
        &ehcache,
        "MokaCache as Ehcache",
        "Java-compatible alias",
    )?;
    require_absent(
        EHCACHE_COMPAT,
        &ehcache,
        "struct Ehcache",
        "Ehcache implementation",
    )?;
    require_absent(EHCACHE_COMPAT, &ehcache, "moka::", "direct Moka dependency")?;

    let moka_adapter = read(MOKA_ADAPTER)?;
    require_contains(
        MOKA_ADAPTER,
        &moka_adapter,
        "DEFAULT_MAX_MOKA_ACTIVE_BATCH_COUNT",
        "Moka-native default naming",
    )?;
    require_contains(
        MOKA_ADAPTER,
        &moka_adapter,
        "SharedStringCachePolicy",
        "engine-owned cache policy",
    )?;
    require_absent(
        MOKA_ADAPTER,
        &moka_adapter,
        "moka::",
        "direct Moka implementation",
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
