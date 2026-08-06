/// 由引擎解析为 BIFF8 调色板 ICV 的中立颜色请求。
/// 对应 Java：`org.apache.poi.hssf.util.HSSFColor`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8Color {
    /// 自动颜色（`ICV_AUTO`）。
    Automatic,
    /// 已有索引调色板颜色。
    Indexed(u8),
    /// 在工作簿自定义调色板中分配的 RGB 颜色。
    Rgb(u32),
}

