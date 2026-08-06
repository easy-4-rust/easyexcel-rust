use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

use easyexcel::{AnalysisContext, EasyExcel, ExcelError, ExcelRow, ReadListener};
use futures_util::Stream;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::{ExcelImport, ExcelWebError, WebExecutionContext};

const EXECUTION_STOPPED: &str = "easyexcel-web execution stopped";

/// 类型化、具有背压和取消语义的 Excel 行流。
///
/// 解析器只会领先消费者至 [`super::ExcelWebPolicy::row_channel_capacity`]
/// 指定的行数。丢弃该对象会取消后台解析并触发临时文件清理。
#[derive(Debug)]
pub struct ExcelRows<T> {
    receiver: mpsc::Receiver<Result<T, ExcelWebError>>,
    cancellation: CancellationToken,
}

impl<T> ExcelRows<T>
where
    T: ExcelRow + Send + 'static,
{
    pub(crate) fn spawn(import: ExcelImport<T>) -> Self {
        let (artifact, context) = import.into_parts();
        let capacity = context.policy().row_channel_capacity();
        let timeout = context.policy().processing_timeout();
        let cancellation = context.cancellation_token();
        let (sender, receiver) = mpsc::channel(capacity);
        let terminal_sent = Arc::new(AtomicBool::new(false));
        let task_context = context;
        let task_sender = sender;
        tokio::spawn(async move {
            let worker_permit = match task_context.acquire_worker_permit().await {
                Ok(permit) => permit,
                Err(error) => {
                    send_terminal_async(&terminal_sent, &task_sender, error).await;
                    return;
                }
            };
            let worker_sender = task_sender.clone();
            let worker_terminal_sender = task_sender.clone();
            let worker_terminal = Arc::clone(&terminal_sent);
            let worker_context = task_context.clone();
            let completion_context = task_context.clone();
            let worker = tokio::task::spawn_blocking(move || {
                let _worker_permit = worker_permit;
                let path = artifact.path_buf();
                let listener = ChannelReadListener::new(
                    worker_sender,
                    worker_context,
                    Arc::clone(&worker_terminal),
                );
                let result = EasyExcel::read::<T, _>(path, listener).do_read();
                if completion_context.is_cancelled() {
                    send_terminal(
                        &worker_terminal,
                        &worker_terminal_sender,
                        ExcelWebError::cancelled(),
                    );
                    return;
                }
                if let Err(error) = result {
                    if error_is_execution_stop(&error) {
                        return;
                    }
                    send_terminal(
                        &worker_terminal,
                        &worker_terminal_sender,
                        ExcelWebError::from(error),
                    );
                }
            });

            match tokio::time::timeout(timeout, worker).await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    send_terminal_async(
                        &terminal_sent,
                        &task_sender,
                        ExcelWebError::Worker {
                            message: error.to_string(),
                        },
                    )
                    .await;
                }
                Err(_) => {
                    if !terminal_sent.swap(true, Ordering::AcqRel) {
                        task_context.cancel();
                        let _ = task_sender
                            .send(Err(ExcelWebError::processing_timeout()))
                            .await;
                    }
                }
            }
        });

        Self {
            receiver,
            cancellation,
        }
    }

    /// 异步获取下一行；返回 `None` 表示解析完成。
    pub async fn next_row(&mut self) -> Option<Result<T, ExcelWebError>> {
        self.receiver.recv().await
    }

    /// 主动取消后台解析。
    pub fn cancel(&self) {
        self.cancellation.cancel();
    }
}

impl<T> Stream for ExcelRows<T> {
    type Item = Result<T, ExcelWebError>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(context)
    }
}

impl<T> Drop for ExcelRows<T> {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

struct ChannelReadListener<T> {
    sender: mpsc::Sender<Result<T, ExcelWebError>>,
    context: WebExecutionContext,
    terminal_sent: Arc<AtomicBool>,
    rows: u64,
}

impl<T> ChannelReadListener<T> {
    fn new(
        sender: mpsc::Sender<Result<T, ExcelWebError>>,
        context: WebExecutionContext,
        terminal_sent: Arc<AtomicBool>,
    ) -> Self {
        Self {
            sender,
            context,
            terminal_sent,
            rows: 0,
        }
    }

    fn stop_error() -> ExcelError {
        ExcelError::Unsupported(EXECUTION_STOPPED.to_string())
    }
}

impl<T> ReadListener<T> for ChannelReadListener<T>
where
    T: Send,
{
    fn invoke(&mut self, data: T, _context: &AnalysisContext) -> easyexcel::Result<()> {
        if self.context.is_cancelled() {
            send_terminal(
                &self.terminal_sent,
                &self.sender,
                ExcelWebError::cancelled(),
            );
            return Err(Self::stop_error());
        }

        let limit = self.context.policy().resource_limits().max_rows();
        if self.rows >= limit {
            send_terminal(
                &self.terminal_sent,
                &self.sender,
                ExcelWebError::RowLimitExceeded { limit },
            );
            return Err(Self::stop_error());
        }
        self.rows += 1;
        if self.sender.blocking_send(Ok(data)).is_err() {
            self.context.cancel();
            return Err(Self::stop_error());
        }
        Ok(())
    }

    fn has_next(&mut self, _context: &AnalysisContext) -> bool {
        !self.context.is_cancelled()
    }
}

fn error_is_execution_stop(error: &ExcelError) -> bool {
    matches!(error, ExcelError::Unsupported(message) if message == EXECUTION_STOPPED)
}

fn send_terminal<T>(
    terminal_sent: &AtomicBool,
    sender: &mpsc::Sender<Result<T, ExcelWebError>>,
    error: ExcelWebError,
) {
    if !terminal_sent.swap(true, Ordering::AcqRel) {
        let _ = sender.blocking_send(Err(error));
    }
}

async fn send_terminal_async<T>(
    terminal_sent: &AtomicBool,
    sender: &mpsc::Sender<Result<T, ExcelWebError>>,
    error: ExcelWebError,
) {
    if !terminal_sent.swap(true, Ordering::AcqRel) {
        let _ = sender.send(Err(error)).await;
    }
}
