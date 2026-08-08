//! 对应 Java：`com.alibaba.excel.constant.OrderConstant`.

/// The system's own style. (Java `DEFAULT_DEFINE_STYLE`)
/// 对应 Java：com.alibaba.excel.constant.OrderConstant。
pub const DEFAULT_DEFINE_STYLE: i32 = -70_000;

/// Annotation style definition. (Java `ANNOTATION_DEFINE_STYLE`)
/// 对应 Java：com.alibaba.excel.constant.OrderConstant。
pub const ANNOTATION_DEFINE_STYLE: i32 = -60_000;

/// Define style. (Java `DEFINE_STYLE`)
/// 对应 Java：com.alibaba.excel.constant.OrderConstant。
pub const DEFINE_STYLE: i32 = -50_000;

/// Default order. (Java `DEFAULT_ORDER`)
/// 对应 Java：com.alibaba.excel.constant.OrderConstant。
pub const DEFAULT_ORDER: i32 = 0;

/// Sorting of styles written to cells. (Java `FILL_STYLE`)
/// 对应 Java：com.alibaba.excel.constant.OrderConstant。
pub const FILL_STYLE: i32 = 50_000;

/// Java `OrderConstant` 的静态常量门面。
#[derive(Debug, Clone, Copy, Default)]
pub struct OrderConstant;

impl OrderConstant {
    pub const DEFAULT_DEFINE_STYLE: i32 = DEFAULT_DEFINE_STYLE;
    pub const ANNOTATION_DEFINE_STYLE: i32 = ANNOTATION_DEFINE_STYLE;
    pub const DEFINE_STYLE: i32 = DEFINE_STYLE;
    pub const DEFAULT_ORDER: i32 = DEFAULT_ORDER;
    pub const FILL_STYLE: i32 = FILL_STYLE;
}
