use std::path::{Path, PathBuf};
use std::time::Duration;

use easyexcel::io::ResourceLimits;

/// `EasyExcel` Web 请求统一资源与执行策略。
///
/// 框架适配层必须共享同一策略，避免不同框架对上传大小、行数和缓冲区
/// 采用不一致的默认值。
#[derive(Debug, Clone)]
pub struct ExcelWebPolicy {
    resource_limits: ResourceLimits,
    upload_timeout: Duration,
    processing_timeout: Duration,
    max_concurrent_tasks: usize,
    row_channel_capacity: usize,
    io_chunk_size: usize,
    temp_directory: Option<PathBuf>,
}

impl ExcelWebPolicy {
    /// 创建 Web 执行策略。
    #[must_use]
    pub fn new(resource_limits: ResourceLimits) -> Self {
        Self {
            resource_limits,
            ..Self::default()
        }
    }

    /// 设置统一基础资源限制。
    #[must_use]
    pub const fn with_resource_limits(mut self, resource_limits: ResourceLimits) -> Self {
        self.resource_limits = resource_limits;
        self
    }

    /// 设置接收请求体的最大持续时间。
    #[must_use]
    pub const fn with_upload_timeout(mut self, upload_timeout: Duration) -> Self {
        self.upload_timeout = upload_timeout;
        self
    }

    /// 设置 Excel 解析或生成的最大持续时间。
    #[must_use]
    pub const fn with_processing_timeout(mut self, processing_timeout: Duration) -> Self {
        self.processing_timeout = processing_timeout;
        self
    }

    /// 设置同一 [`super::ExcelWebRuntime`] 允许并行执行的解析和生成任务数。
    ///
    /// `max_concurrent_tasks` 为零时按最小并发数一处理。
    #[must_use]
    pub fn with_max_concurrent_tasks(mut self, max_concurrent_tasks: usize) -> Self {
        self.max_concurrent_tasks = max_concurrent_tasks.max(1);
        self
    }

    /// 设置解析线程与异步消费者之间的有界行通道容量。
    ///
    /// `capacity` 为零时按最小容量一处理，确保通道始终具备背压语义。
    #[must_use]
    pub fn with_row_channel_capacity(mut self, capacity: usize) -> Self {
        self.row_channel_capacity = capacity.max(1);
        self
    }

    /// 设置下载读取时建议使用的 I/O 分块大小。
    ///
    /// `chunk_size` 为零时按最小值一字节处理。
    #[must_use]
    pub fn with_io_chunk_size(mut self, chunk_size: usize) -> Self {
        self.io_chunk_size = chunk_size.max(1);
        self
    }

    /// 设置受控临时目录；未设置时使用操作系统临时目录。
    #[must_use]
    pub fn with_temp_directory(mut self, temp_directory: impl Into<PathBuf>) -> Self {
        self.temp_directory = Some(temp_directory.into());
        self
    }

    /// 返回统一基础资源限制。
    #[must_use]
    pub const fn resource_limits(&self) -> ResourceLimits {
        self.resource_limits
    }

    /// 返回上传超时。
    #[must_use]
    pub const fn upload_timeout(&self) -> Duration {
        self.upload_timeout
    }

    /// 返回处理超时。
    #[must_use]
    pub const fn processing_timeout(&self) -> Duration {
        self.processing_timeout
    }

    /// 返回共享 runtime 的最大并发解析和生成任务数。
    #[must_use]
    pub const fn max_concurrent_tasks(&self) -> usize {
        self.max_concurrent_tasks
    }

    /// 返回有界行通道容量。
    #[must_use]
    pub const fn row_channel_capacity(&self) -> usize {
        self.row_channel_capacity
    }

    /// 返回建议 I/O 分块大小。
    #[must_use]
    pub const fn io_chunk_size(&self) -> usize {
        self.io_chunk_size
    }

    /// 返回受控临时目录。
    #[must_use]
    pub fn temp_directory(&self) -> Option<&Path> {
        self.temp_directory.as_deref()
    }
}

impl Default for ExcelWebPolicy {
    fn default() -> Self {
        Self {
            resource_limits: ResourceLimits::default(),
            upload_timeout: Duration::from_secs(30),
            processing_timeout: Duration::from_secs(300),
            max_concurrent_tasks: std::thread::available_parallelism().map_or(4, usize::from),
            row_channel_capacity: 32,
            io_chunk_size: 64 * 1024,
            temp_directory: None,
        }
    }
}
