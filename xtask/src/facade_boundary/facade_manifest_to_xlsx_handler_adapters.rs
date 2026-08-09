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
const MODEL_CHART_MUTATION_ENGINE: &str =
    "crates/easyexcel-model/src/model/chart_mutation.rs";
const FACADE_CHART_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/context/chart_mutation.rs",
    "crates/easyexcel/src/context/chart_range.rs",
    "crates/easyexcel/src/context/chart_series.rs",
    "crates/easyexcel/src/context/chart_type.rs",
];
const XLSX_GENERATED_CHART_ENGINE: &str =
    "crates/easyexcel-xlsx/src/xlsx/generation/generated_chart.rs";
const XLS_GENERATED_CHART_ENGINE: &str =
    "crates/easyexcel-xls/src/biff8/workbook/biff8cell_to_write_bof/biff8book.rs";
const XLS_GENERATED_CELL_ENGINE: &str =
    "crates/easyexcel-xls/src/biff8/workbook/biff8cell_to_write_bof/generated_biff8_cell_value.rs";
const XLS_TEMPLATE_ADAPTER: &str = "crates/easyexcel/src/write/xls_adapter/template.rs";
const XLSX_GENERATION_ENGINE: &str = "crates/easyexcel-xlsx/src/xlsx/generation.rs";
const FILL_CONFIG_OWNER: &str =
    "crates/easyexcel/src/write/metadata/fill/fill_config.rs";
const BUILDER_FILL_CONFIG_ADAPTER: &str =
    "crates/easyexcel/src/write/excel_builder/fill_config.rs";
const ORPHAN_WRITER_SHEET_BUILDER: &str =
    "crates/easyexcel/src/write/excel_writer_sheet_builder.rs";
const WEB_HEADER_ENGINE: &str = "crates/easyexcel-web/src/web/http_headers.rs";
const WEB_HEADER_ADAPTERS: &[&str] = &[
    "crates/easyexcel-actix/src/headers.rs",
    "crates/easyexcel-axum/src/headers.rs",
    "crates/easyexcel-hyper/src/headers.rs",
    "crates/easyexcel-poem/src/headers.rs",
    "crates/easyexcel-rocket/src/headers.rs",
    "crates/easyexcel-salvo/src/headers.rs",
    "crates/easyexcel-warp/src/headers.rs",
];
const WEB_RESPONSE_ADAPTERS: &[&str] = &[
    "crates/easyexcel-actix/src/excel_response.rs",
    "crates/easyexcel-axum/src/excel_response.rs",
    "crates/easyexcel-hyper/src/excel_response.rs",
    "crates/easyexcel-poem/src/excel_response.rs",
    "crates/easyexcel-rocket/src/excel_response.rs",
    "crates/easyexcel-salvo/src/excel_response.rs",
    "crates/easyexcel-warp/src/excel_response.rs",
];
const CSV_ENCODING_ADAPTER: &str = "crates/easyexcel/src/write/csv_encoding_writer.rs";
const CSV_MODEL_BOUNDARIES: &[(&str, &str, &str, &str)] = &[
    (
        "crates/easyexcel-csv/src/csv/csv_workbook.rs",
        "crates/easyexcel/src/metadata/csv/csv_workbook.rs",
        "CsvWorkbook",
        "pub type CsvWorkbook = easyexcel_csv::CsvWorkbook<CellValue>;",
    ),
    (
        "crates/easyexcel-csv/src/csv/csv_sheet.rs",
        "crates/easyexcel/src/metadata/csv/csv_sheet.rs",
        "CsvSheet",
        "pub type CsvSheet = easyexcel_csv::CsvSheet<CellValue>;",
    ),
    (
        "crates/easyexcel-csv/src/csv/csv_cell.rs",
        "crates/easyexcel/src/metadata/csv/csv_cell.rs",
        "CsvCell",
        "pub type CsvCell = easyexcel_csv::CsvCell<CellValue>;",
    ),
    (
        "crates/easyexcel-csv/src/csv/csv_row.rs",
        "crates/easyexcel/src/metadata/csv/csv_row.rs",
        "CsvRow",
        "pub type CsvRow = easyexcel_csv::CsvRow<CellValue>;",
    ),
    (
        "crates/easyexcel-csv/src/csv/csv_cell_style.rs",
        "crates/easyexcel/src/metadata/csv/csv_cell_style.rs",
        "CsvCellStyle",
        "pub use easyexcel_csv::CsvCellStyle;",
    ),
];
const EXCEL_TYPE_ADAPTER: &str = "crates/easyexcel/src/support/excel_type_enum.rs";
const XLSX_FACADE: &str = "crates/easyexcel/src/xlsx.rs";
const XLS_RECORD_DISPATCHER: &str = "crates/easyexcel/src/analysis/v03/xls_record_dispatcher.rs";
const XLS_SAX_ADAPTER: &str = "crates/easyexcel/src/analysis/v03/xls_sax_analyser.rs";
const XLSX_SAX_ADAPTER: &str = "crates/easyexcel/src/analysis/v07/xlsx_sax_analyser.rs";
const XLS_OBJ_HANDLER: &str = "crates/easyexcel/src/analysis/v03/handlers/obj_record_handler.rs";
const STYLE_UTIL_ADAPTER: &str = "crates/easyexcel/src/util/style_util.rs";
const DATA_FORMATTER_ENGINE: &str = "crates/easyexcel-format/src/format/data_formatter.rs";
const DATA_FORMATTER_ADAPTER: &str =
    "crates/easyexcel/src/metadata/format/data_formatter.rs";
