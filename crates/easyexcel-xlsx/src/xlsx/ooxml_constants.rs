//! OOXML `SpreadsheetML` 标签与属性常量。
//!
//! 对应 Java：`com.alibaba.excel.constant.ExcelXmlConstants`。这些名称属于
//! XLSX 格式协议，由 `easyexcel-xlsx` 统一维护；EasyExcel 门面只做兼容重导出。

/// `dimension` 标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const DIMENSION_TAG: &str = "dimension";
/// `row` 标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const ROW_TAG: &str = "row";
/// `f`（公式）标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const CELL_FORMULA_TAG: &str = "f";
/// `v`（值）标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const CELL_VALUE_TAG: &str = "v";
/// `t`（内联字符串值）标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const CELL_INLINE_STRING_VALUE_TAG: &str = "t";
/// `c`（单元格）标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const CELL_TAG: &str = "c";
/// `mergeCell` 标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const MERGE_CELL_TAG: &str = "mergeCell";
/// `hyperlink` 标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const HYPERLINK_TAG: &str = "hyperlink";

/// `s` 属性。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const ATTRIBUTE_S: &str = "s";
/// `ref` 属性。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const ATTRIBUTE_REF: &str = "ref";
/// `r` 属性。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const ATTRIBUTE_R: &str = "r";
/// `t` 属性。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const ATTRIBUTE_T: &str = "t";
/// `location` 属性。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const ATTRIBUTE_LOCATION: &str = "location";
/// `r:id` 属性。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const ATTRIBUTE_RID: &str = "r:id";

/// 单元格范围分隔符。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const CELL_RANGE_SPLIT: &str = ":";

/// 共享字符串中的 `t` 标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const SHAREDSTRINGS_T_TAG: &str = "t";
/// 共享字符串中的 `si` 标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const SHAREDSTRINGS_SI_TAG: &str = "si";
/// 共享字符串中的 `rPh`（注音）标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const SHAREDSTRINGS_RPH_TAG: &str = "rPh";
