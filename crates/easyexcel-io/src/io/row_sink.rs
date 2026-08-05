use super::{Result, StreamCell, StreamInfo};

/// 流式行消费者。
pub trait RowSink {
    /// 在第一行之前调用一次。
    fn begin(&mut self, _info: &StreamInfo) -> Result<()> {
        Ok(())
    }

    /// 按升序接收一个非空行。
    fn row(&mut self, row: u32, cells: &[StreamCell]) -> Result<()>;

    /// 在最后一行之后调用一次。
    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}
