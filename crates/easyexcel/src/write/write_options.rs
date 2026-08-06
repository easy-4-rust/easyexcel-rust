//! XLSX 写入配置类型。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.WriteBasicParameter`。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/metadata/WriteBasicParameter.java

use std::path::PathBuf;

use crate::core::{CacheLocation, ConverterRegistry, CsvCharset};

use crate::write::cell_style::CellStyle;
use crate::write::merge::loop_merge_strategy::LoopMergeStrategy;
use crate::write::merge_range::MergeRange;

/// XLSX 写入配置。
///
/// 对应 Java：`com.alibaba.excel.write.metadata.WriteBasicParameter`。
/// 通过 `WriteOptions::default()` 创建默认配置，然后使用 builder 方法修改。
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct WriteOptions {
    /// 显式输出类型覆盖文件扩展名。
    /// (Java `WriteWorkbook.excelType`)
    pub excel_type: Option<crate::support::ExcelTypeEnum>,
    /// 工作表名称。
    pub sheet_name: String,
    /// 可选的逻辑工作表编号，从零开始。
    pub sheet_index: Option<usize>,
    /// 工作表名称和字符串单元格的自动修剪。 (Java `autoTrim`)
    pub auto_trim: bool,
    /// 是否启用 Excel 1904 日期窗口。 (Java `use1904windowing`)
    pub use_1904_windowing: bool,
    /// 用于格式化输出的区域设置名称。 (Java `locale`)
    pub locale: String,
    /// 是否对极端 General 格式数字使用科学计数法。
    /// (Java `useScientificFormat`)
    pub use_scientific_format: bool,
    /// 反射元数据的字段缓存位置。 (Java `filedCacheLocation`)
    pub filed_cache_location: CacheLocation,
    /// 是否使用单行常量内存工作表。
    pub constant_memory: bool,
    /// 流式溢出文件是否使用 gzip (SXSSF `setCompressTempFiles`)。
    pub compress_temp_files: bool,
    /// 是否写入列标题。
    pub need_head: bool,
    /// 是否启用 Java 内置的默认样式处理器。
    pub use_default_style: bool,
    /// 是否冻结标题行。
    pub freeze_head: bool,
    /// 显式冻结窗格位置为 `(row, column)`。
    pub freeze_panes: Option<(u32, u16)>,
    /// 要包含的物理列索引。
    pub include_column_indexes: Option<Vec<usize>>,
    /// 要包含的 Rust 字段名称。
    pub include_column_field_names: Option<Vec<String>>,
    /// 要排除的物理列索引。
    pub exclude_column_indexes: Vec<usize>,
    /// 要排除的 Rust 字段名称。
    pub exclude_column_field_names: Vec<String>,
    /// 包含的列是否遵循包含列表的顺序。
    pub order_by_include_column: bool,
    /// 相对标题行索引。 (Java `WriteBasicParameter.relativeHeadRowIndex`)
    pub relative_head_row_index: i32,
    /// 是否自动合并标题。 (Java `WriteBasicParameter.automaticMergeHead`)
    pub automatic_merge_head: bool,
    /// 在写入行数据之前合并的绝对范围。
    pub merge_ranges: Vec<MergeRange>,
    /// 使用的列是否自动适配。
    pub auto_width: bool,
    /// Excel 字符单位的显式列宽。
    pub column_widths: Vec<(u16, u16)>,
    /// 应用于标题单元格的样式。
    pub head_style: CellStyle,
    /// 按相对数据行索引循环的内容样式。
    pub content_styles: Vec<CellStyle>,
    /// 应用于数据行的重复合并策略。
    pub loop_merges: Vec<LoopMergeStrategy>,
    /// 可选的动态多级标题路径，每列一个路径。
    pub dynamic_head: Option<Vec<Vec<String>>>,
    /// 用于 ECMA-376 Agile Encryption 的 XLSX 输出密码。
    pub password: Option<String>,
    /// 用于 CSV 输出的字符编码。
    pub charset: CsvCharset,
    /// CSV 输出是否以编码的字节顺序标记开头。
    pub with_bom: bool,
    /// 状态化 `ExcelOutputStream` 是否由 `finish` 关闭。
    pub auto_close_stream: bool,
    /// `finish_on_exception` 是否在错误前发出累积的行。
    pub write_excel_on_exception: bool,
    /// Java 风格的全局注册转换器。
    pub converters: ConverterRegistry,
    /// 模板文件路径。 (Java `WriteWorkbook.templateFile`)
    pub template_file: Option<PathBuf>,
    /// 内存中的模板字节。 (Java `WriteWorkbook.templateInputStream`)
    pub template_bytes: Option<Vec<u8>>,
    /// 为 `true` 时，`with_template` 使用旧版 calamine → `rust_xlsxwriter` 值重放路径。
    pub use_legacy_template_seed: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            excel_type: None,
            sheet_name: "Sheet1".to_owned(),
            sheet_index: None,
            auto_trim: true,
            use_1904_windowing: false,
            locale: "default".to_owned(),
            use_scientific_format: false,
            filed_cache_location: CacheLocation::ThreadLocal,
            constant_memory: false,
            compress_temp_files: false,
            need_head: true,
            use_default_style: true,
            freeze_head: false,
            freeze_panes: None,
            include_column_indexes: None,
            include_column_field_names: None,
            exclude_column_indexes: Vec::new(),
            exclude_column_field_names: Vec::new(),
            order_by_include_column: false,
            merge_ranges: Vec::new(),
            auto_width: false,
            column_widths: Vec::new(),
            head_style: CellStyle::new().bold(true),
            content_styles: Vec::new(),
            loop_merges: Vec::new(),
            dynamic_head: None,
            password: None,
            charset: CsvCharset::default(),
            with_bom: true,
            auto_close_stream: true,
            write_excel_on_exception: false,
            converters: ConverterRegistry::default(),
            relative_head_row_index: 0,
            automatic_merge_head: true,
            template_file: None,
            template_bytes: None,
            use_legacy_template_seed: false,
        }
    }
}
