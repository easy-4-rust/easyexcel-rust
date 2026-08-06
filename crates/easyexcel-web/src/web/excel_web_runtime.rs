use std::sync::Arc;

use tokio::sync::Semaphore;

use super::{ExcelWebPolicy, WebExecutionContext};

/// Web 应用级共享 `EasyExcel` 执行环境。
///
/// 应用应创建一个实例并放入框架状态。由它创建的所有请求上下文共享同一个
/// 并发许可池，从而限制阻塞式 Excel 解析和生成任务的总数。
#[derive(Debug, Clone)]
pub struct ExcelWebRuntime {
    policy: ExcelWebPolicy,
    worker_slots: Arc<Semaphore>,
}

impl ExcelWebRuntime {
    /// 使用统一策略创建应用级执行环境。
    #[must_use]
    pub fn new(policy: ExcelWebPolicy) -> Self {
        let worker_slots = Arc::new(Semaphore::new(policy.max_concurrent_tasks()));
        Self {
            policy,
            worker_slots,
        }
    }

    /// 为一次请求创建带调用方请求标识的上下文。
    #[must_use]
    pub fn context(&self, request_id: impl Into<Arc<str>>) -> WebExecutionContext {
        WebExecutionContext::with_worker_slots(
            request_id,
            self.policy.clone(),
            Arc::clone(&self.worker_slots),
        )
    }

    /// 为一次请求创建自动请求标识的上下文。
    #[must_use]
    pub fn generated_context(&self) -> WebExecutionContext {
        WebExecutionContext::generated_with_worker_slots(
            self.policy.clone(),
            Arc::clone(&self.worker_slots),
        )
    }

    /// 返回应用统一策略。
    #[must_use]
    pub const fn policy(&self) -> &ExcelWebPolicy {
        &self.policy
    }

    /// 返回当前可立即启动的解析和生成任务数。
    #[must_use]
    pub fn available_permits(&self) -> usize {
        self.worker_slots.available_permits()
    }
}
