use super::{Result, RowSink};

/// 可将工作表逐行推送给消费者的数据源。
pub trait RowSource {
    /// 将数据源中的行写入指定消费者。
    fn stream(&mut self, sink: &mut dyn RowSink) -> Result<()>;
}
