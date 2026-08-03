//! Excel 输出流类型。
//!
//! 对应 Java：`java.io.OutputStream` 的 Excel 包装。
//! 原文件：easyexcel-core 内部 `OutputStream` 包装。

use std::io::Write;
use std::sync::{Arc, Mutex};

/// 输出流克隆共享底层 writer。关闭任一克隆会丢弃底层 writer。
///
/// 对应 Java：`java.io.OutputStream`。
/// 通过 `ExcelOutputStream::new(writer)` 包装，支持克隆共享。
pub struct ExcelOutputStream<W> {
    pub(crate) inner: Arc<Mutex<Option<W>>>,
}

impl<W> ExcelOutputStream<W> {
    /// 包装一个拥有的字节写入器。
    #[must_use]
    pub fn new(writer: W) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(writer))),
        }
    }

    /// 关闭共享流，在释放所有权前刷新它。
    ///
    /// # Errors
    ///
    /// 当锁被污染或最终刷新失败时返回错误。
    pub fn close(&self) -> std::io::Result<()>
    where
        W: Write,
    {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("output stream lock poisoned"))?;
        if let Some(mut writer) = guard.take() {
            writer.flush()?;
        }
        Ok(())
    }

    /// 返回流是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.lock().map_or(true, |writer| writer.is_none())
    }

    /// 对底层 writer 运行只读回调。
    ///
    /// 流关闭后或锁被污染时返回 `None`。
    pub fn with_inner<R>(&self, inspect: impl FnOnce(&W) -> R) -> Option<R> {
        self.inner
            .lock()
            .ok()
            .and_then(|writer| writer.as_ref().map(inspect))
    }

    /// 当这是唯一句柄且流打开时，回收底层 writer。
    ///
    /// # Errors
    ///
    /// 当另一个克隆存在、流已关闭或锁被污染时返回句柄。
    pub fn into_inner(self) -> std::result::Result<W, Self> {
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => match inner.into_inner() {
                Ok(Some(writer)) => Ok(writer),
                Ok(None) | Err(_) => Err(Self {
                    inner: Arc::new(Mutex::new(None)),
                }),
            },
            Err(inner) => Err(Self { inner }),
        }
    }
}

impl<W> Clone for ExcelOutputStream<W> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<W> Write for ExcelOutputStream<W>
where
    W: Write,
{
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.inner
            .lock()
            .map_err(|_| std::io::Error::other("output stream lock poisoned"))?
            .as_mut()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "stream closed"))?
            .write(buffer)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner
            .lock()
            .map_err(|_| std::io::Error::other("output stream lock poisoned"))?
            .as_mut()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "stream closed"))?
            .flush()
    }
}
