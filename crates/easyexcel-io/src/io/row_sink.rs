use super::{Result, StreamCell, StreamInfo};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 流式行消费者。
pub trait RowSink {
    /// 在第一行之前调用一次。
    ///
    /// # Errors
    ///
    /// 消费者初始化失败时返回错误。
    fn begin(&mut self, _info: &StreamInfo) -> Result<()> {
        Ok(())
    }

    /// 按升序接收一个非空行。
    ///
    /// # Errors
    ///
    /// 消费者无法处理该行时返回错误。
    fn row(&mut self, row: u32, cells: &[StreamCell]) -> Result<()>;

    /// 在最后一行之后调用一次。
    ///
    /// # Errors
    ///
    /// 消费者收尾失败时返回错误。
    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}
