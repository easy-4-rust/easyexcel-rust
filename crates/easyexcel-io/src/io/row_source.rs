use super::{Result, RowSink};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 可将工作表逐行推送给消费者的数据源。
pub trait RowSource {
    /// 将数据源中的行写入指定消费者。
    ///
    /// # Errors
    ///
    /// 数据源读取失败或消费者拒绝数据时返回错误。
    fn stream(&mut self, sink: &mut dyn RowSink) -> Result<()>;
}
