//! 写处理器请求的后端中立工作簿修改。

use crate::CellValue;

/// 写生命周期回调提交、由具体格式后端在保存前执行的修改。
///
/// 对应 Java：POI `Workbook` / `Sheet` / `Cell` 句柄上的延迟修改。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WriteMutation {
    /// 设置指定工作表中的单元格值。
    SetCell {
        sheet_name: String,
        row_index: u32,
        column_index: u16,
        value: CellValue,
    },
    /// 使用密码保护指定工作表。
    ProtectSheet {
        sheet_name: String,
        password: String,
    },
}
