/// 对应 Java：无直接对应对象；Rust 架构扩展。 校验门面只依赖基础引擎，不直接依赖格式、压缩、加密或缓存实现库。
// 该函数是一份按源码路径顺序执行的架构契约清单；保持单一审计入口可以让失败位置与
// 规范条目一一对应，拆分会削弱对完整门面边界的可审计性。
#[allow(clippy::too_many_lines)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
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

    let cache_engine = read_module_family(CACHE_ENGINE)?;
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

    let cache_policy_engine = read_module_family(CACHE_POLICY_ENGINE)?;
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

    let xlsx_event_reader_engine = read_module_family(XLSX_EVENT_READER_ENGINE)?;
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
        let source = read_module_family(path)?;
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

    let moka_adapter = read_module_family(MOKA_ADAPTER)?;
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

    let file_cache_adapter = read_module_family(FILE_CACHE_ADAPTER)?;
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

    let csv_encoding_adapter = read_module_family(CSV_ENCODING_ADAPTER)?;
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

    for (engine_path, adapter_path, symbol, adapter_marker) in CSV_MODEL_BOUNDARIES {
        let engine = read_module_family(engine_path)?;
        require_contains(
            engine_path,
            &engine,
            &format!("pub struct {symbol}"),
            "CSV-engine-owned public model",
        )?;
        let adapter = read(adapter_path)?;
        require_contains(
            adapter_path,
            &adapter,
            adapter_marker,
            "thin CSV facade type adapter",
        )?;
        require_absent(
            adapter_path,
            &adapter,
            &format!("pub struct {symbol}"),
            "facade-owned duplicate CSV model",
        )?;
    }

    let excel_type_adapter = read_module_family(EXCEL_TYPE_ADAPTER)?;
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

    let model_stored_row_engine = read_module_family(MODEL_STORED_ROW_ENGINE)?;
    require_contains(
        MODEL_STORED_ROW_ENGINE,
        &model_stored_row_engine,
        "self.stored_range()",
        "engine-owned sparse model bounds",
    )?;

    let chart_model = read_module_family(MODEL_CHART_MUTATION_ENGINE)?;
    require_contains(
        MODEL_CHART_MUTATION_ENGINE,
        &chart_model,
        "pub struct ChartMutation",
        "model-owned backend-neutral chart request",
    )?;
    for path in FACADE_CHART_ADAPTERS {
        let source = read(path)?;
        require_contains(
            path,
            &source,
            "pub use easyexcel_model::",
            "thin facade chart-model re-export",
        )?;
        require_absent(path, &source, "pub struct Chart", "facade-owned chart model")?;
        require_absent(path, &source, "pub enum Chart", "facade-owned chart model")?;
    }
    let xlsx_chart_engine = read_module_family(XLSX_GENERATED_CHART_ENGINE)?;
    require_contains(
        XLSX_GENERATED_CHART_ENGINE,
        &xlsx_chart_engine,
        "pub fn add_chart(workbook: &mut Workbook, mutation: &ChartMutation)",
        "XLSX-engine-owned chart compilation",
    )?;
    let xls_chart_engine = read_module_family(XLS_GENERATED_CHART_ENGINE)?;
    require_contains(
        XLS_GENERATED_CHART_ENGINE,
        &xls_chart_engine,
        "pub fn add_chart_mutation(",
        "XLS-engine-owned chart compilation",
    )?;
    for (path, forbidden) in [
        (XLSX_COMMENT_MUTATION_ADAPTER, "generation::Chart::new"),
        (XLS_CELL_EMISSION_ADAPTER, "Biff8Chart::new"),
        (XLS_CELL_EMISSION_ADAPTER, "fn checked_chart_row"),
    ] {
        let source = read_module_family(path)?;
        require_absent(path, &source, forbidden, "facade-owned chart compilation")?;
    }

    let xls_generated_cell = read(XLS_GENERATED_CELL_ENGINE)?;
    require_contains(
        XLS_GENERATED_CELL_ENGINE,
        &xls_generated_cell,
        "pub enum GeneratedBiff8CellValue",
        "XLS-engine-owned scalar cell compilation",
    )?;
    for path in [XLS_CELL_EMISSION_ADAPTER, XLS_TEMPLATE_ADAPTER] {
        let source = read(path)?;
        require_absent(
            path,
            &source,
            "Biff8Value::",
            "facade-owned BIFF8 value compilation",
        )?;
        require_absent(
            path,
            &source,
            "Biff8RichText::new",
            "facade-owned BIFF8 rich-text compilation",
        )?;
    }

    let xlsx_generation = read_module_family(XLSX_GENERATION_ENGINE)?;
    require_contains(
        XLSX_GENERATION_ENGINE,
        &xlsx_generation,
        "pub fn write_rich_string_with_font_specs(",
        "XLSX-engine-owned rich-text font compilation",
    )?;
    require_contains(
        XLSX_GENERATION_ENGINE,
        &xlsx_generation,
        "pub fn compile_blank_format_workbook(formats: &[Format])",
        "XLSX-engine-owned template style workbook compilation",
    )?;

    let fill_config_owner = read(FILL_CONFIG_OWNER)?;
    for required in [
        "direction: Option<FillDirection>",
        "force_new_row: Option<bool>",
        "auto_style: Option<bool>",
        "self.direction.get_or_insert(FillDirection::Vertical)",
        "self.force_new_row.get_or_insert(false)",
        "self.auto_style.get_or_insert(true)",
    ] {
        require_contains(
            FILL_CONFIG_OWNER,
            &fill_config_owner,
            required,
            "Java-nullable FillConfig lifecycle",
        )?;
    }
    let builder_fill_config = read(BUILDER_FILL_CONFIG_ADAPTER)?;
    require_contains(
        BUILDER_FILL_CONFIG_ADAPTER,
        &builder_fill_config,
        "pub use crate::write::metadata::fill::fill_config::FillConfig",
        "single FillConfig owner",
    )?;
    require_absent(
        BUILDER_FILL_CONFIG_ADAPTER,
        &builder_fill_config,
        "pub struct FillConfig",
        "duplicate facade FillConfig owner",
    )?;
    require_path_absent(
        ORPHAN_WRITER_SHEET_BUILDER,
        "orphan duplicate ExcelWriterSheetBuilder source",
    )?;

    let web_header_engine = read(WEB_HEADER_ENGINE)?;
    require_contains(
        WEB_HEADER_ENGINE,
        &web_header_engine,
        "pub fn excel_attachment_content_disposition(file_name: &str) -> String",
        "framework-neutral attachment encoding",
    )?;
    for path in WEB_HEADER_ADAPTERS.iter().chain(WEB_RESPONSE_ADAPTERS) {
        let source = read(path)?;
        require_absent(
            path,
            &source,
            "urlencoding::encode",
            "framework adapter owned attachment encoding",
        )?;
    }

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

    let xls_sax_adapter = read_module_family(XLS_SAX_ADAPTER)?;
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

    let ooxml_sax_adapter = read_module_family(XLSX_SAX_ADAPTER)?;
    let ooxml_sax_production = production_prefix(&ooxml_sax_adapter);
    for needle in ["list_xlsx_sheets(&path, &options)", "read_xlsx::<T, L>("] {
        require_contains(
            XLSX_SAX_ADAPTER,
            ooxml_sax_production,
            needle,
            "XLSX-engine-backed facade analysis",
        )?;
    }
    for forbidden in ["quick_xml::", "zip::", "ZipArchive", "from_utf8"] {
        require_absent(
            XLSX_SAX_ADAPTER,
            ooxml_sax_production,
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
    for path in ANALYSIS_PUBLIC_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "pub ", "Java analysis public owner")?;
        require_contains(path, &source, "对应 Java", "Java analysis source ownership")?;
        require_absent(path, &source, "todo!()", "analysis implementation")?;
        require_absent(path, &source, "unimplemented!()", "analysis implementation")?;
    }
    for path in CONTEXT_PUBLIC_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "pub ", "Java context public owner")?;
        require_contains(path, &source, "对应 Java", "Java context source ownership")?;
        require_absent(path, &source, "todo!()", "context implementation")?;
        require_absent(path, &source, "unimplemented!()", "context implementation")?;
    }
    for path in UTILITY_PUBLIC_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "pub ", "Java utility public adapter")?;
        require_contains(path, &source, "Java", "Java utility source ownership")?;
        require_absent(path, &source, "todo!()", "utility implementation")?;
        require_absent(path, &source, "unimplemented!()", "utility implementation")?;
    }
    for path in READ_RUNTIME_PUBLIC_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "pub ", "Java read-runtime public adapter")?;
        require_contains(path, &source, "Java", "Java read-runtime source ownership")?;
        require_absent(path, &source, "todo!()", "read-runtime implementation")?;
        require_absent(path, &source, "unimplemented!()", "read-runtime implementation")?;
    }
    for path in CORE_METADATA_PUBLIC_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "pub ", "Java core-metadata public adapter")?;
        require_contains(path, &source, "Java", "Java core-metadata source ownership")?;
        require_absent(path, &source, "todo!()", "core-metadata implementation")?;
        require_absent(path, &source, "unimplemented!()", "core-metadata implementation")?;
    }
    for path in WRITE_RUNTIME_PUBLIC_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "pub ", "Java write-runtime public adapter")?;
        require_contains(path, &source, "Java", "Java write-runtime source ownership")?;
        require_absent(path, &source, "todo!()", "write-runtime implementation")?;
        require_absent(path, &source, "unimplemented!()", "write-runtime implementation")?;
    }
    for (path, needle, purpose) in [
        (
            "crates/easyexcel-csv/src/csv/csv_data_format.rs",
            "switch_builtin_formats_for_locale(locale)",
            "locale-specific CSV built-in formats",
        ),
        (
            "crates/easyexcel/src/write/metadata/row_data.rs",
            "fn get(&self, index: usize) -> Option<&CellValue>",
            "RowData Java get contract",
        ),
        (
            "crates/easyexcel/src/write/metadata/row_data.rs",
            "fn size(&self) -> usize",
            "RowData Java size contract",
        ),
        (
            "crates/easyexcel/src/write/merge/loop_merge_strategy.rs",
            "pub fn with_column(each_rows: i32, column_index: i32)",
            "LoopMergeStrategy two-argument constructor",
        ),
        (
            "crates/easyexcel/src/write/merge/loop_merge_strategy.rs",
            "pub fn from_property(",
            "LoopMergeStrategy property constructor",
        ),
        (
            "crates/easyexcel/src/write/metadata/write_workbook.rs",
            "pub excel_type_override: Option<crate::support::ExcelTypeEnum>",
            "nullable Java WriteWorkbook excel type",
        ),
        (
            "crates/easyexcel/src/write/metadata/write_workbook.rs",
            "pub charset_override: Option<CsvCharset>",
            "nullable Java WriteWorkbook charset",
        ),
        (
            "crates/easyexcel/src/write/metadata/write_sheet.rs",
            "java_sheet_no: Option<i32>",
            "nullable Java WriteSheet sheet number",
        ),
        (
            "crates/easyexcel/src/write/metadata/write_table.rs",
            "java_table_no: Option<i32>",
            "nullable Java WriteTable table number",
        ),
        (
            "crates/easyexcel/src/write/metadata/write_basic_parameter.rs",
            "impl std::hash::Hash for WriteBasicParameter",
            "WriteBasicParameter Lombok value semantics",
        ),
        (
            "crates/easyexcel/src/write/property/excel_write_head_property.rs",
            "impl std::hash::Hash for ExcelWriteHeadProperty",
            "ExcelWriteHeadProperty Lombok value semantics",
        ),
        (
            "crates/easyexcel/src/write/executor/excel_write_add_executor.rs",
            "append_rows_to_worksheet",
            "writer-engine-backed add executor",
        ),
        (
            "crates/easyexcel/src/write/executor/excel_write_fill_executor.rs",
            "delegate: Option<&'a mut dyn WriteFillExecutor>",
            "stateful template-engine-backed fill executor",
        ),
        (
            "crates/easyexcel/src/write/metadata/fill/fill_config.rs",
            "if self.has_init",
            "FillConfig delayed one-time initialization",
        ),
    ] {
        let source = read_module_family(path)?;
        require_contains(path, &source, needle, purpose)?;
    }
    let csv_data_format_adapter = read("crates/easyexcel/src/metadata/csv/csv_data_format.rs")?;
    require_contains(
        "crates/easyexcel/src/metadata/csv/csv_data_format.rs",
        &csv_data_format_adapter,
        "pub use easyexcel_csv::CsvDataFormat",
        "CSV-engine-owned data format",
    )?;
    require_absent(
        "crates/easyexcel/src/metadata/csv/csv_data_format.rs",
        &csv_data_format_adapter,
        "pub struct CsvDataFormat",
        "facade-owned duplicate CSV data format",
    )?;
    for (path, needle, purpose) in [
        (
            "crates/easyexcel/src/metadata/head.rs",
            "pub force_index: Option<bool>",
            "nullable Java Head.forceIndex state",
        ),
        (
            "crates/easyexcel/src/metadata/head.rs",
            "pub force_name: Option<bool>",
            "nullable Java Head.forceName state",
        ),
        (
            "crates/easyexcel/src/metadata/global_configuration.rs",
            "pub const fn get_use1904windowing(&self) -> bool",
            "exact Java use1904windowing getter spelling",
        ),
        (
            "crates/easyexcel/src/metadata/global_configuration.rs",
            "pub const fn set_use1904windowing(&mut self, value: bool)",
            "exact Java use1904windowing setter spelling",
        ),
        (
            "crates/easyexcel/src/metadata/font.rs",
            "pub fn get_font_name(&self) -> Option<&str>",
            "deprecated Font Java getter",
        ),
        (
            "crates/easyexcel/src/metadata/font.rs",
            "pub const fn get_font_height_in_points(&self) -> i16",
            "deprecated Font height Java getter",
        ),
        (
            "crates/easyexcel/src/metadata/cell.rs",
            "fn get_row_index(&self) -> Option<i32>",
            "Cell Java getter compatibility",
        ),
        (
            "crates/easyexcel/src/metadata/cell.rs",
            "fn get_column_index(&self) -> Option<i32>",
            "Cell Java getter compatibility",
        ),
    ] {
        let source = read_module_family(path)?;
        require_contains(path, &source, needle, purpose)?;
    }
    for (path, engine_symbol, duplicate) in [
        (
            "crates/easyexcel/src/metadata/format/data_formatter.rs",
            "pub use easyexcel_format::",
            "pub struct DataFormatter",
        ),
        (
            "crates/easyexcel/src/metadata/format/excel_general_number_format.rs",
            "pub use easyexcel_format::",
            "pub struct ExcelGeneralNumberFormat",
        ),
    ] {
        let source = read(path)?;
        require_contains(path, &source, engine_symbol, "format-engine-owned implementation")?;
        require_absent(path, &source, duplicate, "facade-owned duplicate format implementation")?;
    }
    for (path, needle, purpose) in [
        (
            "crates/easyexcel/src/cache/read_cache.rs",
            "fn init(&mut self, _analysis_context: &crate::AnalysisContext)",
            "ReadCache AnalysisContext lifecycle signature",
        ),
        (
            "crates/easyexcel/src/cache/selector/simple_read_cache_selector.rs",
            "max_use_map_cache_size_mb: Option<i64>",
            "nullable Java cache selector state",
        ),
        (
            "crates/easyexcel/src/event/analysis_event_listener.rs",
            "pub struct AnalysisEventListenerAdapter",
            "AnalysisEventListener invokeHead to invokeHeadMap bridge",
        ),
        (
            "crates/easyexcel/src/read/listener/ignore_exception_read_listener.rs",
            "pub struct IgnoreExceptionListenerAdapter",
            "IgnoreExceptionReadListener dynamic-dispatch bridge",
        ),
        (
            "crates/easyexcel/src/event/abstract_ignore_exception_read_listener.rs",
            "pub struct AbstractIgnoreExceptionListenerAdapter",
            "AbstractIgnoreExceptionReadListener dynamic-dispatch bridge",
        ),
        (
            "crates/easyexcel/src/event/handler.rs",
            "impl<T: Handler + ?Sized> Order for T",
            "Handler extends Order contract",
        ),
    ] {
        let source = read_module_family(path)?;
        require_contains(path, &source, needle, purpose)?;
    }
    for (path, needle, purpose) in [
        (
            "crates/easyexcel-utils/src/utils/list_utils.rs",
            ".saturating_add(expected_size / 10)",
            "EasyExcel v4.0.3 ListUtils capacity formula",
        ),
        (
            "crates/easyexcel-utils/src/utils/string_utils.rs",
            "const fn is_java_digit(character: char)",
            "Java 8 Character digit semantics",
        ),
        (
            "crates/easyexcel-utils/src/utils/string_utils.rs",
            "const fn is_java_whitespace(character: char)",
            "Java 8 Character whitespace semantics",
        ),
        (
            "crates/easyexcel-model/src/model/dates.rs",
            "pub fn excel_parts_to_datetime(",
            "model-owned DateUtils setCalendar semantics",
        ),
        (
            "crates/easyexcel/src/util/date_utils.rs",
            "round_seconds: bool",
            "DateUtils roundSeconds parameter preservation",
        ),
        (
            "crates/easyexcel/src/util/field_utils.rs",
            "pub fn get_field_class_from_map(",
            "typed BeanMap property-type dispatch",
        ),
        (
            "crates/easyexcel/src/util/easy_excel_temp_file_creation_strategy.rs",
            "pub use easyexcel_io::EasyExcelTempFileCreationStrategy;",
            "I/O-owned Java utility type re-export",
        ),
    ] {
        let source = read_module_family(path)?;
        require_contains(path, &source, needle, purpose)?;
    }
    let analysis_context_contract = read_module_family(
        "crates/easyexcel/src/context/analysis_context.rs",
    )?;
    for needle in [
        "pub trait AnalysisContextLifecycle",
        "fn analysis_event_processor(",
        "fn current_sheet(&mut self",
        "fn read_workbook_holder(",
    ] {
        require_contains(
            "crates/easyexcel/src/context/analysis_context.rs",
            &analysis_context_contract,
            needle,
            "Java AnalysisContext complete lifecycle supertrait",
        )?;
    }
    for path in [
        "crates/easyexcel/src/context/csv/csv_read_context.rs",
        "crates/easyexcel/src/context/xls/xls_read_context.rs",
        "crates/easyexcel/src/context/xlsx/xlsx_read_context.rs",
    ] {
        let source = read_module_family(path)?;
        require_contains(
            path,
            &source,
            ": AnalysisContextLifecycle",
            "format read context extends AnalysisContext lifecycle",
        )?;
    }
    for path in [
        "crates/easyexcel/src/context/csv/default_csv_read_context.rs",
        "crates/easyexcel/src/context/xls/default_xls_read_context.rs",
        "crates/easyexcel/src/context/xlsx/default_xlsx_read_context.rs",
    ] {
        let source = read_module_family(path)?;
        require_contains(
            path,
            &source,
            "impl AnalysisContextLifecycle for Default",
            "concrete format context delegates shared lifecycle",
        )?;
    }

    let style_util_adapter = read_module_family(STYLE_UTIL_ADAPTER)?;
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

    let data_formatter_engine = read_module_family(DATA_FORMATTER_ENGINE)?;
    require_contains(
        DATA_FORMATTER_ENGINE,
        &data_formatter_engine,
        "pub struct DataFormatter",
        "format-engine-owned DataFormatter state",
    )?;
    let data_formatter_adapter = read(DATA_FORMATTER_ADAPTER)?;
    require_contains(
        DATA_FORMATTER_ADAPTER,
        &data_formatter_adapter,
        "pub use easyexcel_format::{",
        "thin DataFormatter facade adapter",
    )?;
    require_absent(
        DATA_FORMATTER_ADAPTER,
        &data_formatter_adapter,
        "pub struct DataFormatter",
        "facade-owned DataFormatter state",
    )?;

    let general_number_format_engine = read_module_family(GENERAL_NUMBER_FORMAT_ENGINE)?;
    require_contains(
        GENERAL_NUMBER_FORMAT_ENGINE,
        &general_number_format_engine,
        "pub struct ExcelGeneralNumberFormat",
        "format-engine-owned General formatter state",
    )?;
    let general_number_format_adapter = read(GENERAL_NUMBER_FORMAT_ADAPTER)?;
    require_contains(
        GENERAL_NUMBER_FORMAT_ADAPTER,
        &general_number_format_adapter,
        "pub use easyexcel_format::{ExcelGeneralNumberFormat",
        "thin General formatter facade adapter",
    )?;
    require_absent(
        GENERAL_NUMBER_FORMAT_ADAPTER,
        &general_number_format_adapter,
        "pub struct ExcelGeneralNumberFormat",
        "facade-owned General formatter state",
    )?;

    let number_data_formatter_engine = read(NUMBER_DATA_FORMATTER_ENGINE)?;
    for needle in [
        "static DATA_FORMATTER: RefCell<Option<DataFormatter>>",
        "pub fn format_number_data(",
        "pub fn remove_thread_local_cache()",
    ] {
        require_contains(
            NUMBER_DATA_FORMATTER_ENGINE,
            &number_data_formatter_engine,
            needle,
            "format-engine-owned thread-local formatter lifecycle",
        )?;
    }
    let number_data_formatter_adapter = read(NUMBER_DATA_FORMATTER_ADAPTER)?;
    for needle in [
        "easyexcel_format::format_number_data(",
        "easyexcel_format::remove_thread_local_cache()",
    ] {
        require_contains(
            NUMBER_DATA_FORMATTER_ADAPTER,
            &number_data_formatter_adapter,
            needle,
            "thin NumberDataFormatterUtils facade adapter",
        )?;
    }
    for forbidden in ["thread_local!", "RefCell<Option<DataFormatter>>"] {
        require_absent(
            NUMBER_DATA_FORMATTER_ADAPTER,
            &number_data_formatter_adapter,
            forbidden,
            "facade-owned formatter cache",
        )?;
    }

    let position_utils_engine = read(POSITION_UTILS_ENGINE)?;
    for symbol in ["get_row_by_row_tagt", "get_row", "get_col"] {
        require_contains(
            POSITION_UTILS_ENGINE,
            &position_utils_engine,
            &format!("pub fn {symbol}"),
            "utils-engine-owned position algorithm",
        )?;
    }
    let position_utils_adapter = read(POSITION_UTILS_ADAPTER)?;
    require_contains(
        POSITION_UTILS_ADAPTER,
        &position_utils_adapter,
        "pub use easyexcel_utils::position_utils",
        "thin PositionUtils facade adapter",
    )?;
    for forbidden in ["parse::<", "char_indices", "saturating_mul"] {
        require_absent(
            POSITION_UTILS_ADAPTER,
            &position_utils_adapter,
            forbidden,
            "facade-owned position algorithm",
        )?;
    }

    let ooxml_constants_engine = read(OOXML_CONSTANTS_ENGINE)?;
    for symbol in [
        "pub const DIMENSION_TAG",
        "pub const CELL_FORMULA_TAG",
        "pub const ATTRIBUTE_RID",
        "pub const SHAREDSTRINGS_RPH_TAG",
    ] {
        require_contains(
            OOXML_CONSTANTS_ENGINE,
            &ooxml_constants_engine,
            symbol,
            "XLSX-engine-owned OOXML protocol constant",
        )?;
    }
    let ooxml_constants_adapter = read(OOXML_CONSTANTS_ADAPTER)?;
    require_contains(
        OOXML_CONSTANTS_ADAPTER,
        &ooxml_constants_adapter,
        "pub use easyexcel_xlsx::xlsx::ooxml_constants::{",
        "thin ExcelXmlConstants facade adapter",
    )?;
    for forbidden in ["= \"dimension\"", "= \"r:id\"", "= \"rPh\""] {
        require_absent(
            OOXML_CONSTANTS_ADAPTER,
            &ooxml_constants_adapter,
            forbidden,
            "facade-owned duplicate OOXML protocol constant",
        )?;
    }

    let date_engine = read_module_family(DATE_ENGINE)?;
    for needle in [
        "pub const DATE_FORMAT_19",
        "pub const DAY_MILLISECONDS",
        "pub fn infer_java_date_pattern",
        "pub fn parse_java_date",
    ] {
        require_contains(
            DATE_ENGINE,
            &date_engine,
            needle,
            "model-owned date protocol and conversion",
        )?;
    }
    let date_utils_adapter = read(DATE_UTILS_ADAPTER)?;
    require_contains(
        DATE_UTILS_ADAPTER,
        &date_utils_adapter,
        "pub use easyexcel_model::{",
        "thin DateUtils constant adapter",
    )?;
    require_contains(
        DATE_UTILS_ADAPTER,
        &date_utils_adapter,
        "easyexcel_model::infer_java_date_pattern(value)",
        "model-owned Java date-pattern inference",
    )?;
    for forbidden in ["value.chars().count()", "= \"yyyy-MM-dd\""] {
        require_absent(
            DATE_UTILS_ADAPTER,
            &date_utils_adapter,
            forbidden,
            "facade-owned date parsing algorithm",
        )?;
    }

    let builtin_formats_engine = read_module_family(BUILTIN_FORMATS_ENGINE)?;
    for needle in [
        "pub static BUILTIN_FORMATS_ALL_LANGUAGES",
        "pub static BUILTIN_FORMATS_CN",
        "pub static BUILTIN_FORMATS_US",
        "pub fn get_builtin_format_for_locale",
    ] {
        require_contains(
            BUILTIN_FORMATS_ENGINE,
            &builtin_formats_engine,
            needle,
            "format-engine-owned builtin format table",
        )?;
    }
    let builtin_formats_adapter = read(BUILTIN_FORMATS_ADAPTER)?;
    require_contains(
        BUILTIN_FORMATS_ADAPTER,
        &builtin_formats_adapter,
        "pub use easyexcel_format::{",
        "thin BuiltinFormats facade adapter",
    )?;
    for forbidden in [
        "pub static BUILTIN_FORMATS_ALL_LANGUAGES",
        "LazyLock<HashMap",
        "fn build_map",
    ] {
        require_absent(
            BUILTIN_FORMATS_ADAPTER,
            &builtin_formats_adapter,
            forbidden,
            "facade-owned builtin format table",
        )?;
    }

    let number_utils_adapter = read_module_family(NUMBER_UTILS_ADAPTER)?;
    for engine_call in [
        "easyexcel_format::parse_short(value)",
        "easyexcel_format::parse_big_decimal(value)",
        "easyexcel_format::format_decimal(value, negative, pattern, rounding_mode)",
    ] {
        require_contains(
            NUMBER_UTILS_ADAPTER,
            &number_utils_adapter,
            engine_call,
            "format-engine-owned NumberUtils algorithm",
        )?;
    }
    for forbidden in [".parse::<i16>()", ".parse::<BigDecimal>()", "with_scale_round("] {
        require_absent(
            NUMBER_UTILS_ADAPTER,
            &number_utils_adapter,
            forbidden,
            "facade-owned NumberUtils parsing or rounding",
        )?;
    }

    // CommentData 保留 Java 风格的格式中立配置；NOTE/TXO/OBJ/MSODRAWING 和
    // OOXML note 的实际编码必须分别由 XLS/XLSX 引擎承载。
    let comment_data_adapter = read_module_family(COMMENT_DATA_ADAPTER)?;
    for needle in [
        "visible: Option<bool>",
        "pub const fn get_visible(&self) -> Option<bool>",
    ] {
        require_contains(
            COMMENT_DATA_ADAPTER,
            &comment_data_adapter,
            needle,
            "format-neutral comment visibility configuration",
        )?;
    }

    let xls_comment_engine = read_module_family(XLS_COMMENT_ENGINE)?;
    require_contains(
        XLS_COMMENT_ENGINE,
        &xls_comment_engine,
        "pub const fn with_visible(mut self, visible: bool) -> Self",
        "XLS-engine-owned comment visibility state",
    )?;
    let xls_comment_encoder = read_module_family(XLS_COMMENT_ENCODER)?;
    require_contains(
        XLS_COMMENT_ENCODER,
        &xls_comment_encoder,
        "let flags = if comment.visible { 0x0002u16 } else { 0u16 };",
        "XLS-engine-owned NOTE visibility encoding",
    )?;
    for path in [XLS_COMMENT_WRITE_ADAPTER, XLS_COMMENT_TEMPLATE_ADAPTER] {
        let source = read_module_family(path)?;
        require_contains(
            path,
            &source,
            ".with_visible(visible)",
            "thin XLS comment visibility adapter",
        )?;
        require_absent(path, &source, "0x0002u16", "facade-owned NOTE flag encoding")?;
    }

    let xlsx_comment_engine = read_module_family(XLSX_COMMENT_ENGINE)?;
    require_contains(
        XLSX_COMMENT_ENGINE,
        &xlsx_comment_engine,
        "note = note.set_visible(visible);",
        "XLSX-engine-owned note visibility encoding",
    )?;
    for path in [XLSX_COMMENT_ROW_ADAPTER, XLSX_COMMENT_MUTATION_ADAPTER] {
        let source = read_module_family(path)?;
        require_contains(
            path,
            &source,
            "comment.get_visible(),",
            "thin XLSX comment visibility adapter",
        )?;
        require_absent(path, &source, ".set_visible(", "facade-owned OOXML note encoding")?;
    }

    let xls_comment_sheet = read_module_family(XLS_COMMENT_SHEET_ENGINE)?;
    for needle in [
        "pub fn set_comment(&mut self, comment: Biff8Comment)",
        "pub fn remove_comment(&mut self, row: u32, col: usize) -> Result<bool>",
    ] {
        require_contains(
            XLS_COMMENT_SHEET_ENGINE,
            &xls_comment_sheet,
            needle,
            "XLS-engine-owned comment overwrite/delete semantics",
        )?;
    }

    let xls_comment_template = read_module_family(XLS_COMMENT_TEMPLATE_ENGINE)?;
    for needle in [
        "pub fn remove_comment(&mut self, sheet_name: &str, row: u32, col: usize)",
        "remove_escher_comment_shape(&record.data, u32::from(shape_id))",
        "decrement_escher_dg_count(&mut record.data)",
        "append_dgg_drawing(&mut self.records[index].data, used_shapes)",
        "decrement_existing_dgg_shapes(&mut updated[dgg_index].data, drawing_id, 1)",
        "fn append_rows_inner(",
        "fn replace_scalar_placeholders_on_sheet_inner(",
        "fn fill_collection_placeholders_inner(",
        "self.sheet_index(sheet_name)?;",
    ] {
        require_contains(
            XLS_COMMENT_TEMPLATE_ENGINE,
            &xls_comment_template,
            needle,
            "XLS-template-owned transactional record mutation",
        )?;
    }

    let xls_drawing_group = read_module_family(XLS_DRAWING_GROUP_ENGINE)?;
    require_contains(
        XLS_DRAWING_GROUP_ENGINE,
        &xls_drawing_group,
        "pub(crate) fn drawing_group_for_clusters(clusters: &[(u16, u32)])",
        "XLS-engine-owned global DGG allocation",
    )?;
    let xls_workbook_drawing_plan = read_module_family(XLS_WORKBOOK_DRAWING_PLAN)?;
    for needle in [
        "let mut drawing_clusters = Vec::new();",
        "&drawing_group_for_clusters(&drawing_clusters)",
        "sheet_drawing_plans.push((comment_drawing_id, first_chart_drawing_id))",
    ] {
        require_contains(
            XLS_WORKBOOK_DRAWING_PLAN,
            &xls_workbook_drawing_plan,
            needle,
            "single Workbook-global XLS drawing allocation plan",
        )?;
    }

    let xlsx_comment_template = read_module_family(XLSX_COMMENT_TEMPLATE_ENGINE)?;
    for needle in [
        "pub fn remove_comment(",
        "pub fn import_comment(",
        "pub fn set_template_hyperlink(",
        "pub fn set_template_image(",
        "fn import_image(&mut self, compiled_xlsx: &[u8], sheet_name: &str)",
        "remove_xml_element_by_attribute(&comments_xml, \"comment\", \"ref\", &reference)",
        "remove_vml_comment_shape(",
        "relationship_target_by_type(&source_rels, \"/comments\")",
        "with_next_vml_shape_id(&vml_xml, &source_shape)",
    ] {
        require_contains(
            XLSX_COMMENT_TEMPLATE_ENGINE,
            &xlsx_comment_template,
            needle,
            "XLSX-engine-owned comment XML/VML deletion",
        )?;
    }

    let xlsx_comment_mutation = read_module_family(XLSX_COMMENT_MUTATION_ADAPTER)?;
    for needle in [
        "fn apply_xlsx_template_cell(",
        "package.set_cell_with_decorations(",
    ] {
        require_contains(
            XLSX_COMMENT_MUTATION_ADAPTER,
            &xlsx_comment_mutation,
            needle,
            "thin XLSX template-comment compiler adapter",
        )?;
    }
    for forbidden in [
        "<comments",
        "<v:shape",
        "vmlDrawing{vml_number}",
        "import_xlsx_template_comment",
        "workbook handler image mutations require an explicit image anchor",
    ] {
        require_absent(
            XLSX_COMMENT_MUTATION_ADAPTER,
            &xlsx_comment_mutation,
            forbidden,
            "facade-owned XLSX comment package encoding",
        )?;
    }
    require_contains(
        XLSX_COMMENT_MUTATION_ADAPTER,
        &xlsx_comment_mutation,
        "CellValue::Image(image) => insert_image_data(",
        "existing XLSX image mutation implementation reused for a single image",
    )?;

    let xlsx_template_fill_engine = read_module_family(XLSX_TEMPLATE_FILL_ENGINE)?;
    for needle in [
        "pub fn replace_collection_fills_in_sheet_with_decorations(",
        "pub fn replace_scalar_cells_in_sheet_with_decorations(",
        "pub fn append_rows_to_sheet_with_decorations(",
        "fn template_decoration_placements(",
        "pub fn template_value_decorations(",
        "TemplateDecoration::Hyperlink(hyperlink.clone())",
        "TemplateDecoration::Image(image)",
    ] {
        require_contains(
            XLSX_TEMPLATE_FILL_ENGINE,
            &xlsx_template_fill_engine,
            needle,
            "XLSX-engine-owned template decoration placement",
        )?;
    }
    let xlsx_template_rich_text = read_module_family(XLSX_TEMPLATE_RICH_TEXT_ENGINE)?;
    for needle in [
        "pub struct TemplateRichText",
        "pub fn from_runs(runs: &[(FontFormatSpec, String)])",
        "fn font_properties_xml(font: &FontFormatSpec)",
    ] {
        require_contains(
            XLSX_TEMPLATE_RICH_TEXT_ENGINE,
            &xlsx_template_rich_text,
            needle,
            "XLSX-engine-owned template rich-text encoding",
        )?;
    }
    let template_fill_adapter = read_module_family(TEMPLATE_FILL_ADAPTER)?;
    for (needle, purpose) in [
        (
            "template_rich_text_cell_value(value)",
            "shared rich-text font adaptation",
        ),
        (
            "template_hyperlink_value(",
            "format-neutral template hyperlink adaptation",
        ),
        (
            "template_images_value(value, images)",
            "format-neutral template image adaptation",
        ),
    ] {
        require_contains(TEMPLATE_FILL_ADAPTER, &template_fill_adapter, needle, purpose)?;
    }
    let template_rich_value_adapter = read_module_family(TEMPLATE_WRITE_ADAPTER)?;
    for needle in [
        "template_value_decorations(",
        "pub(crate) fn set_cell_with_decorations(",
        ".set_template_comment(",
        ".set_template_hyperlink(",
        ".set_template_image(",
    ] {
        require_contains(
            TEMPLATE_WRITE_ADAPTER,
            &template_rich_value_adapter,
            needle,
            "typed preserved-template decoration handoff",
        )?;
    }
    for forbidden in [
        "<comments",
        "<v:shape",
        "vmlDrawing",
        "<hyperlink",
        "<xdr:",
        "xl/media/",
    ] {
        require_absent(
            TEMPLATE_FILL_ADAPTER,
            &template_fill_adapter,
            forbidden,
            "facade-owned template comment XML",
        )?;
    }
    let template_writer_adapter = read_module_family(TEMPLATE_WRITER_ADAPTER)?;
    for needle in [
        "replace_collection_fills_in_sheet_with_decorations(",
        "replace_scalar_cells_in_sheet_with_decorations(",
        "append_rows_to_sheet_with_decorations(",
        "package.set_template_comment(",
        "package.set_template_hyperlink(",
        "package.set_template_image(",
    ] {
        require_contains(
            TEMPLATE_WRITER_ADAPTER,
            &template_writer_adapter,
            needle,
            "stateful template decoration placement handoff",
        )?;
    }

    let xls_template_engine = read_module_family(XLS_COMMENT_TEMPLATE_ENGINE)?;
    for needle in [
        "pub fn replace_scalar_cells_on_sheet(",
        "pub fn fill_collection_cells(",
        "fn set_cell_with_xf(",
    ] {
        require_contains(
            XLS_COMMENT_TEMPLATE_ENGINE,
            &xls_template_engine,
            needle,
            "BIFF8-engine-owned typed template placement",
        )?;
    }
    let xls_template_adapter = read_module_family(XLS_TEMPLATE_ADAPTER)?;
    for needle in [
        "pub fn replace_scalar_cell_values_on_sheet(",
        "pub fn fill_collection_cell_values(",
        ".replace_scalar_cells_on_sheet(sheet_name, &cells)",
        ".fill_collection_cells(",
        "self.apply_template_decorations(decorations)?",
        "legacy XLS writing does not support images until BIFF8 Workbook drawing records are implemented",
    ] {
        require_contains(
            XLS_TEMPLATE_ADAPTER,
            &xls_template_adapter,
            needle,
            "typed BIFF8 template-value adaptation",
        )?;
    }
    require_absent(
        XLS_TEMPLATE_ADAPTER,
        &xls_template_adapter,
        "| CellValue::Images { value, .. } = value",
        "XLS template image payload silently unwrapped to its scalar value",
    )?;
    for path in [TEMPLATE_WRITER_ADAPTER, BUILDER_FILL_EXECUTOR] {
        let source = read_module_family(path)?;
        for needle in [
            "replace_scalar_cell_values_on_sheet(",
            "fill_collection_cell_values(",
        ] {
            require_contains(path, &source, needle, "typed XLS template fill handoff")?;
        }
        require_absent(
            path,
            &source,
            "value.as_text()",
            "XLS template rich value flattened to text",
        )?;
    }
    let builder_fill_executor = read_module_family(BUILDER_FILL_EXECUTOR)?;
    for (needle, purpose) in [
        (
            "write_excel_on_exception: bool",
            "template writeExcelOnException lifecycle state",
        ),
        (
            "if on_exception && !self.write_excel_on_exception",
            "template exception-output policy",
        ),
        (
            "xls.to_bytes_with_password_and_macro_policy(",
            "password and macro-policy preserving BIFF8 serialization",
        ),
        (
            "biff8_macro_policy: crate::Biff8MacroPolicy",
            "template BIFF8 macro lifecycle state",
        ),
    ] {
        require_contains(BUILDER_FILL_EXECUTOR, &builder_fill_executor, needle, purpose)?;
    }
    let template_writer = read_module_family(TEMPLATE_WRITER_ADAPTER)?;
    require_contains(
        TEMPLATE_WRITER_ADAPTER,
        &template_writer,
        "std::fs::File::create(path)?;",
        "Java-compatible empty path output on discarded template session",
    )?;
    for (needle, purpose) in [
        (
            "TemplateOutput::Managed { writer, close }",
            "type-erased facade output stream handoff",
        ),
        (
            "discard_template_output(&mut self.output, self.auto_close_stream)?",
            "stream-aware discarded template lifecycle",
        ),
    ] {
        require_contains(TEMPLATE_WRITER_ADAPTER, &template_writer, needle, purpose)?;
    }
    require_absent(
        TEMPLATE_WRITER_ADAPTER,
        &template_writer,
        "self.finished = true;\n            return write_template_bytes_to_output",
        "template session marked finished before output succeeds",
    )?;
    require_contains(
        TEMPLATE_WRITER_ADAPTER,
        &template_writer,
        "write_entries_to_output_with_password(\n            &mut self.output,\n            &entries,\n            self.auto_close_stream,\n            self.package_password.as_deref(),\n        )?;\n        self.entries = entries;",
        "template package state committed only after successful output",
    )?;
    for (needle, purpose) in [
        (
            "easyexcel_xlsx::is_encrypted_ooxml(&bytes)",
            "encrypted OOXML template detection",
        ),
        (
            "easyexcel_xlsx::decrypt_package(&bytes, password)",
            "XLSX-engine-owned template decryption",
        ),
        (
            "inner.set_package_password(password.clone())",
            "XLSX template output password propagation",
        ),
        (
            "&self.biff8_macro_policy",
            "BIFF8 template macro policy propagation",
        ),
    ] {
        require_contains(BUILDER_FILL_EXECUTOR, &builder_fill_executor, needle, purpose)?;
    }
    require_contains(
        TEMPLATE_WRITER_ADAPTER,
        &template_writer,
        "easyexcel_xlsx::encrypt_package_to(&plaintext, password, &mut encrypted)",
        "XLSX-engine-owned template output encryption",
    )?;
    let excel_builder_adapter = read_module_family("crates/easyexcel/src/excel_builder.rs")?;
    for (needle, purpose) in [
        (
            "let target = writer.take_template_output();",
            "facade writer output ownership transfer",
        ),
        (
            "executor.redirect_output(target, auto_close_stream);",
            "shared template executor output redirection",
        ),
        (
            "builder.set_active_template_fill_executor(Box::new(executor));",
            "template output ownership activates executor lifecycle",
        ),
        (
            "先完成模板解析，再移交真实流",
            "non-destructive template validation ordering",
        ),
        (
            "writer.discard_uninitialized_template_output()?;",
            "template initialization failure output cleanup",
        ),
    ] {
        require_contains(
            "crates/easyexcel/src/excel_builder.rs",
            &excel_builder_adapter,
            needle,
            purpose,
        )?;
    }
    if excel_builder_adapter
        .matches("builder.finish_on_exception()?;")
        .count()
        < 2
    {
        return Err(
            "all one-shot template fill paths must finish the active executor after fill errors"
                .into(),
        );
    }
    let excel_writer_adapter = read_module_family(
        "crates/easyexcel/src/excel_writer/write_raw_bytes_to_write_xls_batch_onto_template.rs",
    )?;
    for needle in [
        "pub(crate) fn take_template_output(",
        "pub(crate) fn discard_uninitialized_template_output(",
        "TemplateOutput::Managed",
        "close: self.close_stream.take()",
    ] {
        require_contains(
            "crates/easyexcel/src/excel_writer/write_raw_bytes_to_write_xls_batch_onto_template.rs",
            &excel_writer_adapter,
            needle,
            "stateful output stream and close callback handoff",
        )?;
    }
    let excel_builder_impl = read_module_family(
        "crates/easyexcel/src/write/excel_builder_impl.rs",
    )?;
    require_contains(
        "crates/easyexcel/src/write/excel_builder_impl.rs",
        &excel_builder_impl,
        "pub(crate) fn set_active_template_fill_executor(",
        "template executor resource activation",
    )?;
    if excel_builder_impl
        .matches("crate::excel_builder::wire_template_fill(self)?;")
        .count()
        < 3
    {
        return Err(
            "ExcelBuilderImpl must lazily wire the existing template executor for fill, ordinary write and merge"
                .into(),
        );
    }
    require_absent(
        "crates/easyexcel/src/write/excel_builder_impl.rs",
        &excel_builder_impl,
        "build through easyexcel::builder_from_writer",
        "public ExcelBuilderImpl path requiring facade-private manual fill wiring",
    )?;

    let excel_row_contract = read_module_family(EXCEL_ROW_CONTRACT)?;
    require_contains(
        EXCEL_ROW_CONTRACT,
        &excel_row_contract,
        "fn supports_static_scalar_write() -> bool",
        "fail-closed Stateful value capability contract",
    )?;
    let excel_row_derive = read_module_family(EXCEL_ROW_DERIVE)?;
    for needle in [
        "fn supports_static_scalar_write() -> bool",
        "true",
    ] {
        require_contains(
            EXCEL_ROW_DERIVE,
            &excel_row_derive,
            needle,
            "derive-proven static scalar row capability",
        )?;
    }
    let stateful_policy = read_module_family(STATEFUL_BACKEND_POLICY)?;
    require_contains(
        STATEFUL_BACKEND_POLICY,
        &stateful_policy,
        "if !T::supports_static_scalar_write()",
        "manual ExcelRow cannot be guessed streaming-safe",
    )?;
    require_contains(
        STATEFUL_BACKEND_POLICY,
        &stateful_policy,
        "self.memory_selection = WriteMemorySelection::Explicit;",
        "gzip spill is an explicit constant-memory selection",
    )?;
    let java_stateful_policy = read_module_family(JAVA_STATEFUL_BACKEND_POLICY)?;
    for needle in [
        "pub fn compress_temp_files(mut self, enabled: bool) -> Self",
        "self.memory_selection = Some(false);",
    ] {
        require_contains(
            JAVA_STATEFUL_BACKEND_POLICY,
            &java_stateful_policy,
            needle,
            "Java-style builder explicit gzip spill selection",
        )?;
    }
    let stateful_writer = read_module_family(STATEFUL_WRITER)?;
    for needle in [
        "let has_deferred_mutations = !self.mutation_plan.is_empty()?;",
        "self.ensure_backend_for_write::<T>(sheet_with_table.options(), &handlers)?;",
        "回调后的真实状态再判定一次",
    ] {
        require_contains(
            STATEFUL_WRITER,
            &stateful_writer,
            needle,
            "Stateful Auto promotion coverage",
        )?;
    }

    let comment_mutation_plan = read_module_family(COMMENT_MUTATION_PLAN)?;
    require_contains(
        COMMENT_MUTATION_PLAN,
        &comment_mutation_plan,
        "pub(crate) fn remove_comment(",
        "backend-neutral comment deletion mutation",
    )?;
    for forbidden in ["NOTE_SID", "<comment", "<v:shape"] {
        require_absent(
            COMMENT_MUTATION_PLAN,
            &comment_mutation_plan,
            forbidden,
            "format-specific comment encoding in mutation plan",
        )?;
    }

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
    let xlsx_template_adapter = read_module_family(XLSX_TEMPLATE_ADAPTER)?;
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
    let xlsx_template_selection_engine = read_module_family(XLSX_TEMPLATE_SELECTION_ENGINE)?;
    require_contains(
        XLSX_TEMPLATE_SELECTION_ENGINE,
        &xlsx_template_selection_engine,
        "pub fn equivalent(self, other: TemplateSheetSelector<'_>) -> bool",
        "template sheet equivalence algorithm",
    )?;

    let row_processing_adapter = read_module_family(ROW_PROCESSING_ADAPTER)?;
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

    let io_sheet_selection_engine = read_module_family(IO_SHEET_SELECTION_ENGINE)?;
    require_contains(
        IO_SHEET_SELECTION_ENGINE,
        &io_sheet_selection_engine,
        "pub fn matches(self, index: usize, name: Option<&str>, auto_trim: bool) -> bool",
        "streaming sheet-selection predicate",
    )?;

    let io_row_range_engine = read_module_family(IO_ROW_RANGE_ENGINE)?;
    require_contains(
        IO_ROW_RANGE_ENGINE,
        &io_row_range_engine,
        "pub fn row_is_selected(",
        "shared row-selection predicate",
    )?;

    let io_format_engine = read_module_family(IO_FORMAT_ENGINE)?;
    require_contains(
        IO_FORMAT_ENGINE,
        &io_format_engine,
        "pub fn detect_path(path: &Path) -> Result<Self>",
        "path extension and magic format detection",
    )?;
    let excel_analyser_adapter = read_module_family(EXCEL_ANALYSER_ADAPTER)?;
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

    let io_gzip_cell_engine = read_module_family(IO_GZIP_CELL_ENGINE)?;
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
    let gzip_spill_adapter = read_module_family(GZIP_SPILL_ADAPTER)?;
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

    let template_write_adapter = read_module_family(TEMPLATE_WRITE_ADAPTER)?;
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

    let xlsx_template_adapter = read_module_family(XLSX_TEMPLATE_ADAPTER)?;
    require_contains(
        XLSX_TEMPLATE_ADAPTER,
        &xlsx_template_adapter,
        "easyexcel_xlsx::OoxmlPackage::from_entries(entries.to_vec()).to_bytes()",
        "XLSX-engine-owned OOXML ZIP encoding",
    )?;

    let read_helpers_adapter = read_module_family(READ_HELPERS_ADAPTER)?;
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

    let excel_writer_core = read_module_family(EXCEL_WRITER_CORE)?;
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

    let string_utils_engine = read_module_family(STRING_UTILS_ENGINE)?;
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

    let class_utils_adapter = read_module_family(CLASS_UTILS_ADAPTER)?;
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
    for (forbidden, purpose) in [
        (
            "pub struct ContentPropertyKey",
            "nested ContentPropertyKey object in ClassUtils module",
        ),
        (
            "pub struct FieldCacheKey",
            "nested FieldCacheKey object in ClassUtils module",
        ),
    ] {
        require_absent(CLASS_UTILS_ADAPTER, &class_utils_adapter, forbidden, purpose)?;
    }

    let content_property_key_adapter = read_module_family(CONTENT_PROPERTY_KEY_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct ContentPropertyKey", "dedicated content-property cache-key owner"),
        ("clazz: Option<TypeId>", "Java Class identity carrier"),
        ("head_class: Option<TypeId>", "Java head Class identity carrier"),
        ("field_name: String", "Java field-name carrier"),
        ("#[derive(Debug, Clone, PartialEq, Eq, Hash)]", "value equality and hash contract"),
    ] {
        require_contains(
            CONTENT_PROPERTY_KEY_ADAPTER,
            &content_property_key_adapter,
            needle,
            purpose,
        )?;
    }

    let field_cache_key_adapter = read_module_family(FIELD_CACHE_KEY_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct FieldCacheKey", "dedicated field cache-key owner"),
        ("exclude_column_field_names: Vec<String>", "excluded field-name identity"),
        ("exclude_column_indexes: Vec<usize>", "excluded column-index identity"),
        ("include_column_field_names: Vec<String>", "included field-name identity"),
        ("include_column_indexes: Vec<usize>", "included column-index identity"),
        ("#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]", "value equality and hash contract"),
    ] {
        require_contains(
            FIELD_CACHE_KEY_ADAPTER,
            &field_cache_key_adapter,
            needle,
            purpose,
        )?;
    }

    let bean_map_utils_adapter = read_module_family(BEAN_MAP_UTILS_ADAPTER)?;
    for (forbidden, purpose) in [
        (
            "pub struct EasyExcelNamingPolicy",
            "nested EasyExcelNamingPolicy object in BeanMapUtils module",
        ),
        (
            "pub struct BeanMap",
            "Rust BeanMap carrier in BeanMapUtils module",
        ),
    ] {
        require_absent(
            BEAN_MAP_UTILS_ADAPTER,
            &bean_map_utils_adapter,
            forbidden,
            purpose,
        )?;
    }
    require_contains(
        BEAN_MAP_UTILS_ADAPTER,
        &bean_map_utils_adapter,
        "BeanMap::from_parts(",
        "derive-backed BeanMap construction",
    )?;

    let bean_map_adapter = read_module_family(BEAN_MAP_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct BeanMap", "dedicated Rust BeanMap carrier"),
        ("values: BTreeMap<&'static str, CellValue>", "converted field values"),
        ("field_types: BTreeMap<&'static str, Option<&'static str>>", "declared field types"),
        ("pub(crate) fn from_parts(", "restricted construction boundary"),
    ] {
        require_contains(BEAN_MAP_ADAPTER, &bean_map_adapter, needle, purpose)?;
    }

    let naming_policy_adapter = read_module_family(EASY_EXCEL_NAMING_POLICY_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct EasyExcelNamingPolicy", "dedicated nested Java object owner"),
        ("pub const INSTANCE: Self = Self", "Java singleton contract"),
        ("\"ByEasyExcelCGLIB\"", "Java naming tag"),
    ] {
        require_contains(
            EASY_EXCEL_NAMING_POLICY_ADAPTER,
            &naming_policy_adapter,
            needle,
            purpose,
        )?;
    }

    let fill_executor_adapter = read_module_family(EXCEL_WRITE_FILL_EXECUTOR_ADAPTER)?;
    require_absent(
        EXCEL_WRITE_FILL_EXECUTOR_ADAPTER,
        &fill_executor_adapter,
        "pub struct UniqueDataFlagKey",
        "nested UniqueDataFlagKey object in fill executor module",
    )?;
    let unique_data_flag_key_adapter = read_module_family(UNIQUE_DATA_FLAG_KEY_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct UniqueDataFlagKey", "dedicated fill data-domain key owner"),
        ("sheet_no: Option<i32>", "sheet-number identity"),
        ("sheet_name: Option<String>", "sheet-name identity"),
        ("wrapper_name: Option<String>", "wrapper-name identity"),
        ("#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]", "Java equals/hashCode contract"),
    ] {
        require_contains(
            UNIQUE_DATA_FLAG_KEY_ADAPTER,
            &unique_data_flag_key_adapter,
            needle,
            purpose,
        )?;
    }

    let analysis_cell_adapter = read_module_family(ANALYSIS_CELL_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct AnalysisCell", "real template-analysis cell model"),
        ("impl Default for AnalysisCell", "Java no-argument construction alternative"),
        ("variable_list: Vec::new()", "nullable-list normalization"),
        ("prepare_data_list: Vec::new()", "prepared-data list normalization"),
        ("cell_type: WriteTemplateAnalysisCellType::Common", "usable cell-type invariant"),
        ("self.column_index == other.column_index && self.row_index == other.row_index", "Java coordinate equality"),
    ] {
        require_contains(
            ANALYSIS_CELL_ADAPTER,
            &analysis_cell_adapter,
            needle,
            purpose,
        )?;
    }

    let basic_parameter_adapter = read_module_family(BASIC_PARAMETER_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct BasicParameter", "shared read/write parameter owner"),
        ("pub fn new() -> Self", "Java no-argument construction"),
        ("pub const fn get_use1904windowing", "exact Java digit-bearing getter mapping"),
        ("pub const fn set_use1904windowing", "exact Java digit-bearing setter mapping"),
        ("pub custom_converter_list: Vec<String>", "compile-time converter registration carrier"),
        ("pub filed_cache_location: Option<CacheLocation>", "Java misspelled cache-location contract"),
    ] {
        require_contains(
            BASIC_PARAMETER_ADAPTER,
            &basic_parameter_adapter,
            needle,
            purpose,
        )?;
    }

    let cell_data_adapter = read_module_family(CELL_DATA_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct CellData<T = ()>", "generic Java cell-data owner"),
        ("impl<T: PartialEq> PartialEq for CellData<T>", "explicit Java equality contract"),
        ("impl<T: Hash> Hash for CellData<T>", "explicit Java hashCode alternative"),
        ("self.formula_data == other.formula_data", "formula payload equality"),
        ("self.formula_data.hash(state)", "formula payload hashing"),
    ] {
        require_contains(CELL_DATA_ADAPTER, &cell_data_adapter, needle, purpose)?;
    }
    for (forbidden, purpose) in [
        ("self.row_index == other.row_index", "AbstractCell row coordinate in Java subclass equality"),
        ("self.column_index == other.column_index", "AbstractCell column coordinate in Java subclass equality"),
        ("self.row_index.hash(state)", "AbstractCell row coordinate in Java subclass hash"),
        ("self.column_index.hash(state)", "AbstractCell column coordinate in Java subclass hash"),
    ] {
        require_absent(CELL_DATA_ADAPTER, &cell_data_adapter, forbidden, purpose)?;
    }
    let formula_data_adapter = read_module_family(FORMULA_DATA_ADAPTER)?;
    require_contains(
        FORMULA_DATA_ADAPTER,
        &formula_data_adapter,
        "#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]",
        "CellData formula hash carrier",
    )?;

    let head_adapter = read_module_family(HEAD_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct Head", "real header metadata owner"),
        ("pub field_key: Option<String>", "backend-neutral Java Field identity"),
        ("pub fn from_java_fields(", "Java six-argument constructor alternative"),
        ("pub fn get_field(&self) -> Option<&str> { self.field_key.as_deref() }", "reflection-field getter isolation"),
        ("pub fn set_field(&mut self, value: Option<String>) { self.field_key = value; }", "reflection-field setter isolation"),
    ] {
        require_contains(HEAD_ADAPTER, &head_adapter, needle, purpose)?;
    }

    let converter_key_adapter = read_module_family(CONVERTER_KEY_ADAPTER)?;
    for (needle, purpose) in [
        ("pub struct ConverterKey", "dedicated nested Java key owner"),
        ("rust_type: TypeId", "backend-neutral Java Class identity"),
        ("cell_data_type: Option<CellDataType>", "optional Excel type identity"),
        ("pub const fn get_clazz", "Java Class getter alternative"),
        ("pub const fn set_clazz", "Java Class setter alternative"),
        ("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]", "Java equals/hashCode contract"),
    ] {
        require_contains(
            CONVERTER_KEY_ADAPTER,
            &converter_key_adapter,
            needle,
            purpose,
        )?;
    }
    let converter_key_build_adapter = read_module_family(CONVERTER_KEY_BUILD_ADAPTER)?;
    require_absent(
        CONVERTER_KEY_BUILD_ADAPTER,
        &converter_key_build_adapter,
        "pub struct ConverterKey",
        "nested ConverterKey object in static build module",
    )?;
    for (needle, purpose) in [
        ("pub const fn build_key_for_type(", "Java one-argument buildKey overload"),
        ("pub const fn build_key_for_type_and_cell_data(", "Java two-argument buildKey overload"),
        ("pub fn build_key<T: 'static>(", "existing generic Rust shortcut"),
    ] {
        require_contains(
            CONVERTER_KEY_BUILD_ADAPTER,
            &converter_key_build_adapter,
            needle,
            purpose,
        )?;
    }

    let converter_contract = read_module_family(CONVERTER_CONTRACT_ADAPTER)?;
    for needle in [
        "pub trait Converter<T>",
        "fn support_java_type_key(&self) -> TypeId",
        "fn support_excel_type_key(&self) -> CellDataType",
        "fn convert_to_java_data(&self, context: &ReadConverterContext<'_>)",
        "fn convert_to_excel_data(",
    ] {
        require_contains(
            CONVERTER_CONTRACT_ADAPTER,
            &converter_contract,
            needle,
            "shared Java Converter overload and dispatch contract",
        )?;
    }
    for (path, type_name) in CONCRETE_CONVERTER_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(
            path,
            &source,
            &format!("pub struct {type_name}"),
            "one-file-per-Java-converter public owner",
        )?;
        require_contains(
            path,
            &source,
            "fn convert_to_excel_data(",
            "concrete converter write behavior",
        )?;
        require_absent(
            path,
            &source,
            "todo!()",
            "concrete converter implementation",
        )?;
        require_absent(
            path,
            &source,
            "unimplemented!()",
            "concrete converter implementation",
        )?;
    }
    let read_converter_context = read_module_family(READ_CONVERTER_CONTEXT_ADAPTER)?;
    for needle in [
        "pub const fn set_read_cell_data",
        "pub const fn set_content_property",
        "pub const fn set_analysis_context",
    ] {
        require_contains(
            READ_CONVERTER_CONTEXT_ADAPTER,
            &read_converter_context,
            needle,
            "Java ReadConverterContext mutable lifecycle",
        )?;
    }
    let write_converter_context = read_module_family(WRITE_CONVERTER_CONTEXT_ADAPTER)?;
    for needle in [
        "pub const fn set_value",
        "pub const fn set_content_property",
        "pub const fn set_write_context",
    ] {
        require_contains(
            WRITE_CONVERTER_CONTEXT_ADAPTER,
            &write_converter_context,
            needle,
            "Java WriteConverterContext mutable lifecycle",
        )?;
    }
    let default_converter_loader = read_module_family(DEFAULT_CONVERTER_LOADER_ADAPTER)?;
    for needle in [
        "pub fn load_default_write_converter() -> ConverterRegistry",
        "pub fn load_default_read_converter() -> ConverterRegistry",
        "registry.register_for_write_type",
    ] {
        require_contains(
            DEFAULT_CONVERTER_LOADER_ADAPTER,
            &default_converter_loader,
            needle,
            "single registry owner for concrete converter families",
        )?;
    }

    let field_utils_adapter = read_module_family(FIELD_UTILS_ADAPTER)?;
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

    let read_holder_contract = read_module_family(READ_HOLDER_CONTRACT)?;
    for (needle, purpose) in [
        (
            "pub trait ReadHolder: ConfigurationHolder",
            "Java ReadHolder configuration inheritance",
        ),
        (
            "fn read_listener_list(&self) -> &[String]",
            "Java ReadHolder listener contract",
        ),
        (
            "fn excel_read_head_property(&self) -> &ExcelReadHeadProperty",
            "Java ReadHolder head-property contract",
        ),
    ] {
        require_contains(READ_HOLDER_CONTRACT, &read_holder_contract, needle, purpose)?;
    }
    require_absent(
        READ_HOLDER_CONTRACT,
        &read_holder_contract,
        "analysis_context",
        "AnalysisContext incorrectly exposed as ReadHolder API",
    )?;

    let write_holder_contract = read_module_family(WRITE_HOLDER_CONTRACT)?;
    require_contains(
        WRITE_HOLDER_CONTRACT,
        &write_holder_contract,
        "pub trait WriteHolder: ConfigurationHolder",
        "Java WriteHolder configuration inheritance",
    )?;

    let abstract_read_holder = read_module_family(ABSTRACT_READ_HOLDER)?;
    for needle in [
        "impl ConfigurationHolder for AbstractReadHolder",
        "impl ReadHolder for AbstractReadHolder",
    ] {
        require_contains(
            ABSTRACT_READ_HOLDER,
            &abstract_read_holder,
            needle,
            "facade-owned ReadHolder lifecycle state",
        )?;
    }

    let abstract_write_holder = read_module_family(ABSTRACT_WRITE_HOLDER)?;
    for needle in [
        "impl ConfigurationHolder for AbstractWriteHolder",
        "impl WriteHolder for AbstractWriteHolder",
    ] {
        require_contains(
            ABSTRACT_WRITE_HOLDER,
            &abstract_write_holder,
            needle,
            "facade-owned WriteHolder lifecycle state",
        )?;
    }

    for path in CONCRETE_READ_HOLDER_CONTRACTS {
        let source = read_module_family(path)?;
        require_contains(
            path,
            &source,
            "delegate_read_holder_contract!(",
            "concrete Java read-holder trait contract",
        )?;
    }

    for path in FORMAT_READ_SHEET_HOLDERS {
        let source = read_module_family(path)?;
        require_contains(
            path,
            &source,
            "pub fn from_read_sheet(",
            "format ReadSheetHolder parameterized constructor",
        )?;
    }

    for path in CONCRETE_WRITE_HOLDER_CONTRACTS {
        let source = read_module_family(path)?;
        require_contains(
            path,
            &source,
            "delegate_write_holder_contract!(",
            "concrete Java write-holder trait contract",
        )?;
    }

    let write_basic_parameter = read_module_family(WRITE_BASIC_PARAMETER)?;
    for needle in [
        "pub fn from_options(options: &crate::WriteOptions) -> Self",
        "head: options.dynamic_head.clone()",
        "converters: options.converters.clone()",
    ] {
        require_contains(
            WRITE_BASIC_PARAMETER,
            &write_basic_parameter,
            needle,
            "WriteOptions to Java WriteBasicParameter propagation",
        )?;
    }

    let read_workbook_holder = read_module_family(READ_WORKBOOK_HOLDER)?;
    for needle in [
        "holder.input_stream = value.get_input_stream()",
        "holder.auto_close_stream = value.get_auto_close_stream().unwrap_or(true)",
        "holder.ignore_empty_row = value.get_ignore_empty_row().unwrap_or(true)",
        "get_mandatory_use_input_stream()",
    ] {
        require_contains(
            READ_WORKBOOK_HOLDER,
            &read_workbook_holder,
            needle,
            "Java ReadWorkbookHolder constructor propagation",
        )?;
    }

    let read_sheet_holder = read_module_family(READ_SHEET_HOLDER)?;
    require_contains(
        READ_SHEET_HOLDER,
        &read_sheet_holder,
        "cell_map: IndexMap<usize, CellValue>",
        "Java LinkedHashMap row-cell ordering",
    )?;
    let read_row_holder = read_module_family(READ_ROW_HOLDER)?;
    require_contains(
        READ_ROW_HOLDER,
        &read_row_holder,
        "current_row_analysis_result: Option<CustomReadObject>",
        "Java Object row-analysis result carrier",
    )?;

    let write_workbook_holder = read_module_family(WRITE_WORKBOOK_HOLDER)?;
    for needle in [
        "WriteBasicParameter::from_options(&value.options)",
        "AbstractWriteHolder::from_parameter(&parameter, None)",
        "holder.output_stream = value.output_stream.clone()",
        "holder.temp_template_input_stream = template_input_stream",
        "holder.in_memory = Some(value.in_memory_override.unwrap_or(false))",
        "holder.charset = value.options.charset.name().to_owned()",
        "holder.password = value.options.password.clone()",
    ] {
        require_contains(
            WRITE_WORKBOOK_HOLDER,
            &write_workbook_holder,
            needle,
            "Java WriteWorkbookHolder constructor propagation",
        )?;
    }
    let write_sheet_holder = read_module_family(WRITE_SHEET_HOLDER)?;
    for needle in [
        "Some(parent.abstract_holder())",
        "parent.get_temp_template_input_stream().is_some()",
        "parent_write_workbook_holder_id = Some(std::ptr::from_ref(parent).addr())",
    ] {
        require_contains(
            WRITE_SHEET_HOLDER,
            &write_sheet_holder,
            needle,
            "Java WriteSheetHolder parent propagation",
        )?;
    }
    let write_table_holder = read_module_family(WRITE_TABLE_HOLDER)?;
    for needle in [
        "parent.abstract_holder()",
        "parent_sheet: Option<String>",
        "parent.sheet_name().to_owned()",
        "parent_write_sheet_holder_id = Some(std::ptr::from_ref(parent).addr())",
    ] {
        require_contains(
            WRITE_TABLE_HOLDER,
            &write_table_holder,
            needle,
            "Java WriteTableHolder parent propagation",
        )?;
    }
    for needle in [
        "cell_style_index_map: HashMap<Option<WriteCellStyle>, Vec<WriteCellStyle>>",
        "data_format_map: Vec<DataFormatData>",
        "font_map: Vec<WriteFont>",
        "return origin_cell_style.cloned()",
        "let use_cache = origin_cell_style.is_none()",
        "build_cell_style(origin_cell_style, Some(write_cell_style))",
        "self.create_font(write_cell_style.get_write_font(), origin_font, use_cache)",
        "self.create_data_format(data_format.as_ref(), use_cache)",
        ".entry(origin_cell_style.cloned())",
        "build_data_format(Some(data_format_data))",
        "build_font(origin_font, write_font)?",
    ] {
        require_contains(
            WRITE_WORKBOOK_HOLDER,
            &write_workbook_holder,
            needle,
            "backend-neutral WriteWorkbookHolder style cache semantics",
        )?;
    }

    let compatible_reader_builder = read_module_family(COMPATIBLE_READER_BUILDER)?;
    for needle in [
        "explicit_excel_type: Option<ExcelTypeEnum>",
        "pub const fn excel_type",
        "pub const fn auto_close_stream",
        "pub const fn mandatory_use_input_stream",
        "pub const fn auto_trim",
        "pub const fn use_1904_windowing",
        "ExcelReader::from_temporary_input_with_explicit_type",
        "ExcelReader::new_with_explicit_type",
    ] {
        require_contains(
            COMPATIBLE_READER_BUILDER,
            &compatible_reader_builder,
            needle,
            "Java-compatible reader builder lifecycle and explicit format propagation",
        )?;
    }
    let typed_reader_builder = read_module_family(TYPED_READER_BUILDER)?;
    require_contains(
        TYPED_READER_BUILDER,
        &typed_reader_builder,
        "explicit_excel_type: Option<ExcelTypeEnum>",
        "typed reader explicit format propagation",
    )?;
    let excel_reader = read_module_family(EXCEL_READER_ADAPTER)?;
    require_contains(
        EXCEL_READER_ADAPTER,
        &excel_reader,
        "ExcelAnalyserImpl::from_path_with_type",
        "reader to analyser explicit format propagation",
    )?;
    let writer_sheet_builder = read_module_family(WRITER_SHEET_BUILDER)?;
    for needle in [
        "pub fn do_fill(mut self, data: &dyn Any)",
        "pub fn do_fill_with_config(",
        "pub fn do_fill_with<F>",
        "pub fn do_fill_with_config_supplier<F>",
        "crate::excel_builder::do_fill_template_with_config",
    ] {
        require_contains(
            WRITER_SHEET_BUILDER,
            &writer_sheet_builder,
            needle,
            "Java ExcelWriterSheetBuilder fill overload family",
        )?;
    }
    for forbidden in ["HashMap<String, u32>", "HashMap<String, u16>"] {
        require_absent(
            WRITE_WORKBOOK_HOLDER,
            &write_workbook_holder,
            forbidden,
            "string-key style index placeholder",
        )?;
    }

    let style_property = read_module_family(STYLE_PROPERTY_ADAPTER)?;
    for (needle, purpose) in [
        (
            "pub const fn new() -> Self",
            "Java StyleProperty no-argument construction",
        ),
        (
            "pub fn from_cell_style(cell_style: ExcelCellStyle)",
            "annotation style promotion",
        ),
        (
            "write_cell_style: WriteCellStyle",
            "owned runtime style carrier",
        ),
        ("Option<&WriteFont>", "owned runtime font exposure"),
    ] {
        require_contains(STYLE_PROPERTY_ADAPTER, &style_property, needle, purpose)?;
    }

    let write_cell_style = read_module_family(WRITE_CELL_STYLE_ADAPTER)?;
    for needle in [
        "pub fn build(",
        "style_property: Option<&crate::StyleProperty>",
        "font_property: Option<&crate::FontProperty>",
        "if style_property.is_none() && font_property.is_none()",
        "pub fn merge(source: &Self, target: &mut Self)",
    ] {
        require_contains(
            WRITE_CELL_STYLE_ADAPTER,
            &write_cell_style,
            needle,
            "Java WriteCellStyle build/merge semantics",
        )?;
    }
    require_absent(
        WRITE_CELL_STYLE_ADAPTER,
        &write_cell_style,
        "pub const fn build(self) -> Self",
        "non-Java WriteCellStyle build shape",
    )?;
    for (needle, purpose) in [
        ("pub font_name: Option<String>", "runtime font-name ownership"),
        ("pub fn to_write_font(&self) -> WriteFont", "lossless runtime font conversion"),
    ] {
        let font_property = read_module_family(FONT_PROPERTY_ADAPTER)?;
        require_contains(FONT_PROPERTY_ADAPTER, &font_property, needle, purpose)?;
    }
    for (path, needles) in METADATA_PROPERTY_ADAPTERS {
        let source = read_module_family(path)?;
        for needle in *needles {
            require_contains(
                path,
                &source,
                needle,
                "Java metadata property owner and mutable accessor contract",
            )?;
        }
        for forbidden in ["todo!()", "unimplemented!()"] {
            require_absent(path, &source, forbidden, "metadata property implementation")?;
        }
    }
    for (path, needles) in WRITE_STYLE_VALUE_ADAPTERS {
        let source = read_module_family(path)?;
        for needle in *needles {
            require_contains(path, &source, needle, "Java runtime write-style value contract")?;
        }
    }
    for path in WRITE_STYLE_ANNOTATION_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "pub struct", "typed Java write-style annotation carrier")?;
        require_contains(path, &source, "pub const fn", "write-style annotation value accessor")?;
        require_absent(path, &source, "todo!()", "write-style annotation implementation")?;
        require_absent(path, &source, "unimplemented!()", "write-style annotation implementation")?;
    }
    for path in WRITE_STYLE_STRATEGY_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "对应 Java", "Java write-style strategy ownership")?;
        require_absent(path, &source, "todo!()", "write-style strategy implementation")?;
        require_absent(path, &source, "unimplemented!()", "write-style strategy implementation")?;
    }
    let horizontal_style_contract = read_module_family(HORIZONTAL_STYLE_STRATEGY_ADAPTER)?;
    for needle in [
        "impl Default for HorizontalCellStyleStrategy",
        "#[derive(Debug, Clone, PartialEq, Eq, Hash)]",
        "crate::constant::order_constant::DEFINE_STYLE",
    ] {
        require_contains(
            HORIZONTAL_STYLE_STRATEGY_ADAPTER,
            &horizontal_style_contract,
            needle,
            "HorizontalCellStyleStrategy Java construction/value/order semantics",
        )?;
    }
    let default_style = read_module_family("crates/easyexcel/src/write/style/default_style.rs")?;
    for needle in [
        "fill_foreground_color: Some(ExcelColor::Indexed(22))",
        ".font_name(\"宋体\")",
        "crate::constant::order_constant::DEFAULT_DEFINE_STYLE",
        "fn style_cell_style(&self, context: &WriteCellContext)",
        "fn style_write_font(&self, context: &WriteCellContext)",
    ] {
        require_contains(
            "crates/easyexcel/src/write/style/default_style.rs",
            &default_style,
            needle,
            "Java DefaultStyle complete header behavior",
        )?;
    }
    let excel_cell_style = read_module_family(EXCEL_CELL_STYLE_ADAPTER)?;
    require_contains(
        EXCEL_CELL_STYLE_ADAPTER,
        &excel_cell_style,
        "pub struct ExcelCellStyle",
        "independent engine style carrier",
    )?;
    require_absent(
        EXCEL_CELL_STYLE_ADAPTER,
        &excel_cell_style,
        "pub use crate::write::metadata::style::write_cell_style::WriteCellStyle as ExcelCellStyle",
        "runtime/engine style alias",
    )?;
    let horizontal_style = read_module_family(HORIZONTAL_STYLE_STRATEGY_ADAPTER)?;
    for needle in [
        "head_style: WriteCellStyle",
        "content_styles: Vec<WriteCellStyle>",
        "self.head_style.font = Some(font.clone())",
        "fn style_write_font(&self, context: &WriteCellContext) -> Option<WriteFont>",
    ] {
        require_contains(
            HORIZONTAL_STYLE_STRATEGY_ADAPTER,
            &horizontal_style,
            needle,
            "runtime style strategy ownership",
        )?;
    }
    let write_handler = read_module_family(WRITE_HANDLER_ADAPTER)?;
    require_contains(
        WRITE_HANDLER_ADAPTER,
        &write_handler,
        "fn style_write_font(&self, _context: &WriteCellContext) -> Option<WriteFont>",
        "runtime handler font channel",
    )?;
    for needle in [
        "fn before_workbook_create",
        "fn after_workbook_create",
        "fn after_workbook_dispose",
        "fn before_sheet_create",
        "fn after_sheet_create",
        "fn before_row_create",
        "fn after_row_create",
        "fn after_row_dispose",
        "fn before_cell_create",
        "fn after_cell_create",
        "fn after_cell_data_converted",
        "fn after_cell_dispose",
    ] {
        require_contains(
            WRITE_HANDLER_ADAPTER,
            &write_handler,
            needle,
            "unified object-safe Java write handler lifecycle",
        )?;
    }
    for path in WRITE_HANDLER_MARKERS {
        let source = read_module_family(path)?;
        require_contains(
            path,
            &source,
            ": crate::core::WriteHandler",
            "Java handler-interface marker over executable lifecycle owner",
        )?;
    }
    for path in ABSTRACT_WRITE_HANDLERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "pub struct Abstract", "deprecated Java abstract handler owner")?;
        require_contains(path, &source, "impl WriteHandler for", "abstract handler default no-op implementation")?;
    }
    for path in WRITE_HANDLER_CHAINS {
        let source = read_module_family(path)?;
        for needle in ["pub fn get_handler", "pub fn set_handler", "pub const fn get_next", "pub fn set_next", "pub fn add_last"] {
            require_contains(path, &source, needle, "Java linked handler-chain lifecycle")?;
        }
        require_absent(path, &source, "todo!()", "handler-chain implementation")?;
        require_absent(path, &source, "unimplemented!()", "handler-chain implementation")?;
    }
    for (path, needles) in [
        (
            WRITE_WORKBOOK_CONTEXT_ADAPTER,
            &["pub fn get_write_workbook_holder", "pub const fn get_write_context", "pub fn set_write_context", "pub fn set_write_workbook_holder"][..],
        ),
        (
            WRITE_SHEET_CONTEXT_ADAPTER,
            &["pub const fn get_write_workbook_holder", "pub fn get_write_sheet_holder", "pub fn set_write_context", "pub fn set_write_sheet_holder"][..],
        ),
        (
            WRITE_ROW_CONTEXT_ADAPTER,
            &["pub const fn get_row", "pub fn set_row", "pub fn set_row_index", "pub fn set_write_table_holder"][..],
        ),
        (
            WRITE_CELL_CONTEXT_ADAPTER,
            &["pub const fn get_row", "pub fn set_row", "pub fn set_cell", "pub fn set_first_cell_data", "pub const fn set_target_cell_data_type"][..],
        ),
    ] {
        let source = read_module_family(path)?;
        for needle in needles {
            require_contains(path, &source, needle, "Java write-handler context mutable lifecycle")?;
        }
    }
    let default_write_handler_loader = read_module_family(DEFAULT_WRITE_HANDLER_LOADER_ADAPTER)?;
    for needle in [
        "pub fn default_write_handler_list()",
        "pub fn load_default_handler_for(",
        "DimensionWorkbookWriteHandler::new()",
        "DefaultRowWriteHandler::new()",
        "FillStyleCellWriteHandler::new()",
    ] {
        require_contains(
            DEFAULT_WRITE_HANDLER_LOADER_ADAPTER,
            &default_write_handler_loader,
            needle,
            "Java default write handler loading order",
        )?;
    }
    let cell_format_context = read_module_family(CELL_FORMAT_CONTEXT_ADAPTER)?;
    require_contains(
        CELL_FORMAT_CONTEXT_ADAPTER,
        &cell_format_context,
        "pub(crate) handler_font: Option<crate::WriteFont>",
        "runtime handler font propagation",
    )?;
    let xlsx_cell_emission = read_module_family(XLSX_CELL_EMISSION_ADAPTER)?;
    require_contains(
        XLSX_CELL_EMISSION_ADAPTER,
        &xlsx_cell_emission,
        "name: style.get_font_name().map(str::to_owned)",
        "XLSX dynamic font-name application",
    )?;
    let xls_cell_emission = read_module_family(XLS_CELL_EMISSION_ADAPTER)?;
    require_contains(
        XLS_CELL_EMISSION_ADAPTER,
        &xls_cell_emission,
        "apply_write_font(&mut request, &font)",
        "XLS dynamic font-name application",
    )?;

    let write_font = read_module_family(WRITE_FONT_ADAPTER)?;
    require_contains(
        WRITE_FONT_ADAPTER,
        &write_font,
        "pub fn merge(source: &Self, target: &mut Self)",
        "Java WriteFont merge side effect",
    )?;
    require_absent(
        STYLE_PROPERTY_ADAPTER,
        &style_property,
        "pub const fn build(self)",
        "internal conversion misidentified as Java overloaded annotation build",
    )?;

    let content_style = read_module_family(CONTENT_STYLE_ADAPTER)?;
    require_contains(
        CONTENT_STYLE_ADAPTER,
        &content_style,
        "StyleProperty::from_write_cell_style(self.to_write_cell_style())",
        "annotation-owned StyleProperty conversion",
    )?;

    for path in JAVA_ENUM_CONTRACT_ADAPTERS {
        let source = read_module_family(path)?;
        for needle in ["pub const ALL:", "pub const fn java_name", "impl std::str::FromStr"] {
            require_contains(
                path,
                &source,
                needle,
                "Java enum values/valueOf idiomatic contract",
            )?;
        }
    }
    for (path, needles) in JAVA_CONSTANT_CONTRACT_ADAPTERS {
        let source = read_module_family(path)?;
        for needle in *needles {
            require_contains(path, &source, needle, "Java constant owner observable contract")?;
        }
    }
    for path in JAVA_EXCEPTION_CONTRACT_ADAPTERS {
        let source = read_module_family(path)?;
        require_contains(path, &source, "impl std::error::Error for", "Rust error-chain carrier")?;
        require_contains(path, &source, "impl From<", "unified ExcelError conversion")?;
        require_absent(path, &source, "todo!()", "exception implementation")?;
        require_absent(path, &source, "unimplemented!()", "exception implementation")?;
    }
    let data_convert_exception = read_module_family(
        "crates/easyexcel/src/exception/excel_data_convert_exception.rs",
    )?;
    for needle in [
        "impl PartialEq for ExcelDataConvertException",
        "impl Hash for ExcelDataConvertException",
        "self.row_index == other.row_index",
    ] {
        require_contains(
            "crates/easyexcel/src/exception/excel_data_convert_exception.rs",
            &data_convert_exception,
            needle,
            "ExcelDataConvertException callSuper=false value semantics",
        )?;
    }
    let write_data_convert_exception = read_module_family(
        "crates/easyexcel/src/exception/excel_write_data_convert_exception.rs",
    )?;
    for needle in [
        "impl PartialEq for ExcelWriteDataConvertException",
        "impl Hash for ExcelWriteDataConvertException",
        "self.cell_write_handler_context == other.cell_write_handler_context",
    ] {
        require_contains(
            "crates/easyexcel/src/exception/excel_write_data_convert_exception.rs",
            &write_data_convert_exception,
            needle,
            "ExcelWriteDataConvertException callSuper=false value semantics",
        )?;
    }
    let excel_type_enum = read_module_family(EXCEL_TYPE_ADAPTER)?;
    for needle in [
        "pub fn value_of(read_workbook: &crate::read::metadata::ReadWorkbook)",
        "easyexcel_io::Format::detect_path(file)",
        "if let Some(input) = read_workbook.get_input_stream()",
    ] {
        require_contains(
            EXCEL_TYPE_ADAPTER,
            &excel_type_enum,
            needle,
            "Java ExcelTypeEnum.valueOf(ReadWorkbook) adapter",
        )?;
    }

    let rust_public_api_generator = read(RUST_PUBLIC_API_GENERATOR)?;
    for needle in [
        "tokens[:2] == [\"pub\", \"variant\"]",
        "tokens[:2] == [\"pub\", \"field\"]",
        "cargo-public-api 当前对 enum variant 输出为",
        "[A-Z][A-Za-z0-9_]*(?:\\(|\\s*\\{|$)",
    ] {
        require_contains(
            RUST_PUBLIC_API_GENERATOR,
            &rust_public_api_generator,
            needle,
            "Rust enum/field public API extraction",
        )?;
    }

    let public_api_mapping_suggester = read(PUBLIC_API_MAPPING_SUGGESTER)?;
    for (needle, purpose) in [
        (
            "CSV_STATEFUL_MEMBERS = {",
            "per-member CSV semantic ownership classification",
        ),
        (
            "def is_csv_poi_compatibility_member",
            "CSV POI compatibility boundary classifier",
        ),
        (
            "def rust_member_owner",
            "single runtime-owner resolver for transparent public aliases",
        ),
        (
            ".rsplit(\"$\", 1)[-1]",
            "Java nested owner resolves to the real inner Rust type",
        ),
        (
            "NOMINAL_STATIC_UTILITY_OWNERS = {",
            "nominal static API types take precedence over module alternatives",
        ),
        (
            "def holder_constructor_names",
            "descriptor-aware Holder constructor mapping",
        ),
        (
            "def read_cell_data_names",
            "descriptor-aware ReadCellData overload mapping",
        ),
        (
            "def analysis_context_names",
            "descriptor-aware AnalysisContext lifecycle mapping",
        ),
        (
            "def pascal_case",
            "Java enum constant to Rust variant normalization",
        ),
        (
            "BACKEND_NEUTRAL_ENUM_MEMBERS = {",
            "POI enum getter backend-neutral alternative classification",
        ),
        (
            "if java_name in {\"values\", \"valueOf\"}",
            "Java enum generated API idiomatic mapping",
        ),
        (
            "if is_csv_poi_compatibility_member(java):",
            "owner-level alternative mapping for POI-only CSV methods",
        ),
        (
            "instead of copying a same-name empty method",
            "auditable no-empty-shell mapping rationale",
        ),
    ] {
        require_contains(
            PUBLIC_API_MAPPING_SUGGESTER,
            &public_api_mapping_suggester,
            needle,
            purpose,
        )?;
    }
    let read_cell_data = read_module_family(READ_CELL_DATA_ADAPTER)?;
    for needle in [
        "pub fn empty() -> Self",
        "pub fn from_type(cell_type: CellDataType) -> Self",
        "pub fn from_type_and_string(",
        "pub fn from_boolean(value: bool) -> Self",
        "pub fn from_string(value: impl Into<String>) -> Self",
        "pub fn from_number(value: BigDecimal) -> Self",
        "pub fn new_instance(",
        "pub fn new_instance_original(",
        "pub fn clone_data(&self) -> Self",
    ] {
        require_contains(
            READ_CELL_DATA_ADAPTER,
            &read_cell_data,
            needle,
            "Java ReadCellData constructor/factory carrier",
        )?;
    }
    for (path, excluded_field, purpose) in [
        (
            CLIENT_ANCHOR_DATA_ADAPTER,
            "self.coordinates == other.coordinates",
            "ClientAnchorData Lombok callSuper=false equality",
        ),
        (
            HYPERLINK_DATA_ADAPTER,
            "self.coordinates == other.coordinates",
            "HyperlinkData Lombok callSuper=false equality",
        ),
        (
            IMAGE_DATA_ADAPTER,
            "self.anchor == other.anchor",
            "ImageData Lombok callSuper=false equality",
        ),
        (
            COMMENT_DATA_ADAPTER,
            "self.anchor == other.anchor",
            "CommentData Lombok callSuper=false equality",
        ),
        (
            COMMENT_DATA_ADAPTER,
            "self.visible == other.visible",
            "CommentData Java equality excludes Rust visibility extension",
        ),
    ] {
        let source = read_module_family(path)?;
        require_contains(path, &source, "impl PartialEq for", purpose)?;
        require_contains(path, &source, "impl Hash for", purpose)?;
        require_absent(path, &source, excluded_field, purpose)?;
    }
    let rich_text_string_data = read_module_family(RICH_TEXT_STRING_DATA_ADAPTER)?;
    require_contains(
        RICH_TEXT_STRING_DATA_ADAPTER,
        &rich_text_string_data,
        "PartialEq, Eq, Hash",
        "CommentData Java hash carrier for rich text body",
    )?;
    for (path, needles) in [
        (
            CSV_WORKBOOK_ENGINE,
            &["pub const fn identity(&self) -> usize", "sheet.set_csv_workbook(Some(self.identity))"][..],
        ),
        (
            CSV_SHEET_ENGINE,
            &["row.set_csv_workbook(self.csv_workbook_id)", "row.set_csv_sheet(Some(self.identity))"][..],
        ),
        (
            CSV_ROW_ENGINE,
            &["pub const fn get_csv_sheet(&self) -> Option<usize>", "cell.set_csv_workbook(self.csv_workbook_id)", "cell.set_csv_sheet(self.csv_sheet_id)"][..],
        ),
        (
            CSV_CELL_ENGINE,
            &["pub const fn get_csv_sheet(&self) -> Option<usize>", "csv_sheet_id: Option<usize>"][..],
        ),
        (
            CSV_CELL_STYLE_ENGINE,
            &["pub const fn get_font_index_as_int(&self) -> usize", "pub const fn set_font(&mut self, _font: Option<()>)"][..],
        ),
        (
            CSV_RICH_TEXT_ENGINE,
            &["pub const fn num_formatting_runs(&self) -> usize", "pub const fn clear_formatting(&mut self)"][..],
        ),
    ] {
        let source = read_module_family(path)?;
        for needle in needles {
            require_contains(
                path,
                &source,
                needle,
                "CSV model family owner and Java lifecycle/POI alternative carrier",
            )?;
        }
    }
    let data_format_data = read_module_family(DATA_FORMAT_DATA_ENGINE)?;
    for needle in [
        "#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]",
        "pub fn merge(source: Option<&Self>, target: Option<&mut Self>)",
        "pub fn clone_data(&self) -> Self",
    ] {
        require_contains(
            DATA_FORMAT_DATA_ENGINE,
            &data_format_data,
            needle,
            "shared DataFormatData Java value semantics",
        )?;
    }
    let excel_content_property = read_module_family(EXCEL_CONTENT_PROPERTY_ADAPTER)?;
    for needle in [
        "pub const EMPTY: Self",
        "pub field_name: Option<String>",
        "pub converter_key: Option<String>",
        "pub fn get_field(&self) -> Option<&str>",
        "pub fn get_converter(&self) -> Option<&str>",
    ] {
        require_contains(
            EXCEL_CONTENT_PROPERTY_ADAPTER,
            &excel_content_property,
            needle,
            "ExcelContentProperty compile-time metadata alternative",
        )?;
    }
    let analysis_context_impl = read_module_family(ANALYSIS_CONTEXT_IMPL_ADAPTER)?;
    for needle in [
        "pub fn current_sheet(&mut self, read_sheet: &ReadSheet)",
        "pub const fn read_workbook_holder(&self) -> &ReadWorkbookHolder",
        "pub const fn read_sheet_holder(&self) -> Option<&ReadSheetHolder>",
        "pub const fn read_row_holder(&self) -> Option<&ReadRowHolder>",
        "pub fn set_read_row_holder(&mut self, read_row_holder: ReadRowHolder)",
        "pub fn set_read_sheet_list(&mut self, read_sheet_list: Vec<ReadSheet>)",
        "pub fn get_input_stream(&self) -> Option<&[u8]>",
        "pub fn interrupt(&self) -> Result<()>",
    ] {
        require_contains(
            ANALYSIS_CONTEXT_IMPL_ADAPTER,
            &analysis_context_impl,
            needle,
            "split AnalysisContext mutable lifecycle carrier",
        )?;
    }
    for obsolete in [
        "easyexcel/src",
        "crates/easyexcel/src/read/read.rs",
        "crates/easyexcel/src/read/read/metadata.rs",
        "crates/easyexcel/src/cache/ehcache.rs",
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
