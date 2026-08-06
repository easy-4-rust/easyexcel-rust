use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::CancellationToken;

use super::{ExcelWebError, ExcelWebPolicy};

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// 一次 Web Excel 操作共享的请求上下文。
///
/// 上下文将请求标识、统一策略和取消令牌贯穿上传、解析、生成与响应阶段。
#[derive(Debug, Clone)]
pub struct WebExecutionContext {
    request_id: Arc<str>,
    policy: ExcelWebPolicy,
    cancellation: CancellationToken,
    worker_slots: Arc<Semaphore>,
}

impl WebExecutionContext {
    /// 使用调用方提供的请求标识创建上下文。
    #[must_use]
    pub fn new(request_id: impl Into<Arc<str>>, policy: ExcelWebPolicy) -> Self {
        let worker_slots = Arc::new(Semaphore::new(policy.max_concurrent_tasks()));
        Self::with_worker_slots(request_id, policy, worker_slots)
    }

    pub(crate) fn with_worker_slots(
        request_id: impl Into<Arc<str>>,
        policy: ExcelWebPolicy,
        worker_slots: Arc<Semaphore>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            policy,
            cancellation: CancellationToken::new(),
            worker_slots,
        }
    }

    /// 创建具有进程内唯一请求标识的上下文。
    #[must_use]
    pub fn generated(policy: ExcelWebPolicy) -> Self {
        let worker_slots = Arc::new(Semaphore::new(policy.max_concurrent_tasks()));
        Self::generated_with_worker_slots(policy, worker_slots)
    }

    pub(crate) fn generated_with_worker_slots(
        policy: ExcelWebPolicy,
        worker_slots: Arc<Semaphore>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        Self::with_worker_slots(
            format!("excel-{timestamp:x}-{sequence:x}"),
            policy,
            worker_slots,
        )
    }

    /// 返回请求标识。
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// 返回统一执行策略。
    #[must_use]
    pub const fn policy(&self) -> &ExcelWebPolicy {
        &self.policy
    }

    /// 返回可供框架断连监听器复用的取消令牌。
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    /// 请求协作式取消当前操作。
    pub fn cancel(&self) {
        self.cancellation.cancel();
    }

    /// 判断当前操作是否已经取消。
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// 在阶段边界检查取消状态。
    ///
    /// # Errors
    ///
    /// 已收到取消信号时返回稳定的取消错误。
    pub fn checkpoint(&self) -> Result<(), ExcelWebError> {
        if self.is_cancelled() {
            Err(ExcelWebError::cancelled())
        } else {
            Ok(())
        }
    }

    pub(crate) async fn acquire_worker_permit(
        &self,
    ) -> Result<OwnedSemaphorePermit, ExcelWebError> {
        self.checkpoint()?;
        let cancellation = self.cancellation_token();
        let acquire = Arc::clone(&self.worker_slots).acquire_owned();
        tokio::select! {
            () = cancellation.cancelled() => Err(ExcelWebError::cancelled()),
            result = tokio::time::timeout(self.policy.processing_timeout(), acquire) => {
                match result {
                    Ok(Ok(permit)) => Ok(permit),
                    Ok(Err(error)) => Err(ExcelWebError::Worker {
                        message: error.to_string(),
                    }),
                    Err(_) => Err(ExcelWebError::processing_timeout()),
                }
            }
        }
    }
}
