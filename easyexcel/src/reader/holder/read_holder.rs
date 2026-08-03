//! 对应 Java：`com.alibaba.excel.read.metadata.holder.ReadHolder` (interface).

use crate::core::AnalysisContext;

/// 对应 Java：`ReadHolder extends ConfigurationHolder`.
pub trait ReadHolder {
    /// Returns the analysis context. (Java `getAnalysisContext()`)
    fn analysis_context(&self) -> &AnalysisContext;
}
