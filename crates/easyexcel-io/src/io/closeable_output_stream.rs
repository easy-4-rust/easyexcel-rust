//! 可克隆、可显式关闭的共享输出流。

use std::io::Write;
use std::sync::{Arc, Mutex};

/// 多个句柄共享同一个底层 writer 的可关闭输出流。
///
/// 任一句柄关闭流后，所有克隆都会观察到关闭状态；关闭前会刷新底层
/// writer。该类型不依赖 EasyExcel 门面，可供 CLI、HTTP 适配器和模板写入器复用。
pub struct CloseableOutputStream<W> {
    inner: Arc<Mutex<Option<W>>>,
}

impl<W> CloseableOutputStream<W> {
    /// 包装一个拥有的字节写入器。
    #[must_use]
    pub fn new(writer: W) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(writer))),
        }
    }

    /// 关闭共享流，在释放底层 writer 前刷新它。
    ///
    /// # Errors
    ///
    /// 锁被污染或最终刷新失败时返回 I/O 错误。
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

    /// 返回共享流是否已经关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.lock().map_or(true, |writer| writer.is_none())
    }

    /// 对仍处于打开状态的底层 writer 运行只读回调。
    ///
    /// 流已关闭或锁被污染时返回 `None`。
    pub fn with_inner<R>(&self, inspect: impl FnOnce(&W) -> R) -> Option<R> {
        self.inner
            .lock()
            .ok()
            .and_then(|writer| writer.as_ref().map(inspect))
    }

    /// 当当前实例是唯一句柄且流仍打开时，回收底层 writer。
    ///
    /// # Errors
    ///
    /// 仍有其他克隆、流已关闭或锁被污染时返回原共享流句柄。
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

impl<W> Clone for CloseableOutputStream<W> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<W> Write for CloseableOutputStream<W>
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
