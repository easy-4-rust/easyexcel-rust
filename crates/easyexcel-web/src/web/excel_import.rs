use std::fmt::Display;
use std::marker::PhantomData;

use bytes::Bytes;
use easyexcel::ExcelRow;
use easyexcel::io::Format;
use futures_util::{Stream, StreamExt, stream};
use tokio::io::AsyncWriteExt;

use super::temp_artifact::TempArtifact;
use super::{ExcelRows, ExcelWebError, WebExecutionContext};

/// 已安全接收并等待解析的 Excel 请求。
///
/// 上传内容按分块写入受控临时文件，不会把完整工作簿保存在内存中。调用
/// [`Self::rows`] 后，底层同步解析器通过有界通道向异步消费者施加背压。
#[derive(Debug)]
pub struct ExcelImport<T> {
    artifact: TempArtifact,
    file_name: Option<String>,
    format: Format,
    received_bytes: u64,
    context: WebExecutionContext,
    marker: PhantomData<fn() -> T>,
}

impl<T> ExcelImport<T>
where
    T: ExcelRow + Send + 'static,
{
    /// 从框架提供的请求体分块流接收工作簿。
    ///
    /// `extension` 可以带或不带前导点。适配层应传入可信的文件名扩展名，
    /// 并把框架断连信号连接到 [`WebExecutionContext::cancel`]。
    ///
    /// # Errors
    ///
    /// 格式不支持、请求体传输失败、超过字节限制、超时、取消或临时 I/O
    /// 失败时返回统一错误。
    pub async fn receive<S, E>(
        stream: S,
        extension: &str,
        file_name: Option<String>,
        context: WebExecutionContext,
    ) -> Result<Self, ExcelWebError>
    where
        S: Stream<Item = Result<Bytes, E>>,
        E: Display,
    {
        context.checkpoint()?;
        let extension = extension
            .trim()
            .trim_start_matches('.')
            .to_ascii_lowercase();
        let format = Format::from_extension(&extension).ok_or_else(|| {
            ExcelWebError::UnsupportedMediaType {
                extension: extension.clone(),
            }
        })?;
        let suffix = format!(".{extension}");
        let artifact = TempArtifact::create(&suffix, context.policy())?;
        let request_id = context.request_id().to_string();
        let upload_timeout = context.policy().upload_timeout();
        let cancellation = context.cancellation_token();

        let receive = receive_to_artifact(stream, &artifact, &context);
        let received_bytes = tokio::select! {
            () = cancellation.cancelled() => return Err(ExcelWebError::cancelled()),
            result = tokio::time::timeout(upload_timeout, receive) => {
                if let Ok(result) = result {
                    result?
                } else {
                    context.cancel();
                    return Err(ExcelWebError::processing_timeout());
                }
            }
        };

        tracing::debug!(
            request_id,
            ?format,
            received_bytes,
            "Excel 请求体已安全接收"
        );
        Ok(Self {
            artifact,
            file_name,
            format,
            received_bytes,
            context,
            marker: PhantomData,
        })
    }

    /// 从单个字节缓冲区创建请求，主要供无需请求体流的框架桥接和测试使用。
    ///
    /// # Errors
    ///
    /// 与 [`Self::receive`] 相同。
    pub async fn from_bytes(
        bytes: Bytes,
        extension: &str,
        file_name: Option<String>,
        context: WebExecutionContext,
    ) -> Result<Self, ExcelWebError> {
        let chunks = stream::iter([Ok::<Bytes, std::convert::Infallible>(bytes)]);
        Self::receive(chunks, extension, file_name, context).await
    }

    /// 启动 Event Mode 解析并返回具有背压的类型化行流。
    #[must_use]
    pub fn rows(self) -> ExcelRows<T> {
        ExcelRows::spawn(self)
    }

    /// 返回调用方提供的原始文件名。
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    /// 返回已识别的表格格式。
    #[must_use]
    pub const fn format(&self) -> Format {
        self.format
    }

    /// 返回已落盘的请求体字节数。
    #[must_use]
    pub const fn received_bytes(&self) -> u64 {
        self.received_bytes
    }

    /// 返回请求执行上下文。
    #[must_use]
    pub const fn context(&self) -> &WebExecutionContext {
        &self.context
    }

    pub(crate) fn into_parts(self) -> (TempArtifact, WebExecutionContext) {
        (self.artifact, self.context)
    }
}

async fn receive_to_artifact<S, E>(
    stream: S,
    artifact: &TempArtifact,
    context: &WebExecutionContext,
) -> Result<u64, ExcelWebError>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Display,
{
    futures_util::pin_mut!(stream);
    let mut output = tokio::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(artifact.path())
        .await?;
    let limit = context.policy().resource_limits().max_file_bytes();
    let mut received = 0_u64;

    while let Some(chunk) = stream.next().await {
        context.checkpoint()?;
        let chunk = chunk.map_err(|error| ExcelWebError::Transport {
            message: error.to_string(),
        })?;
        let chunk_length = u64::try_from(chunk.len()).unwrap_or(u64::MAX);
        let next_size = received.saturating_add(chunk_length);
        if next_size > limit {
            return Err(ExcelWebError::FileTooLarge {
                actual: next_size,
                limit,
            });
        }
        output.write_all(&chunk).await?;
        received = next_size;
    }
    output.flush().await?;
    Ok(received)
}
