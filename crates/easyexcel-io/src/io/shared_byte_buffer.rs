//! 可克隆的线程安全字节输出缓冲区。

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// 允许多个写入器句柄共享同一字节缓冲区。
#[derive(Clone, Default)]
pub struct SharedByteBuffer {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl SharedByteBuffer {
    /// 取出当前全部字节，并清空共享缓冲区。
    ///
    /// # Errors
    ///
    /// 缓冲区互斥锁被污染时返回 I/O 错误。
    pub fn take(&self) -> io::Result<Vec<u8>> {
        self.bytes
            .lock()
            .map_err(|_| io::Error::other("shared byte buffer lock poisoned"))
            .map(|mut bytes| std::mem::take(&mut *bytes))
    }
}

impl Write for SharedByteBuffer {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.bytes
            .lock()
            .map_err(|_| io::Error::other("shared byte buffer lock poisoned"))?
            .extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