const GENERAL_NUMBER_FORMAT_ENGINE: &str =
    "crates/easyexcel-format/src/format/excel_general_number_format.rs";
const GENERAL_NUMBER_FORMAT_ADAPTER: &str =
    "crates/easyexcel/src/metadata/format/excel_general_number_format.rs";
const NUMBER_DATA_FORMATTER_ENGINE: &str =
    "crates/easyexcel-format/src/format/number_data_formatter_utils.rs";
const NUMBER_DATA_FORMATTER_ADAPTER: &str =
    "crates/easyexcel/src/util/number_data_formatter_utils.rs";
const POSITION_UTILS_ENGINE: &str = "crates/easyexcel-utils/src/utils/position_utils.rs";
const POSITION_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/position_utils.rs";
const OOXML_CONSTANTS_ENGINE: &str =
    "crates/easyexcel-xlsx/src/xlsx/ooxml_constants.rs";
const OOXML_CONSTANTS_ADAPTER: &str =
    "crates/easyexcel/src/constant/excel_xml_constants.rs";
const DATE_ENGINE: &str = "crates/easyexcel-model/src/model/dates.rs";
const DATE_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/date_utils.rs";
const BUILTIN_FORMATS_ENGINE: &str =
    "crates/easyexcel-format/src/format/builtin_formats.rs";
const BUILTIN_FORMATS_ADAPTER: &str =
    "crates/easyexcel/src/constant/builtin_formats.rs";
const NUMBER_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/number_utils.rs";
const FACADE_ERROR: &str = "crates/easyexcel/src/support/excel_error.rs";
const XLSX_TEMPLATE_ADAPTER: &str = "crates/easyexcel/src/template/template_writer.rs";
const XLSX_TEMPLATE_SELECTION_ENGINE: &str = "crates/easyexcel-xlsx/src/xlsx/template_source.rs";
const XLSX_EVENT_READER_ENGINE: &str = "crates/easyexcel-xlsx/src/xlsx/event_reader.rs";
const ROW_PROCESSING_ADAPTER: &str = "crates/easyexcel/src/read/row_processing.rs";
const TEMPLATE_WRITE_ADAPTER: &str = "crates/easyexcel/src/write/template_write.rs";
const TEMPLATE_FILL_ADAPTER: &str = "crates/easyexcel/src/template/fill_engine.rs";
const TEMPLATE_WRITER_ADAPTER: &str = "crates/easyexcel/src/template/template_writer.rs";
const BUILDER_FILL_EXECUTOR: &str =
    "crates/easyexcel/src/template/builder_fill_executor.rs";
const EXCEL_ROW_CONTRACT: &str = "crates/easyexcel/src/metadata/excel_row.rs";
const EXCEL_ROW_DERIVE: &str =
    "crates/easyexcel-derive/src/expand/excel_row/trait_impl.rs";
const STATEFUL_BACKEND_POLICY: &str =
    "crates/easyexcel/src/write/builder/excel_writer_builder.rs";
const JAVA_STATEFUL_BACKEND_POLICY: &str =
    "crates/easyexcel/src/write/excel_writer_builder.rs";
const STATEFUL_WRITER: &str = "crates/easyexcel/src/excel_writer/new_to_output_path.rs";
const XLSX_TEMPLATE_FILL_ENGINE: &str = "crates/easyexcel-xlsx/src/xlsx/template_fill.rs";
const XLSX_TEMPLATE_RICH_TEXT_ENGINE: &str =
    "crates/easyexcel-xlsx/src/xlsx/template_xml/template_rich_text.rs";
const READ_HELPERS_ADAPTER: &str = "crates/easyexcel/src/read/read_helpers.rs";
const EXCEL_WRITER_CORE: &str = "crates/easyexcel/src/write/excel_writer_core.rs";
const STRING_UTILS_ENGINE: &str = "crates/easyexcel-utils/src/utils/string_utils.rs";
const CLASS_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/class_utils.rs";
const CONTENT_PROPERTY_KEY_ADAPTER: &str =
    "crates/easyexcel/src/util/content_property_key.rs";
const FIELD_CACHE_KEY_ADAPTER: &str = "crates/easyexcel/src/util/field_cache_key.rs";
const BEAN_MAP_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/bean_map_utils.rs";
const BEAN_MAP_ADAPTER: &str = "crates/easyexcel/src/util/bean_map.rs";
const EASY_EXCEL_NAMING_POLICY_ADAPTER: &str =
    "crates/easyexcel/src/util/easy_excel_naming_policy.rs";
const EXCEL_WRITE_FILL_EXECUTOR_ADAPTER: &str =
    "crates/easyexcel/src/write/executor/excel_write_fill_executor.rs";
const UNIQUE_DATA_FLAG_KEY_ADAPTER: &str =
    "crates/easyexcel/src/write/executor/unique_data_flag_key.rs";
const ANALYSIS_CELL_ADAPTER: &str =
    "crates/easyexcel/src/write/metadata/fill/analysis_cell.rs";
const BASIC_PARAMETER_ADAPTER: &str = "crates/easyexcel/src/metadata/basic_parameter.rs";
const CELL_DATA_ADAPTER: &str = "crates/easyexcel/src/metadata/data/cell_data.rs";
const FORMULA_DATA_ADAPTER: &str = "crates/easyexcel/src/metadata/data/formula_data.rs";
const HEAD_ADAPTER: &str = "crates/easyexcel/src/metadata/head.rs";
const CONVERTER_KEY_ADAPTER: &str = "crates/easyexcel/src/converters/converter_key.rs";
const CONVERTER_KEY_BUILD_ADAPTER: &str =
    "crates/easyexcel/src/converters/converter_key_build.rs";
