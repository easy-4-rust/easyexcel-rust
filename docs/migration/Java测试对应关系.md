# Java 测试 → Rust 测试对应关系

> 基线：Alibaba EasyExcel 4.0.3 `easyexcel-test` 模块
> Java 测试类：38 个（core/ 33 + demo/ 5），demo @Test 方法 40 个
> 更新日期：2026-08-01

## 一、core/ 目录（33 个测试类，全部有 Rust 1:1 对应）

| Java 测试类 | Java 目录 | Rust 测试文件 |
|-------------|-----------|---------------|
| `AnnotationDataTest` | core/annotation | `core_phase1_annotations_1to1_tests.rs` |
| `AnnotationIndexAndNameDataTest` | core/annotation | `core_phase1_annotations_1to1_tests.rs` |
| `BomDataTest` | core/bom | `bom_data_tests.rs` |
| `CacheDataTest` | core/cache | `core_converter_extra_compat_1to1_tests.rs`（mod cache_data_test） |
| `CellDataDataTest` | core/celldata | `core_converter_extra_compat_1to1_tests.rs`（mod cell_data_data_test） |
| `CharsetDataTest` | core/charset | `core_converter_extra_compat_1to1_tests.rs`（mod charset_data_test） |
| `CompatibilityTest` | core/compatibility | `core_converter_extra_compat_1to1_tests.rs`（mod compatibility_test） |
| `ConverterDataTest` | core/converter | `core_converter_extra_compat_1to1_tests.rs`（mod converter_data_test） |
| `ConverterTest` | core/converter | `core_converter_extra_compat_1to1_tests.rs`（mod converter_test） |
| `DateFormatTest` | core/dataformat | `core_converter_extra_compat_1to1_tests.rs`（mod date_format_test） |
| `EncryptDataTest` | core/encrypt | `core_converter_extra_compat_1to1_tests.rs`（mod encrypt_data_test）+ `core_phase5_xls_features_1to1_tests.rs`（mod encrypt_data_test_xls） |
| `ExceptionDataTest` | core/exception | `core_converter_extra_compat_1to1_tests.rs`（mod exception_data_test） |
| `ExcludeOrIncludeDataTest` | core/excludeorinclude | `core_annotation_style_handler_1to1_tests.rs`（mod exclude_or_include_data_test） |
| `ExtraDataTest` | core/extra | `core_converter_extra_compat_1to1_tests.rs`（mod extra_data_test）+ `core_phase5_xls_features_1to1_tests.rs`（mod extra_data_test_xls） |
| `FillDataTest` | core/fill | `core_fill_1to1_tests.rs`（mod fill_data_test） |
| `FillAnnotationDataTest` | core/fill/annotation | `core_fill_1to1_tests.rs`（mod fill_annotation_data_test） |
| `FillStyleAnnotatedTest` | core/fill/style | `core_fill_1to1_tests.rs`（mod fill_style_annotated_test） |
| `FillStyleDataTest` | core/fill/style | `core_fill_1to1_tests.rs`（mod fill_style_data_test） |
| `WriteHandlerTest` | core/handler | `core_annotation_style_handler_1to1_tests.rs`（mod write_handler_test） |
| `ComplexHeadDataTest` | core/head | `core_simple_sort_head_1to1_tests.rs`（mod complex_head_data_test） |
| `ListHeadDataTest` | core/head | `core_simple_sort_head_1to1_tests.rs`（mod list_head_data_test） |
| `NoHeadDataTest` | core/head | `core_simple_sort_head_1to1_tests.rs`（mod no_head_data_test） |
| `LargeDataTest` | core/large | `core_converter_extra_compat_1to1_tests.rs`（mod large_data_test） |
| `MultipleSheetsDataTest` | core/multiplesheets | `core_simple_sort_head_1to1_tests.rs`（mod multiple_sheets_data_test） |
| `NoModelDataTest` | core/nomodel | `core_simple_sort_head_1to1_tests.rs`（mod no_model_data_test） |
| `UnCamelDataTest` | core/noncamel | `core_simple_sort_head_1to1_tests.rs`（mod un_camel_data_test） |
| `ParameterDataTest` | core/parameter | `core_simple_sort_head_1to1_tests.rs`（mod parameter_data_test） |
| `RepetitionDataTest` | core/repetition | `core_simple_sort_head_1to1_tests.rs`（mod repetition_data_test） |
| `SimpleDataTest` | core/simple | `core_simple_sort_head_1to1_tests.rs`（mod simple_data_test） |
| `SkipDataTest` | core/skip | `core_simple_sort_head_1to1_tests.rs`（mod skip_data_test） |
| `SortDataTest` | core/sort | `core_simple_sort_head_1to1_tests.rs`（mod sort_data_test） |
| `StyleDataTest` | core/style | `core_annotation_style_handler_1to1_tests.rs`（mod style_data_test） |
| `TemplateDataTest` | core/template | `core_simple_sort_head_1to1_tests.rs`（mod template_data_test） |

