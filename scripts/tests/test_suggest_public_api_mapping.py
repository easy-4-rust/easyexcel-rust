import importlib.util
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "suggest_public_api_mapping.py"
SPEC = importlib.util.spec_from_file_location("suggest_public_api_mapping", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


def rust_manifest(*items):
    return {"packages": [{"snapshots": [{"items": list(items)}]}]}


def test_maps_camel_case_method_to_snake_case_without_verifying():
    java = {
        "types": [],
        "members": [
            {
                "id": "x.ExcelReaderBuilder#headRowNumber(I)Lx/ExcelReaderBuilder;",
                "kind": "method",
                "name": "headRowNumber",
                "owner": "x.ExcelReaderBuilder",
            }
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:1",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelReaderBuilder::head_row_number(self, u32) -> Self",
        }
    )
    entry = MODULE.suggest(java, rust)[0]
    assert entry["status"] == "candidate"
    assert entry["rust_ids"] == ["rust:1"]
    assert not entry["behavior_tests"]


def test_maps_java_public_static_field_to_rust_associated_const():
    java = {
        "types": [],
        "members": [
            {
                "id": "x.XlsxSaxAnalyser#FIELD:SHARED_STRINGS_PART_NAME:Lx/PartName;",
                "kind": "field",
                "name": "SHARED_STRINGS_PART_NAME",
                "owner": "x.XlsxSaxAnalyser",
            }
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:shared-strings-part-name",
            "kind": "const",
            "signature": (
                "pub const easyexcel::analysis::XlsxSaxAnalyser::"
                "SHARED_STRINGS_PART_NAME: &'static str"
            ),
        }
    )

    entry = MODULE.suggest(java, rust)[0]

    assert entry["status"] == "candidate"
    assert entry["rust_ids"] == ["rust:shared-strings-part-name"]


def test_preserves_ambiguous_overloads_as_fail_closed_candidates():
    java = {
        "types": [],
        "members": [
            {
                "id": "x.Foo#read()V",
                "kind": "method",
                "name": "read",
                "owner": "x.Foo",
            }
        ],
    }
    rust = rust_manifest(
        {"id": "rust:1", "kind": "function", "signature": "pub fn x::Foo::read()"},
        {"id": "rust:2", "kind": "function", "signature": "pub fn y::Foo::read(u32)"},
    )
    entry = MODULE.suggest(java, rust)[0]
    assert entry["status"] == "ambiguous"
    assert entry["rust_ids"] == ["rust:1", "rust:2"]


def test_prefers_root_reexport_over_deeper_compatibility_path():
    java = {
        "types": [{"id": "x.Foo", "kind": "type", "owner": "x.Foo"}],
        "members": [],
    }
    rust = rust_manifest(
        {"id": "rust:root", "kind": "struct", "signature": "pub struct easyexcel::Foo"},
        {
            "id": "rust:deep",
            "kind": "struct",
            "signature": "pub struct easyexcel::metadata::foo::Foo",
        },
    )
    entry = MODULE.suggest(java, rust)[0]
    assert entry["status"] == "candidate"
    assert entry["rust_ids"] == ["rust:root"]


def test_maps_easyexcel_factory_sheet_overloads_to_explicit_facade_methods():
    java = {
        "types": [],
        "members": [
            {
                "id": "com.alibaba.excel.EasyExcelFactory#readSheet(Ljava/lang/Integer;)Lx/Builder;",
                "kind": "method",
                "name": "readSheet",
                "owner": "com.alibaba.excel.EasyExcelFactory",
            },
            {
                "id": "com.alibaba.excel.EasyExcelFactory#writerSheet(Ljava/lang/String;)Lx/Builder;",
                "kind": "method",
                "name": "writerSheet",
                "owner": "com.alibaba.excel.EasyExcelFactory",
            },
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:read-sheet-index",
            "kind": "function",
            "signature": "pub fn easyexcel::EasyExcel::read_sheet_index(i32) -> Builder",
        },
        {
            "id": "rust:writer-sheet-name",
            "kind": "function",
            "signature": "pub fn easyexcel::EasyExcel::writer_sheet_builder_name(String) -> Builder",
        },
    )

    entries = MODULE.suggest(java, rust)

    assert entries[0]["rust_ids"] == ["rust:read-sheet-index"]
    assert entries[1]["rust_ids"] == ["rust:writer-sheet-name"]


def test_maps_input_stream_listener_overload_after_explicit_facade_is_available():
    java = {
        "types": [],
        "members": [
            {
                "id": "com.alibaba.excel.EasyExcelFactory#read(Ljava/io/InputStream;Lx/ReadListener;)Lx/Builder;",
                "kind": "method",
                "name": "read",
                "owner": "com.alibaba.excel.EasyExcelFactory",
            }
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:reader-input-listener",
            "kind": "function",
            "signature": "pub fn easyexcel::EasyExcel::read_from_input_stream(R, L) -> Builder",
        }
    )

    entry = MODULE.suggest(java, rust)[0]

    assert entry["status"] == "candidate"
    assert entry["rust_ids"] == ["rust:reader-input-listener"]


def test_maps_cargo_public_api_type_alias_kind():
    java = {
        "types": [{"id": "x.Factory", "kind": "type", "owner": "x.Factory"}],
        "members": [],
    }
    rust = rust_manifest(
        {
            "id": "rust:factory-alias",
            "kind": "type",
            "signature": "pub type easyexcel::Factory = easyexcel::Facade",
        }
    )

    entry = MODULE.suggest(java, rust)[0]

    assert entry["status"] == "candidate"
    assert entry["rust_ids"] == ["rust:factory-alias"]


def test_maps_empty_java_marker_interface_to_unique_rust_marker_query():
    java = {
        "types": [
            {
                "id": "x.IgnorableHandler",
                "kind": "type",
                "owner": "x.IgnorableHandler",
                "type_kind": "interface",
            }
        ],
        "members": [],
    }
    rust = rust_manifest(
        {
            "id": "rust:marker-query",
            "kind": "function",
            "signature": "pub fn easyexcel::IgnorableHandler::is_ignorable(&self) -> bool",
        }
    )

    entry = MODULE.suggest(java, rust)[0]

    assert entry["status"] == "candidate"
    assert entry["rust_ids"] == ["rust:marker-query"]


def test_maps_abstract_java_base_to_rust_supertrait_contract():
    owner = "x.AbstractXlsRecordHandler"
    java = {
        "types": [
            {
                "id": owner,
                "kind": "type",
                "owner": owner,
                "type_kind": "class",
            }
        ],
        "members": [
            {
                "id": f"{owner}#<init>()V",
                "kind": "constructor",
                "name": "<init>",
                "owner": owner,
            },
            {
                "id": f"{owner}#support()Z",
                "kind": "method",
                "name": "support",
                "owner": owner,
            },
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:abstract-trait",
            "kind": "trait",
            "signature": (
                "pub trait easyexcel::analysis::AbstractXlsRecordHandler: "
                "easyexcel::analysis::XlsRecordHandler"
            ),
        },
        {
            "id": "rust:support",
            "kind": "function",
            "signature": "pub fn easyexcel::XlsRecordHandler::support(&self) -> bool",
        },
    )

    entries = MODULE.suggest(java, rust)

    assert entries[0]["rust_ids"] == ["rust:abstract-trait"]
    assert entries[1]["rust_ids"] == ["rust:abstract-trait"]
    assert entries[2]["rust_ids"] == ["rust:support"]


def test_maps_excel_reader_zero_arg_and_getter_aliases_explicitly():
    java = {
        "types": [],
        "members": [
            {
                "id": "com.alibaba.excel.ExcelReader#getAnalysisContext()Lx/Context;",
                "kind": "method",
                "name": "getAnalysisContext",
                "owner": "com.alibaba.excel.ExcelReader",
            },
            {
                "id": "com.alibaba.excel.ExcelReader#read()V",
                "kind": "method",
                "name": "read",
                "owner": "com.alibaba.excel.ExcelReader",
            },
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:context",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelReader::get_analysis_context(&self) -> &Context",
        },
        {
            "id": "rust:deprecated-read",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelReader::read_deprecated(&mut self) -> Result<()> ",
        },
        {
            "id": "rust:sheet-read",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelReader::read(&mut self, sheets: &[ReadSheet])",
        },
    )

    entries = MODULE.suggest(java, rust)

    assert entries[0]["rust_ids"] == ["rust:context"]
    assert entries[1]["rust_ids"] == ["rust:deprecated-read"]


def test_maps_excel_writer_overloads_to_java_compatible_builder_impl_methods():
    owner = "com.alibaba.excel.ExcelWriter"
    java = {
        "types": [{"id": owner, "kind": "type", "owner": owner}],
        "members": [
            {
                "id": f"{owner}#<init>(Lcom/alibaba/excel/write/metadata/WriteWorkbook;)V",
                "kind": "constructor",
                "name": "<init>",
                "owner": owner,
            },
            {
                "id": f"{owner}#write(Ljava/util/Collection;Lx/WriteSheet;)Lx/ExcelWriter;",
                "kind": "method",
                "name": "write",
                "owner": owner,
            },
            {
                "id": f"{owner}#write(Ljava/util/function/Supplier;Lx/WriteSheet;)Lx/ExcelWriter;",
                "kind": "method",
                "name": "write",
                "owner": owner,
            },
            {
                "id": f"{owner}#write(Ljava/util/function/Supplier;Lx/WriteSheet;Lcom/alibaba/excel/write/metadata/WriteTable;)Lx/ExcelWriter;",
                "kind": "method",
                "name": "write",
                "owner": owner,
            },
            {
                "id": f"{owner}#fill(Ljava/lang/Object;Lx/WriteSheet;)Lx/ExcelWriter;",
                "kind": "method",
                "name": "fill",
                "owner": owner,
            },
            {
                "id": f"{owner}#fill(Ljava/util/function/Supplier;Lcom/alibaba/excel/write/metadata/fill/FillConfig;Lx/WriteSheet;)Lx/ExcelWriter;",
                "kind": "method",
                "name": "fill",
                "owner": owner,
            },
            {
                "id": f"{owner}#writeContext()Lx/WriteContext;",
                "kind": "method",
                "name": "writeContext",
                "owner": owner,
            },
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:type",
            "kind": "struct",
            "signature": "pub struct easyexcel::ExcelBuilderImpl",
        },
        *[
                {
                    "id": f"rust:{name}",
                    "kind": "function",
                    "signature": (
                        f"pub fn easyexcel::ExcelBuilderImpl::{name}() -> Result<&mut Self>"
                        if name.startswith("fill")
                        else f"pub fn easyexcel::ExcelBuilderImpl::{name}()"
                    ),
                }
            for name in [
                "from_write_workbook",
                "write",
                "write_with_supplier",
                "write_with_table_supplier",
                "fill_default",
                "fill_with_config_supplier",
                "write_context",
            ]
        ],
    )

    entries = MODULE.suggest(java, rust)

    assert [entry["rust_ids"] for entry in entries] == [
        ["rust:type"],
        ["rust:from_write_workbook"],
        ["rust:fill_default"],
        ["rust:fill_with_config_supplier"],
        ["rust:write"],
        ["rust:write_with_supplier"],
        ["rust:write_with_table_supplier"],
        ["rust:write_context"],
    ]


def test_maps_excel_builder_add_content_overloads_to_distinct_trait_methods():
    owner = "com.alibaba.excel.write.ExcelBuilder"
    java = {
        "types": [],
        "members": [
            {
                "id": f"{owner}#addContent(Ljava/util/Collection;Lx/WriteSheet;)V",
                "kind": "method",
                "name": "addContent",
                "owner": owner,
            },
            {
                "id": (
                    f"{owner}#addContent(Ljava/util/Collection;Lx/WriteSheet;"
                    "Lcom/alibaba/excel/write/metadata/WriteTable;)V"
                ),
                "kind": "method",
                "name": "addContent",
                "owner": owner,
            },
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:add_content",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelBuilder::add_content()",
        },
        {
            "id": "rust:add_content_with_table",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelBuilder::add_content_with_table()",
        },
    )

    entries = MODULE.suggest(java, rust)

    assert [entry["rust_ids"] for entry in entries] == [
        ["rust:add_content"],
        ["rust:add_content_with_table"],
    ]


def test_maps_excel_builder_impl_constructor_overloads_and_void_methods_exactly():
    owner = "com.alibaba.excel.write.ExcelBuilderImpl"
    java = {
        "types": [],
        "members": [
            {
                "id": f"{owner}#<init>(Lcom/alibaba/excel/write/metadata/WriteWorkbook;)V",
                "kind": "constructor",
                "name": "<init>",
                "owner": owner,
            },
            {
                "id": f"{owner}#addContent(Ljava/util/Collection;Lx/WriteSheet;)V",
                "kind": "method",
                "name": "addContent",
                "owner": owner,
            },
            {
                "id": (
                    f"{owner}#addContent(Ljava/util/Collection;Lx/WriteSheet;"
                    "Lcom/alibaba/excel/write/metadata/WriteTable;)V"
                ),
                "kind": "method",
                "name": "addContent",
                "owner": owner,
            },
            {
                "id": f"{owner}#fill(Ljava/lang/Object;Lx/FillConfig;Lx/WriteSheet;)V",
                "kind": "method",
                "name": "fill",
                "owner": owner,
            },
            {
                "id": f"{owner}#finish(Z)V",
                "kind": "method",
                "name": "finish",
                "owner": owner,
            },
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:constructor",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelBuilderImpl::from_write_workbook(WriteWorkbook) -> Result<Self>",
        },
        {
            "id": "rust:add",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelBuilderImpl::add_content() -> Result<()>",
        },
        {
            "id": "rust:add-table",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelBuilderImpl::add_content_with_table() -> Result<()>",
        },
        {
            "id": "rust:fill-void",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelBuilderImpl::fill(&mut self) -> Result<()>",
        },
        {
            "id": "rust:fill-fluent",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelBuilderImpl::fill(&mut self) -> Result<&mut Self>",
        },
        {
            "id": "rust:finish-bool",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelBuilderImpl::finish(&mut self, bool) -> Result<()>",
        },
        {
            "id": "rust:finish-zero",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelBuilderImpl::finish(&mut self) -> Result<()>",
        },
    )

    by_java = {entry["java_id"]: entry["rust_ids"] for entry in MODULE.suggest(java, rust)}
    assert by_java[f"{owner}#<init>(Lcom/alibaba/excel/write/metadata/WriteWorkbook;)V"] == [
        "rust:constructor"
    ]
    assert by_java[f"{owner}#addContent(Ljava/util/Collection;Lx/WriteSheet;)V"] == [
        "rust:add"
    ]
    assert by_java[
        f"{owner}#addContent(Ljava/util/Collection;Lx/WriteSheet;Lcom/alibaba/excel/write/metadata/WriteTable;)V"
    ] == ["rust:add-table"]
    assert by_java[f"{owner}#fill(Ljava/lang/Object;Lx/FillConfig;Lx/WriteSheet;)V"] == [
        "rust:fill-void"
    ]
    assert by_java[f"{owner}#finish(Z)V"] == ["rust:finish-bool"]


def test_maps_excel_analyser_impl_constructor_to_read_workbook_entrypoint():
    owner = "com.alibaba.excel.analysis.ExcelAnalyserImpl"
    java = {
        "types": [],
        "members": [
            {
                "id": f"{owner}#<init>(Lcom/alibaba/excel/read/metadata/ReadWorkbook;)V",
                "kind": "constructor",
                "name": "<init>",
                "owner": owner,
            }
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:new",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelAnalyserImpl::new() -> Self",
        },
        {
            "id": "rust:from-read-workbook",
            "kind": "function",
            "signature": "pub fn easyexcel::ExcelAnalyserImpl::from_read_workbook(ReadWorkbook) -> Result<Self>",
        },
    )

    entries = MODULE.suggest(java, rust)
    assert entries[0]["rust_ids"] == ["rust:from-read-workbook"]


def test_maps_default_csv_context_constructor_to_read_workbook_entrypoint():
    owner = "com.alibaba.excel.context.csv.DefaultCsvReadContext"
    java = {
        "types": [],
        "members": [
            {
                "id": f"{owner}#<init>(Lcom/alibaba/excel/read/metadata/ReadWorkbook;Lcom/alibaba/excel/support/ExcelTypeEnum;)V",
                "kind": "constructor",
                "name": "<init>",
                "owner": owner,
            }
        ],
    }
    rust = rust_manifest(
        {
            "id": "rust:new",
            "kind": "function",
            "signature": "pub fn easyexcel::context::DefaultCsvReadContext::new(&ReadOptions) -> Self",
        },
        {
            "id": "rust:from-read-workbook",
            "kind": "function",
            "signature": "pub fn easyexcel::context::DefaultCsvReadContext::from_read_workbook(&ReadWorkbook, ExcelTypeEnum) -> Self",
        },
    )

    entries = MODULE.suggest(java, rust)
    assert entries[0]["rust_ids"] == ["rust:from-read-workbook"]


def test_maps_default_xls_context_constructors_to_read_workbook_entrypoint():
    for simple_name, package_name in (
        ("DefaultXlsReadContext", "xls"),
        ("DefaultXlsxReadContext", "xlsx"),
    ):
        owner = f"com.alibaba.excel.context.{package_name}.{simple_name}"
        java = {
            "types": [],
            "members": [
                {
                    "id": f"{owner}#<init>(Lcom/alibaba/excel/read/metadata/ReadWorkbook;Lcom/alibaba/excel/support/ExcelTypeEnum;)V",
                    "kind": "constructor",
                    "name": "<init>",
                    "owner": owner,
                }
            ],
        }
        rust = rust_manifest(
            {
                "id": "rust:new",
                "kind": "function",
                "signature": f"pub fn easyexcel::context::{simple_name}::new(&ReadOptions) -> Self",
            },
            {
                "id": "rust:from-read-workbook",
                "kind": "function",
                "signature": f"pub fn easyexcel::context::{simple_name}::from_read_workbook(&ReadWorkbook, ExcelTypeEnum) -> Self",
            },
        )

        entries = MODULE.suggest(java, rust)
        assert entries[0]["rust_ids"] == ["rust:from-read-workbook"]