const CONVERTER_CONTRACT_ADAPTER: &str = "crates/easyexcel/src/converters/converter.rs";
const READ_CONVERTER_CONTEXT_ADAPTER: &str =
    "crates/easyexcel/src/converters/read_converter_context.rs";
const WRITE_CONVERTER_CONTEXT_ADAPTER: &str =
    "crates/easyexcel/src/converters/write_converter_context.rs";
const DEFAULT_CONVERTER_LOADER_ADAPTER: &str =
    "crates/easyexcel/src/converters/default_converter_loader.rs";
const CONCRETE_CONVERTER_ADAPTERS: &[(&str, &str)] = &[
    ("crates/easyexcel/src/converters/bigdecimal/big_decimal_boolean_converter.rs", "BigDecimalBooleanConverter"),
    ("crates/easyexcel/src/converters/bigdecimal/big_decimal_number_converter.rs", "BigDecimalNumberConverter"),
    ("crates/easyexcel/src/converters/bigdecimal/big_decimal_string_converter.rs", "BigDecimalStringConverter"),
    ("crates/easyexcel/src/converters/biginteger/big_integer_boolean_converter.rs", "BigIntegerBooleanConverter"),
    ("crates/easyexcel/src/converters/biginteger/big_integer_number_converter.rs", "BigIntegerNumberConverter"),
    ("crates/easyexcel/src/converters/biginteger/big_integer_string_converter.rs", "BigIntegerStringConverter"),
    ("crates/easyexcel/src/converters/booleanconverter/boolean_boolean_converter.rs", "BooleanBooleanConverter"),
    ("crates/easyexcel/src/converters/booleanconverter/boolean_number_converter.rs", "BooleanNumberConverter"),
    ("crates/easyexcel/src/converters/booleanconverter/boolean_string_converter.rs", "BooleanStringConverter"),
    ("crates/easyexcel/src/converters/bytearray/boxing_byte_array_image_converter.rs", "BoxingByteArrayImageConverter"),
    ("crates/easyexcel/src/converters/bytearray/byte_array_image_converter.rs", "ByteArrayImageConverter"),
    ("crates/easyexcel/src/converters/byteconverter/byte_boolean_converter.rs", "ByteBooleanConverter"),
    ("crates/easyexcel/src/converters/byteconverter/byte_number_converter.rs", "ByteNumberConverter"),
    ("crates/easyexcel/src/converters/byteconverter/byte_string_converter.rs", "ByteStringConverter"),
    ("crates/easyexcel/src/converters/date/date_date_converter.rs", "DateDateConverter"),
    ("crates/easyexcel/src/converters/date/date_number_converter.rs", "DateNumberConverter"),
    ("crates/easyexcel/src/converters/date/date_string_converter.rs", "DateStringConverter"),
    ("crates/easyexcel/src/converters/doubleconverter/double_boolean_converter.rs", "DoubleBooleanConverter"),
    ("crates/easyexcel/src/converters/doubleconverter/double_number_converter.rs", "DoubleNumberConverter"),
    ("crates/easyexcel/src/converters/doubleconverter/double_string_converter.rs", "DoubleStringConverter"),
    ("crates/easyexcel/src/converters/file/file_image_converter.rs", "FileImageConverter"),
    ("crates/easyexcel/src/converters/floatconverter/float_boolean_converter.rs", "FloatBooleanConverter"),
    ("crates/easyexcel/src/converters/floatconverter/float_number_converter.rs", "FloatNumberConverter"),
    ("crates/easyexcel/src/converters/floatconverter/float_string_converter.rs", "FloatStringConverter"),
    ("crates/easyexcel/src/converters/inputstream/input_stream_image_converter.rs", "InputStreamImageConverter"),
    ("crates/easyexcel/src/converters/integer/integer_boolean_converter.rs", "IntegerBooleanConverter"),
    ("crates/easyexcel/src/converters/integer/integer_number_converter.rs", "IntegerNumberConverter"),
    ("crates/easyexcel/src/converters/integer/integer_string_converter.rs", "IntegerStringConverter"),
    ("crates/easyexcel/src/converters/localdate/local_date_date_converter.rs", "LocalDateDateConverter"),
    ("crates/easyexcel/src/converters/localdate/local_date_number_converter.rs", "LocalDateNumberConverter"),
    ("crates/easyexcel/src/converters/localdate/local_date_string_converter.rs", "LocalDateStringConverter"),
    ("crates/easyexcel/src/converters/localdatetime/local_date_time_date_converter.rs", "LocalDateTimeDateConverter"),
    ("crates/easyexcel/src/converters/localdatetime/local_date_time_number_converter.rs", "LocalDateTimeNumberConverter"),
    ("crates/easyexcel/src/converters/localdatetime/local_date_time_string_converter.rs", "LocalDateTimeStringConverter"),
    ("crates/easyexcel/src/converters/longconverter/long_boolean_converter.rs", "LongBooleanConverter"),
    ("crates/easyexcel/src/converters/longconverter/long_number_converter.rs", "LongNumberConverter"),
    ("crates/easyexcel/src/converters/longconverter/long_string_converter.rs", "LongStringConverter"),
    ("crates/easyexcel/src/converters/shortconverter/short_boolean_converter.rs", "ShortBooleanConverter"),
    ("crates/easyexcel/src/converters/shortconverter/short_number_converter.rs", "ShortNumberConverter"),
    ("crates/easyexcel/src/converters/shortconverter/short_string_converter.rs", "ShortStringConverter"),
    ("crates/easyexcel/src/converters/string/string_boolean_converter.rs", "StringBooleanConverter"),
    ("crates/easyexcel/src/converters/string/string_error_converter.rs", "StringErrorConverter"),
    ("crates/easyexcel/src/converters/string/string_image_converter.rs", "StringImageConverter"),
    ("crates/easyexcel/src/converters/string/string_number_converter.rs", "StringNumberConverter"),
    ("crates/easyexcel/src/converters/string/string_string_converter.rs", "StringStringConverter"),
    ("crates/easyexcel/src/converters/url/url_image_converter.rs", "UrlImageConverter"),
];
const FIELD_UTILS_ADAPTER: &str = "crates/easyexcel/src/util/field_utils.rs";
const READ_HOLDER_CONTRACT: &str =
    "crates/easyexcel/src/read/metadata/holder/read_holder.rs";
