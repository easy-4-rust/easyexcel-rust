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
/// 带 `x:` 前缀的 dimension 标签。
pub const X_DIMENSION_TAG: &str = "x:dimension";
/// 带 `ns2:` 前缀的 dimension 标签。
pub const NS2_DIMENSION_TAG: &str = "ns2:dimension";
/// 带 `x:` 前缀的 row 标签。
pub const X_ROW_TAG: &str = "x:row";
/// 带 `ns2:` 前缀的 row 标签。
pub const NS2_ROW_TAG: &str = "ns2:row";
/// 带 `x:` 前缀的公式标签。
pub const X_CELL_FORMULA_TAG: &str = "x:f";
/// 带 `ns2:` 前缀的公式标签。
pub const NS2_CELL_FORMULA_TAG: &str = "ns2:f";
/// 带 `x:` 前缀的值标签。
pub const X_CELL_VALUE_TAG: &str = "x:v";
/// 带 `ns2:` 前缀的值标签。
pub const NS2_CELL_VALUE_TAG: &str = "ns2:v";
/// 带 `x:` 前缀的内联字符串标签。
pub const X_CELL_INLINE_STRING_VALUE_TAG: &str = "x:t";
/// 带 `ns2:` 前缀的内联字符串标签。
pub const NS2_CELL_INLINE_STRING_VALUE_TAG: &str = "ns2:t";
/// 带 `x:` 前缀的单元格标签。
pub const X_CELL_TAG: &str = "x:c";
/// 带 `ns2:` 前缀的单元格标签。
pub const NS2_CELL_TAG: &str = "ns2:c";
/// 带 `x:` 前缀的合并单元格标签。
pub const X_MERGE_CELL_TAG: &str = "x:mergeCell";
/// 带 `ns2:` 前缀的合并单元格标签。
pub const NS2_MERGE_CELL_TAG: &str = "ns2:mergeCell";
/// 带 `x:` 前缀的超链接标签。
pub const X_HYPERLINK_TAG: &str = "x:hyperlink";
/// 带 `ns2:` 前缀的超链接标签。
pub const NS2_HYPERLINK_TAG: &str = "ns2:hyperlink";

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
/// 带 `x:` 前缀的共享字符串文本标签。
pub const SHAREDSTRINGS_X_T_TAG: &str = "x:t";
/// 带 `ns2:` 前缀的共享字符串文本标签。
pub const SHAREDSTRINGS_NS2_T_TAG: &str = "ns2:t";
/// 共享字符串中的 `si` 标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const SHAREDSTRINGS_SI_TAG: &str = "si";
/// 带 `x:` 前缀的共享字符串项标签。
pub const SHAREDSTRINGS_X_SI_TAG: &str = "x:si";
/// 带 `ns2:` 前缀的共享字符串项标签。
pub const SHAREDSTRINGS_NS2_SI_TAG: &str = "ns2:si";
/// 共享字符串中的 `rPh`（注音）标签。
/// 对应 Java：com.alibaba.excel.constant.ExcelXmlConstants。
pub const SHAREDSTRINGS_RPH_TAG: &str = "rPh";
/// 带 `x:` 前缀的共享字符串注音标签。
pub const SHAREDSTRINGS_X_RPH_TAG: &str = "x:rPh";
/// 带 `ns2:` 前缀的共享字符串注音标签。
pub const SHAREDSTRINGS_NS2_RPH_TAG: &str = "ns2:rPh";
