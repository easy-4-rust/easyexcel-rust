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
const MODEL_STORED_ROW_ENGINE: &str = "crates/easyexcel-model/src/model/stored_row.rs";
const XLSX_FACADE: &str = "crates/easyexcel/src/xlsx.rs";
const XLS_RECORD_DISPATCHER: &str = "crates/easyexcel/src/analysis/v03/xls_record_dispatcher.rs";
const XLS_OBJ_HANDLER: &str =
    "crates/easyexcel/src/analysis/v03/handlers/obj_record_handler.rs";
const STYLE_UTIL_ADAPTER: &str = "crates/easyexcel/src/util/style_util.rs";
const FACADE_ERROR: &str = "crates/easyexcel/src/support/excel_error.rs";
const XLS_TEMPLATE_ADAPTER: &str = "crates/easyexcel/src/write/xls_adapter/template.rs";
const XLSX_TEMPLATE_ADAPTER: &str = "crates/easyexcel/src/template/template_writer.rs";
const ROW_PROCESSING_ADAPTER: &str = "crates/easyexcel/src/read/row_processing.rs";
const TEMPLATE_WRITE_ADAPTER: &str = "crates/easyexcel/src/write/template_write.rs";
const READ_HELPERS_ADAPTER: &str = "crates/easyexcel/src/read/read_helpers.rs";
const EXCEL_WRITER_CORE: &str = "crates/easyexcel/src/write/excel_writer_core.rs";
const STRING_UTILS_ENGINE: &str = "crates/easyexcel-utils/src/utils/string_utils.rs";
const CLASS_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/class_utils.rs";
const FIELD_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/field_utils.rs";

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
    for module in ["csv", "formula", "format", "io", "model", "tabular", "xls", "xlsx"] {
        require_contains(
            FACADE_LIB,
            &facade,
            &format!("pub mod {module};"),
            "foundation API facade module",
        )?;

        let module_path = format!("crates/easyexcel/src/{module}.rs");
        let module_source = read(&module_path)?;
        require_absent(
            &module_path,
            &module_source,
            "::*",
            "wildcard foundation API re-export",
        )?;
    }
    for path in FOUNDATION_ADAPTERS {
        let source = read(path)?;
        require_absent(path, &source, "::*", "wildcard foundation adapter re-export")?;
    }
    for path in JAVA_TRIM_ADAPTERS {
        let source = read(path)?;
        require_contains(
            path,
            &source,
            "easyexcel_utils::string_utils::java_trim",
            "shared Java-compatible trimming",
        )?;
        require_absent(path, &source, ".trim()", "Rust Unicode trim in Java adapter")?;
    }

    let ehcache = read(EHCACHE_COMPAT)?;
    require_contains(
        EHCACHE_COMPAT,
        &ehcache,
        "MokaCache as Ehcache",
        "Java-compatible alias",
    )?;
    require_absent(EHCACHE_COMPAT, &ehcache, "struct Ehcache", "Ehcache implementation")?;
    require_absent(EHCACHE_COMPAT, &ehcache, "moka::", "direct Moka dependency")?;

    let moka_adapter = read(MOKA_ADAPTER)?;
    require_contains(
        MOKA_ADAPTER,
        &moka_adapter,
        "SharedStringCachePolicy",
        "engine-owned cache policy",
    )?;
    require_absent(MOKA_ADAPTER, &moka_adapter, "moka::", "direct Moka implementation")?;

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

    let row_processing_adapter = read(ROW_PROCESSING_ADAPTER)?;
    require_contains(
        ROW_PROCESSING_ADAPTER,
        &row_processing_adapter,
        "easyexcel_io::select_sheet_names(names, selection, auto_trim)",
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
            dependencies.insert(name.trim());
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

fn require_path_absent(path: &str, purpose: &str) -> TaskResult {
    if !Path::new(path).exists() {
        return Ok(());
    }
    Err(format!("{path} must not exist ({purpose})").into())
}