const WRITE_HOLDER_CONTRACT: &str =
    "crates/easyexcel/src/write/metadata/holder/write_holder.rs";
const ABSTRACT_READ_HOLDER: &str =
    "crates/easyexcel/src/read/metadata/holder/abstract_read_holder.rs";
const ABSTRACT_WRITE_HOLDER: &str =
    "crates/easyexcel/src/write/metadata/holder/abstract_write_holder.rs";
const WRITE_BASIC_PARAMETER: &str =
    "crates/easyexcel/src/write/metadata/write_basic_parameter.rs";
const READ_WORKBOOK_HOLDER: &str =
    "crates/easyexcel/src/read/metadata/holder/read_workbook_holder.rs";
const READ_SHEET_HOLDER: &str =
    "crates/easyexcel/src/read/metadata/holder/read_sheet_holder.rs";
const READ_ROW_HOLDER: &str =
    "crates/easyexcel/src/read/metadata/holder/read_row_holder.rs";
const WRITE_WORKBOOK_HOLDER: &str =
    "crates/easyexcel/src/write/metadata/holder/write_workbook_holder.rs";
const WRITE_SHEET_HOLDER: &str =
    "crates/easyexcel/src/write/metadata/holder/write_sheet_holder.rs";
const WRITE_TABLE_HOLDER: &str =
    "crates/easyexcel/src/write/metadata/holder/write_table_holder.rs";
const COMPATIBLE_READER_BUILDER: &str =
    "crates/easyexcel/src/read/builder/excel_reader_builder.rs";
const TYPED_READER_BUILDER: &str = "crates/easyexcel/src/excel_reader_builder.rs";
const EXCEL_READER_ADAPTER: &str = "crates/easyexcel/src/excel_reader.rs";
const WRITER_SHEET_BUILDER: &str =
    "crates/easyexcel/src/write/builder/excel_writer_sheet_builder.rs";
const CONCRETE_READ_HOLDER_CONTRACTS: &[&str] = &[
    "crates/easyexcel/src/read/metadata/holder/read_workbook_holder.rs",
    "crates/easyexcel/src/read/metadata/holder/read_sheet_holder.rs",
    "crates/easyexcel/src/read/metadata/holder/csv/csv_read_workbook_holder.rs",
    "crates/easyexcel/src/read/metadata/holder/csv/csv_read_sheet_holder.rs",
    "crates/easyexcel/src/read/metadata/holder/xls/xls_read_workbook_holder.rs",
    "crates/easyexcel/src/read/metadata/holder/xls/xls_read_sheet_holder.rs",
    "crates/easyexcel/src/read/metadata/holder/xlsx/xlsx_read_workbook_holder.rs",
    "crates/easyexcel/src/read/metadata/holder/xlsx/xlsx_read_sheet_holder.rs",
];
const FORMAT_READ_SHEET_HOLDERS: &[&str] = &[
    "crates/easyexcel/src/read/metadata/holder/csv/csv_read_sheet_holder.rs",
    "crates/easyexcel/src/read/metadata/holder/xls/xls_read_sheet_holder.rs",
    "crates/easyexcel/src/read/metadata/holder/xlsx/xlsx_read_sheet_holder.rs",
];
const CONCRETE_WRITE_HOLDER_CONTRACTS: &[&str] = &[
    "crates/easyexcel/src/write/metadata/holder/write_workbook_holder.rs",
    "crates/easyexcel/src/write/metadata/holder/write_sheet_holder.rs",
    "crates/easyexcel/src/write/metadata/holder/write_table_holder.rs",
];
const STYLE_PROPERTY_ADAPTER: &str =
    "crates/easyexcel/src/metadata/property/style_property.rs";
const WRITE_CELL_STYLE_ADAPTER: &str =
    "crates/easyexcel/src/write/metadata/style/write_cell_style.rs";
const WRITE_FONT_ADAPTER: &str =
    "crates/easyexcel/src/write/metadata/style/write_font.rs";
const FONT_PROPERTY_ADAPTER: &str =
    "crates/easyexcel/src/metadata/property/font_property.rs";
const EXCEL_CELL_STYLE_ADAPTER: &str =
    "crates/easyexcel/src/metadata/excel_cell_style.rs";
const HORIZONTAL_STYLE_STRATEGY_ADAPTER: &str =
    "crates/easyexcel/src/write/style/horizontal_cell_style_strategy.rs";
const WRITE_HANDLER_ADAPTER: &str =
    "crates/easyexcel/src/write/handler/write_handler.rs";
const WRITE_WORKBOOK_CONTEXT_ADAPTER: &str =
    "crates/easyexcel/src/context/write_workbook_context.rs";
const WRITE_SHEET_CONTEXT_ADAPTER: &str =
    "crates/easyexcel/src/context/write_sheet_context.rs";
