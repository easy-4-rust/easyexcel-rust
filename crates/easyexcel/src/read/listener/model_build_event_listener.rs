//! 对应 Java：`com.alibaba.excel.read.listener.ModelBuildEventListener`。

use crate::core::{ConverterRegistry, ExcelRow, Result, RowData};
use crate::metadata::DynamicRow;

/// 将物理行转换成用户模型或无模型行。
///
/// Java 版本通过反射和 `BeanMap` 修改 `ReadRowHolder`；Rust 把相同转换
/// 结果作为返回值交给读取管线，强类型路径由 `ExcelRow`/derive 生成，
/// 无模型路径保留 `ReadDefaultReturn`、稀疏列和表头尾部空列语义。
#[derive(Debug, Clone, Default)]
pub struct ModelBuildEventListener;

impl ModelBuildEventListener {
    /// 创建模型构建监听器。对应 Java 公共无参构造器。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 构建调用方声明的用户模型。
    ///
    /// 对应 Java `invoke` 的 `HeadKindEnum.CLASS` 分支以及私有
    /// `buildUserModel`；字段定位、转换器优先级和错误坐标由 `ExcelRow`
    /// 与 `ConverterRegistry` 保持。
    pub fn build_user_model<T: ExcelRow>(
        &self,
        row: &RowData,
        converters: &ConverterRegistry,
    ) -> Result<T> {
        T::from_row_with_converters(row, converters)
    }

    /// 构建 `Map<Integer, Object>` 等价的无模型行。
    ///
    /// `DynamicRow` 根据行上的 `ReadDefaultReturn` 产生字符串、实际值或
    /// `ReadCellData`，并为缺失的中间列和表头尾部列保留显式 null。
    pub fn build_no_model(&self, row: &RowData) -> Result<DynamicRow> {
        DynamicRow::from_row(row)
    }

    /// 执行强类型行转换。对应 Java `ReadListener#invoke` 中设置
    /// `currentRowAnalysisResult` 前的模型构建步骤。
    pub fn invoke<T: ExcelRow>(
        &mut self,
        row: &RowData,
        converters: &ConverterRegistry,
    ) -> Result<T> {
        self.build_user_model(row, converters)
    }

    /// Java `doAfterAllAnalysed` 的空实现。
    pub const fn do_after_all_analysed(&mut self) {}
}
