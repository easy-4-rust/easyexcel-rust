use std::marker::PhantomData;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use easyexcel::io::Format;
use easyexcel::{EasyExcel, ExcelRow};
use tokio::io::{AsyncRead, ReadBuf};

use super::temp_artifact::TempArtifact;
use super::{ExcelWebError, WebExecutionContext};

/// 已生成并可由 Web 框架流式响应的 Excel 文件。
///
/// 生成过程运行于受控阻塞线程，并默认启用 `EasyExcel` 恒定内存写出。响应阶段
/// 直接从临时文件异步读取，不会把完整文件复制到 `Vec<u8>`。
#[derive(Debug)]
pub struct ExcelExport<T> {
    _artifact: TempArtifact,
    file: tokio::fs::File,
    file_name: String,
    format: Format,
    content_length: u64,
    context: WebExecutionContext,
    marker: PhantomData<fn() -> T>,
}

impl<T> ExcelExport<T>
where
    T: ExcelRow + Send + 'static,
{
    /// 以恒定内存模式生成可流式下载的工作簿。
    ///
    /// `rows` 的迭代器必须可在线程间移动，但数据不要求预先收集到内存。
    ///
    /// # Errors
    ///
    /// 取消、超时、超过行数或文件大小限制、数据转换和 I/O 失败时返回统一错误。
    pub async fn prepare<I>(
        rows: I,
        format: Format,
        file_name: impl Into<String>,
        sheet_name: impl Into<String>,
        context: WebExecutionContext,
    ) -> Result<Self, ExcelWebError>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        context.checkpoint()?;
        let worker_permit = context.acquire_worker_permit().await?;
        let extension = format_extension(format)?;
        let artifact = TempArtifact::create(extension, context.policy())?;
        let path = artifact.path_buf();
        let iterator = rows.into_iter();
        let worker_context = context.clone();
        let sheet_name = sheet_name.into();
        let processing_timeout = context.policy().processing_timeout();
        let request_id = context.request_id().to_string();

        let worker = tokio::task::spawn_blocking(move || {
            let _worker_permit = worker_permit;
            let mut controlled = ControlledRows::new(iterator, worker_context.clone());
            let write_result = EasyExcel::write::<T>(&path)
                .sheet(sheet_name)
                .constant_memory(true)
                .do_write_iter(&mut controlled);

            if let Some(error) = controlled.take_error() {
                return Err(error);
            }
            worker_context.checkpoint()?;
            write_result.map_err(ExcelWebError::from)?;
            let content_length = std::fs::metadata(&path)?.len();
            let limit = worker_context.policy().resource_limits().max_file_bytes();
            if content_length > limit {
                return Err(ExcelWebError::FileTooLarge {
                    actual: content_length,
                    limit,
                });
            }
            Ok((artifact, content_length))
        });

        let (artifact, content_length) =
            match tokio::time::timeout(processing_timeout, worker).await {
                Ok(Ok(result)) => result?,
                Ok(Err(error)) => {
                    return Err(ExcelWebError::Worker {
                        message: error.to_string(),
                    });
                }
                Err(_) => {
                    context.cancel();
                    return Err(ExcelWebError::processing_timeout());
                }
            };

        let file = tokio::fs::File::open(artifact.path()).await?;
        tracing::debug!(request_id, ?format, content_length, "Excel 响应文件已生成");
        let file_name = file_name.into();
        let file_name = sanitize_file_name(&file_name, extension);
        Ok(Self {
            _artifact: artifact,
            file,
            file_name,
            format,
            content_length,
            context,
            marker: PhantomData,
        })
    }

    /// 返回安全的响应文件名。
    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// 返回文件格式。
    #[must_use]
    pub const fn format(&self) -> Format {
        self.format
    }

    /// 返回响应体精确字节数。
    #[must_use]
    pub const fn content_length(&self) -> u64 {
        self.content_length
    }

    /// 返回标准响应媒体类型。
    #[must_use]
    pub fn content_type(&self) -> &'static str {
        match self.format {
            Format::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Format::Xls => "application/vnd.ms-excel",
            Format::Csv => "text/csv; charset=utf-8",
            _ => "application/octet-stream",
        }
    }

    /// 返回请求执行上下文。
    #[must_use]
    pub const fn context(&self) -> &WebExecutionContext {
        &self.context
    }

    /// 主动取消尚未完成的响应读取。
    pub fn cancel(&self) {
        self.context.cancel();
    }

    /// 返回策略建议的响应读取分块大小。
    #[must_use]
    pub const fn io_chunk_size(&self) -> usize {
        self.context.policy().io_chunk_size()
    }
}

impl<T> AsyncRead for ExcelExport<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.context.is_cancelled() {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Excel 响应读取已取消",
            )));
        }
        Pin::new(&mut self.file).poll_read(context, buffer)
    }
}

struct ControlledRows<I> {
    inner: I,
    context: WebExecutionContext,
    emitted: u64,
    error: Option<ExcelWebError>,
}

impl<I> ControlledRows<I> {
    fn new(inner: I, context: WebExecutionContext) -> Self {
        Self {
            inner,
            context,
            emitted: 0,
            error: None,
        }
    }

    fn take_error(&mut self) -> Option<ExcelWebError> {
        self.error.take()
    }
}

impl<I> Iterator for ControlledRows<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.context.is_cancelled() {
            self.error = Some(ExcelWebError::cancelled());
            return None;
        }
        let item = self.inner.next()?;
        let limit = self.context.policy().resource_limits().max_rows();
        if self.emitted >= limit {
            self.error = Some(ExcelWebError::RowLimitExceeded { limit });
            return None;
        }
        self.emitted += 1;
        Some(item)
    }
}

fn format_extension(format: Format) -> Result<&'static str, ExcelWebError> {
    match format {
        Format::Xlsx => Ok(".xlsx"),
        Format::Xls => Ok(".xls"),
        Format::Csv => Ok(".csv"),
        _ => Err(ExcelWebError::UnsupportedMediaType {
            extension: "unknown".to_string(),
        }),
    }
}

fn sanitize_file_name(file_name: &str, extension: &str) -> String {
    let component = Path::new(file_name)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("download")
        .replace(['\r', '\n', '"'], "_");
    if component.to_ascii_lowercase().ends_with(extension) {
        component
    } else {
        format!("{component}{extension}")
    }
}