const WRITE_ROW_CONTEXT_ADAPTER: &str = "crates/easyexcel/src/context/write_row_context.rs";
const WRITE_CELL_CONTEXT_ADAPTER: &str = "crates/easyexcel/src/context/write_cell_context.rs";
const WRITE_HANDLER_CHAINS: &[&str] = &[
    "crates/easyexcel/src/write/handler/chain/workbook_handler_execution_chain.rs",
    "crates/easyexcel/src/write/handler/chain/sheet_handler_execution_chain.rs",
    "crates/easyexcel/src/write/handler/chain/row_handler_execution_chain.rs",
    "crates/easyexcel/src/write/handler/chain/cell_handler_execution_chain.rs",
];
const WRITE_HANDLER_MARKERS: &[&str] = &[
    "crates/easyexcel/src/write/handler/workbook_write_handler.rs",
    "crates/easyexcel/src/write/handler/sheet_write_handler.rs",
    "crates/easyexcel/src/write/handler/row_write_handler.rs",
    "crates/easyexcel/src/write/handler/cell_write_handler.rs",
];
const ABSTRACT_WRITE_HANDLERS: &[&str] = &[
    "crates/easyexcel/src/write/handler/abstract_workbook_write_handler.rs",
    "crates/easyexcel/src/write/handler/abstract_sheet_write_handler.rs",
    "crates/easyexcel/src/write/handler/abstract_row_write_handler.rs",
    "crates/easyexcel/src/write/handler/abstract_cell_write_handler.rs",
];
const DEFAULT_WRITE_HANDLER_LOADER_ADAPTER: &str =
    "crates/easyexcel/src/write/handler/default_write_handler_loader.rs";
const CELL_FORMAT_CONTEXT_ADAPTER: &str =
    "crates/easyexcel/src/write/sheet_style_context/cell_format_context.rs";
const XLSX_CELL_EMISSION_ADAPTER: &str =
    "crates/easyexcel/src/write/excel_writer_core/xlsx_cell_emission.rs";
const XLS_CELL_EMISSION_ADAPTER: &str =
    "crates/easyexcel/src/write/excel_writer_core/xls_write.rs";
const CONTENT_STYLE_ADAPTER: &str =
    "crates/easyexcel/src/annotation/write/style/content_style.rs";
const JAVA_ENUM_CONTRACT_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/enums/boolean_enum.rs",
    "crates/easyexcel/src/enums/byte_order_mark_enum.rs",
    "crates/easyexcel/src/enums/cache_location_enum.rs",
    "crates/easyexcel/src/enums/cell_data_type_enum.rs",
    "crates/easyexcel/src/enums/cell_extra_type_enum.rs",
    "crates/easyexcel/src/enums/head_kind_enum.rs",
    "crates/easyexcel/src/enums/holder_enum.rs",
    "crates/easyexcel/src/enums/numeric_cell_type_enum.rs",
    "crates/easyexcel/src/enums/read_default_return_enum.rs",
    "crates/easyexcel/src/enums/row_type_enum.rs",
    "crates/easyexcel/src/enums/write_direction_enum.rs",
    "crates/easyexcel/src/enums/write_last_row_type_enum.rs",
    "crates/easyexcel/src/enums/write_template_analysis_cell_type_enum.rs",
    "crates/easyexcel/src/enums/write_type_enum.rs",
    "crates/easyexcel/src/enums/poi/fill_pattern_type_enum.rs",
    "crates/easyexcel/src/enums/poi/border_style_enum.rs",
    "crates/easyexcel/src/enums/poi/horizontal_alignment_enum.rs",
    "crates/easyexcel/src/enums/poi/vertical_alignment_enum.rs",
    "crates/easyexcel/src/metadata/data/anchor_type.rs",
    "crates/easyexcel/src/metadata/data/hyperlink_data/hyperlink_type.rs",
    "crates/easyexcel/src/metadata/data/image_type.rs",
    "crates/easyexcel/src/support/excel_type_enum.rs",
];
const RUST_PUBLIC_API_GENERATOR: &str = "scripts/generate_rust_public_api.py";
const PUBLIC_API_MAPPING_SUGGESTER: &str = "scripts/suggest_public_api_mapping.py";
const EXCEL_ANALYSER_ADAPTER: &str = "crates/easyexcel/src/analysis/excel_analyser_impl.rs";
const GZIP_SPILL_ADAPTER: &str = "crates/easyexcel/src/write/gzip_spill.rs";
const COMMENT_DATA_ADAPTER: &str = "crates/easyexcel/src/metadata/data/comment_data.rs";
const READ_CELL_DATA_ADAPTER: &str = "crates/easyexcel/src/metadata/data/read_cell_data.rs";
const CLIENT_ANCHOR_DATA_ADAPTER: &str =
    "crates/easyexcel/src/metadata/data/client_anchor_data.rs";
const HYPERLINK_DATA_ADAPTER: &str = "crates/easyexcel/src/metadata/data/hyperlink_data.rs";
const IMAGE_DATA_ADAPTER: &str = "crates/easyexcel/src/metadata/data/image_data.rs";
const RICH_TEXT_STRING_DATA_ADAPTER: &str =
    "crates/easyexcel/src/metadata/data/rich_text_string_data.rs";