## 二、demo/ 目录（40 个 @Test 方法，全部有 Rust 1:1 命名对应）

| Java 测试类 | @Test 数 | Rust 测试文件 |
|-------------|----------|---------------|
| `read/ReadTest` | 12 | `demo_1to1_tests.rs`（read_test_*）+ `demo_parity_tests.rs` |
| `write/WriteTest` | 20 | `demo_1to1_tests.rs`（write_test_*）+ `demo_write_extra_tests.rs` |
| `fill/FillTest` | 6 | `demo_1to1_tests.rs`（fill_test_*） |
| `rare/WriteTest` | 2 | `demo_1to1_tests.rs`（rare_test_*） |

`web/WebTest`：Spring `@Controller`，0 个 `@Test` 方法；Rust 侧由
`easyexcel-support/easyexcel-support-axum/src/tests.rs` 与
`easyexcel-support/easyexcel-support-actix/src/tests.rs` 覆盖等价下载/上传功能。

## 三、Java 测试功能点覆盖核查（方法级 1:1 命名）

Rust 测试遵循 `mod <java_class_snake>` + `fn <java_method_snake>` 命名，
保证 Java 每个测试方法都能在 Rust 测试中检索到同名对应。

关键边界用例核对（与 Java 断言一致）：

| 功能点 | Java 断言 | Rust 对应 |
|--------|-----------|-----------|
| CSV BOM × charset 矩阵（UTF-8/GBK/UTF-16BE） | 中文表头 + 10 行数据 | `bom_data_tests.rs` |
| GBK 写入 UTF-8 读 → 乱码（负向） | assertNotEquals | `charset_data_test` |
| 密码加密写读（xlsx + xls） | password 往返 | `encrypt_data_test` + `encrypt_data_test_xls` |
| 异常传播 / 提前终止 / 每 sheet 停止 | assertThrows + 条数 | `exception_data_test` |
| exclude/include 4 API + orderByIncludeColumn | 列子集断言 | `exclude_or_include_data_test` |
| 批注 / 超链接 / 合并区域 extraRead | 文本与坐标 | `extra_data_test` |
| 模板填充全场景（forceNewRow/横向/FillWrapper） | 坐标断言 | `core_fill_1to1_tests.rs` |
| 多 sheet 逐 sheet 读取 + doReadAll | 累计条数 | `multiple_sheets_data_test` |
| 无模型 List/Map + ReadDefaultReturn 3 模式 | 类型断言 | `no_model_data_test` |
| 同 sheet 多次 write 追加 | 读回 2 条 | `repetition_data_test` |
| 按 sheet 名选择性读取 | 只读第 2 个 | `skip_data_test` |
| withTemplate 保留模板表头样式 | headRowNumber=3 | `template_data_test` |