const DATA_FORMAT_DATA_ENGINE: &str = "crates/easyexcel-model/src/model/data_format_data.rs";
const CSV_WORKBOOK_ENGINE: &str = "crates/easyexcel-csv/src/csv/csv_workbook.rs";
const CSV_SHEET_ENGINE: &str = "crates/easyexcel-csv/src/csv/csv_sheet.rs";
const CSV_ROW_ENGINE: &str = "crates/easyexcel-csv/src/csv/csv_row.rs";
const CSV_CELL_ENGINE: &str = "crates/easyexcel-csv/src/csv/csv_cell.rs";
const CSV_CELL_STYLE_ENGINE: &str = "crates/easyexcel-csv/src/csv/csv_cell_style.rs";
const CSV_RICH_TEXT_ENGINE: &str = "crates/easyexcel-csv/src/csv/csv_rich_text_string.rs";
const EXCEL_CONTENT_PROPERTY_ADAPTER: &str =
    "crates/easyexcel/src/metadata/property/excel_content_property.rs";
const METADATA_PROPERTY_ADAPTERS: &[(&str, &[&str])] = &[
    (
        "crates/easyexcel/src/metadata/property/style_property.rs",
        &["pub const fn new() -> Self", "pub const fn get_data_format_data", "pub fn set_write_font"],
    ),
    (
        "crates/easyexcel/src/metadata/property/font_property.rs",
        &["pub const fn new() -> Self", "pub fn build(style: ExcelFontStyle)", "pub const fn get_bold", "pub const fn set_bold"],
    ),
    (
        "crates/easyexcel/src/metadata/property/excel_content_property.rs",
        &["pub const EMPTY: Self", "pub fn get_converter", "pub fn set_converter"],
    ),
    (
        "crates/easyexcel/src/metadata/property/excel_head_property.rs",
        &["pub fn from_head_map", "pub const fn get_head_row_number", "pub const fn set_head_row_number", "pub fn set_head_map"],
    ),
    (
        "crates/easyexcel/src/metadata/property/once_absolute_merge_property.rs",
        &["pub const fn new(", "pub const fn get_first_row_index", "pub const fn set_last_column_index"],
    ),
    (
        "crates/easyexcel/src/metadata/property/date_time_format_property.rs",
        &["pub fn build(", "pub fn get_format", "pub const fn set_use_1904windowing"],
    ),
    (
        "crates/easyexcel/src/metadata/property/loop_merge_property.rs",
        &["pub const fn new(", "pub const fn get_each_row", "pub const fn set_column_extend"],
    ),
    (
        "crates/easyexcel/src/metadata/property/number_format_property.rs",
        &["pub fn build(", "pub fn get_format", "pub const fn set_rounding_mode"],
    ),
    (
        "crates/easyexcel/src/metadata/property/row_height_property.rs",
        &["pub const fn new(", "pub const fn get_height", "pub const fn set_height"],
    ),
    (
        "crates/easyexcel/src/metadata/property/column_width_property.rs",
        &["pub const fn new(", "pub const fn get_width", "pub const fn set_width"],
    ),
];
const WRITE_STYLE_VALUE_ADAPTERS: &[(&str, &[&str])] = &[
    (
        "crates/easyexcel/src/write/metadata/style/write_cell_style.rs",
        &["pub const fn new() -> Self", "pub fn build(", "pub fn merge(source: &Self, target: &mut Self)", "pub fn set_write_font"],
    ),
    (
        "crates/easyexcel/src/write/metadata/style/write_font.rs",
        &["pub const fn new() -> Self", "pub fn merge(source: &Self, target: &mut Self)", "pub fn set_font_name", "pub const fn set_bold"],
    ),
];
const WRITE_STYLE_ANNOTATION_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/annotation/write/style/content_style.rs",
    "crates/easyexcel/src/annotation/write/style/head_style.rs",
    "crates/easyexcel/src/annotation/write/style/content_font_style.rs",
    "crates/easyexcel/src/annotation/write/style/head_font_style.rs",
    "crates/easyexcel/src/annotation/write/style/once_absolute_merge.rs",
    "crates/easyexcel/src/annotation/write/style/content_loop_merge.rs",
    "crates/easyexcel/src/annotation/write/style/column_width.rs",
    "crates/easyexcel/src/annotation/write/style/content_row_height.rs",
    "crates/easyexcel/src/annotation/write/style/head_row_height.rs",
];
const WRITE_STYLE_STRATEGY_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/write/style/abstract_cell_style_strategy.rs",
    "crates/easyexcel/src/write/style/abstract_vertical_cell_style_strategy.rs",
    "crates/easyexcel/src/write/style/default_style.rs",
    "crates/easyexcel/src/write/style/horizontal_cell_style_strategy.rs",
    "crates/easyexcel/src/write/style/column/abstract_column_width_style_strategy.rs",
    "crates/easyexcel/src/write/style/column/abstract_head_column_width_style_strategy.rs",
    "crates/easyexcel/src/write/style/column/longest_match_column_width_style_strategy.rs",
    "crates/easyexcel/src/write/style/column/simple_column_width_style_strategy.rs",
    "crates/easyexcel/src/write/style/row/abstract_row_height_style_strategy.rs",
    "crates/easyexcel/src/write/style/row/simple_row_height_style_strategy.rs",
];
const JAVA_CONSTANT_CONTRACT_ADAPTERS: &[(&str, &[&str])] = &[
    (
        "crates/easyexcel/src/constant/excel_xml_constants.rs",
        &["pub struct ExcelXmlConstants", "pub const ATTRIBUTE_RID", "pub const SHAREDSTRINGS_NS2_RPH_TAG"],
    ),
    (
        "crates/easyexcel/src/constant/builtin_formats.rs",
        &["pub struct BuiltinFormats", "pub const GENERAL", "pub fn get_builtin_format", "pub fn switch_builtin_formats_map"],
    ),
    (
        "crates/easyexcel/src/constant/order_constant.rs",
        &["pub struct OrderConstant", "pub const DEFAULT_DEFINE_STYLE", "pub const FILL_STYLE"],
    ),
    (
        "crates/easyexcel/src/constant/easy_excel_constants.rs",
        &["pub struct EasyExcelConstants", "pub fn excel_math_context()", "EXCEL_MATH_CONTEXT_PRECISION"],
    ),
];
const JAVA_EXCEPTION_CONTRACT_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/exception/excel_runtime_exception.rs",
    "crates/easyexcel/src/exception/excel_common_exception.rs",
    "crates/easyexcel/src/exception/excel_analysis_exception.rs",
    "crates/easyexcel/src/exception/excel_analysis_stop_exception.rs",
    "crates/easyexcel/src/exception/excel_analysis_stop_sheet_exception.rs",
    "crates/easyexcel/src/exception/excel_generate_exception.rs",
    "crates/easyexcel/src/exception/excel_data_convert_exception.rs",
    "crates/easyexcel/src/exception/excel_write_data_convert_exception.rs",
];
const ANALYSIS_CONTEXT_IMPL_ADAPTER: &str = "crates/easyexcel/src/context/analysis_context_impl.rs";
const XLS_COMMENT_ENGINE: &str =
    "crates/easyexcel-xls/src/biff8/workbook/biff8cell_to_write_bof/biff8comment.rs";
const XLS_COMMENT_ENCODER: &str =
    "crates/easyexcel-xls/src/biff8/workbook/write_comments.rs";
const XLS_DRAWING_GROUP_ENGINE: &str =
    "crates/easyexcel-xls/src/biff8/workbook/write_charts.rs";
const XLS_WORKBOOK_DRAWING_PLAN: &str =
    "crates/easyexcel-xls/src/biff8/workbook/biff8cell_to_write_bof.rs";
const XLS_COMMENT_SHEET_ENGINE: &str =
    "crates/easyexcel-xls/src/biff8/workbook/biff8cell_to_write_bof/biff8sheet.rs";
const XLS_COMMENT_TEMPLATE_ENGINE: &str =
    "crates/easyexcel-xls/src/biff8/template/rawrecord_to_scalar_placeholder_key.rs";
const XLS_COMMENT_WRITE_ADAPTER: &str =
    "crates/easyexcel/src/write/excel_writer_core/xls_write.rs";
const XLS_COMMENT_TEMPLATE_ADAPTER: &str =
    "crates/easyexcel/src/write/xls_adapter/template.rs";
const XLSX_COMMENT_ENGINE: &str =
    "crates/easyexcel-xlsx/src/xlsx/generation/xlsx_max_rows_to_build_format.rs";
const XLSX_COMMENT_ROW_ADAPTER: &str =
    "crates/easyexcel/src/write/excel_writer_core/xlsx_row_emission.rs";
const XLSX_COMMENT_MUTATION_ADAPTER: &str =
    "crates/easyexcel/src/write/excel_writer_core/xlsx_workbook_mutations.rs";
const XLSX_COMMENT_TEMPLATE_ENGINE: &str =
    "crates/easyexcel-xlsx/src/xlsx/template_package.rs";
const COMMENT_MUTATION_PLAN: &str =
    "crates/easyexcel/src/context/write_mutation_plan.rs";

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
const ANALYSIS_PUBLIC_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/analysis/excel_analyser.rs",
    "crates/easyexcel/src/analysis/excel_analyser_impl.rs",
    "crates/easyexcel/src/analysis/excel_read_executor.rs",
    "crates/easyexcel/src/analysis/csv/csv_excel_read_executor.rs",
    "crates/easyexcel/src/analysis/v03/xls_list_sheet_listener.rs",
    "crates/easyexcel/src/analysis/v03/xls_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/xls_sax_analyser.rs",
    "crates/easyexcel/src/analysis/v03/handlers/abstract_xls_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/blank_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/bof_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/bool_err_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/bound_sheet_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/dummy_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/eof_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/formula_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/hyperlink_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/index_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/label_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/label_sst_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/merge_cells_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/note_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/number_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/obj_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/rk_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/sst_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/string_record_handler.rs",
    "crates/easyexcel/src/analysis/v03/handlers/text_object_record_handler.rs",
    "crates/easyexcel/src/analysis/v07/xlsx_sax_analyser.rs",
    "crates/easyexcel/src/analysis/v07/handlers/abstract_cell_value_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/abstract_xlsx_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/cell_formula_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/cell_inline_string_value_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/cell_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/cell_value_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/count_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/hyperlink_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/merge_cell_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/row_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/xlsx_tag_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/sax/shared_strings_table_handler.rs",
    "crates/easyexcel/src/analysis/v07/handlers/sax/xlsx_row_handler.rs",
];
const CONTEXT_PUBLIC_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/context/analysis_context.rs",
    "crates/easyexcel/src/context/analysis_context_impl.rs",
    "crates/easyexcel/src/context/write_context.rs",
    "crates/easyexcel/src/context/write_context_impl.rs",
    "crates/easyexcel/src/context/csv/csv_read_context.rs",
    "crates/easyexcel/src/context/csv/default_csv_read_context.rs",
    "crates/easyexcel/src/context/xls/xls_read_context.rs",
    "crates/easyexcel/src/context/xls/default_xls_read_context.rs",
    "crates/easyexcel/src/context/xlsx/xlsx_read_context.rs",
    "crates/easyexcel/src/context/xlsx/default_xlsx_read_context.rs",
];
const UTILITY_PUBLIC_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/util/bean_map_utils.rs",
    "crates/easyexcel/src/util/easy_excel_naming_policy.rs",
    "crates/easyexcel/src/util/boolean_utils.rs",
    "crates/easyexcel/src/util/class_utils.rs",
    "crates/easyexcel/src/util/content_property_key.rs",
    "crates/easyexcel/src/util/field_cache_key.rs",
    "crates/easyexcel/src/util/converter_utils.rs",
    "crates/easyexcel/src/util/date_utils.rs",
    "crates/easyexcel/src/util/easy_excel_temp_file_creation_strategy.rs",
    "crates/easyexcel/src/util/field_utils.rs",
    "crates/easyexcel/src/util/file_type_utils.rs",
    "crates/easyexcel/src/util/file_utils.rs",
    "crates/easyexcel/src/util/int_utils.rs",
    "crates/easyexcel/src/util/io_utils.rs",
    "crates/easyexcel/src/util/list_utils.rs",
    "crates/easyexcel/src/util/map_utils.rs",
    "crates/easyexcel/src/util/number_data_formatter_utils.rs",
    "crates/easyexcel/src/util/number_utils.rs",
    "crates/easyexcel/src/util/poi_utils.rs",
    "crates/easyexcel/src/util/position_utils.rs",
    "crates/easyexcel/src/util/sheet_utils.rs",
    "crates/easyexcel/src/util/string_utils.rs",
    "crates/easyexcel/src/util/style_util.rs",
    "crates/easyexcel/src/util/validate.rs",
    "crates/easyexcel/src/util/work_book_util.rs",
    "crates/easyexcel/src/util/write_handler_utils.rs",
];
const READ_RUNTIME_PUBLIC_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/cache/read_cache.rs",
    "crates/easyexcel/src/cache/map_cache.rs",
    "crates/easyexcel/src/cache/xls_cache.rs",
    "crates/easyexcel/src/cache/selector/read_cache_selector.rs",
    "crates/easyexcel/src/cache/selector/eternal_read_cache_selector.rs",
    "crates/easyexcel/src/cache/selector/simple_read_cache_selector.rs",
    "crates/easyexcel/src/event/abstract_ignore_exception_read_listener.rs",
    "crates/easyexcel/src/event/analysis_event_listener.rs",
    "crates/easyexcel/src/event/handler.rs",
    "crates/easyexcel/src/event/listener.rs",
    "crates/easyexcel/src/event/not_repeat_executor.rs",
    "crates/easyexcel/src/event/order.rs",
    "crates/easyexcel/src/event/sync_read_listener.rs",
    "crates/easyexcel/src/read/listener/ignore_exception_read_listener.rs",
    "crates/easyexcel/src/read/listener/model_build_event_listener.rs",
    "crates/easyexcel/src/read/listener/page_read_listener.rs",
    "crates/easyexcel/src/read/listener/read_listener.rs",
    "crates/easyexcel/src/read/processor/analysis_event_processor.rs",
    "crates/easyexcel/src/read/processor/default_analysis_event_processor.rs",
    "crates/easyexcel/src/read/metadata/read_basic_parameter.rs",
    "crates/easyexcel/src/read/metadata/read_sheet.rs",
    "crates/easyexcel/src/read/metadata/read_workbook.rs",
    "crates/easyexcel/src/read/metadata/property/excel_read_head_property.rs",
];
const CORE_METADATA_PUBLIC_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/metadata/head.rs",
    "crates/easyexcel/src/metadata/abstract_holder.rs",
    "crates/easyexcel/src/metadata/global_configuration.rs",
    "crates/easyexcel/src/metadata/cell_range.rs",
    "crates/easyexcel/src/metadata/field_wrapper.rs",
    "crates/easyexcel/src/metadata/font.rs",
    "crates/easyexcel/src/metadata/field_cache.rs",
    "crates/easyexcel/src/metadata/abstract_cell.rs",
    "crates/easyexcel/src/metadata/format/data_formatter.rs",
    "crates/easyexcel/src/metadata/format/excel_general_number_format.rs",
    "crates/easyexcel/src/metadata/configuration_holder.rs",
    "crates/easyexcel/src/metadata/cell.rs",
    "crates/easyexcel/src/metadata/null_object.rs",
    "crates/easyexcel/src/metadata/holder.rs",
];
const WRITE_RUNTIME_PUBLIC_ADAPTERS: &[&str] = &[
    "crates/easyexcel/src/metadata/csv/csv_data_format.rs",
    "crates/easyexcel/src/write/executor/abstract_excel_write_executor.rs",
    "crates/easyexcel/src/write/executor/excel_write_add_executor.rs",
    "crates/easyexcel/src/write/executor/excel_write_fill_executor.rs",
    "crates/easyexcel/src/write/executor/unique_data_flag_key.rs",
    "crates/easyexcel/src/write/merge/abstract_merge_strategy.rs",
    "crates/easyexcel/src/write/merge/loop_merge_strategy.rs",
    "crates/easyexcel/src/write/merge/once_absolute_merge_strategy.rs",
    "crates/easyexcel/src/write/metadata/row_data.rs",
    "crates/easyexcel/src/write/metadata/collection_row_data.rs",
    "crates/easyexcel/src/write/metadata/map_row_data.rs",
    "crates/easyexcel/src/write/metadata/write_basic_parameter.rs",
    "crates/easyexcel/src/write/metadata/write_workbook.rs",
    "crates/easyexcel/src/write/metadata/write_sheet.rs",
    "crates/easyexcel/src/write/metadata/write_table.rs",
    "crates/easyexcel/src/write/metadata/fill/analysis_cell.rs",
    "crates/easyexcel/src/write/metadata/fill/fill_config.rs",
    "crates/easyexcel/src/write/metadata/fill/fill_wrapper.rs",
    "crates/easyexcel/src/write/property/excel_write_head_property.rs",
];
