use std::path::{Path, PathBuf};

use tempfile::TempPath;

use super::{ExcelWebError, ExcelWebPolicy};

/// 由一次 Web 请求独占的临时文件，离开作用域后自动删除。
#[derive(Debug)]
pub(crate) struct TempArtifact {
    path: TempPath,
}

impl TempArtifact {
    /// 在策略指定目录中创建带格式后缀的临时文件。
    pub(crate) fn create(suffix: &str, policy: &ExcelWebPolicy) -> Result<Self, ExcelWebError> {
        let mut builder = tempfile::Builder::new();
        builder.prefix("easyexcel-web-").suffix(suffix);
        let file = if let Some(directory) = policy.temp_directory() {
            std::fs::create_dir_all(directory)?;
            builder.tempfile_in(directory)?
        } else {
            builder.tempfile()?
        };
        Ok(Self {
            path: file.into_temp_path(),
        })
    }

    /// 返回临时文件路径。
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// 返回可移动到阻塞任务中的路径副本。
    pub(crate) fn path_buf(&self) -> PathBuf {
        self.path.to_path_buf()
    }
}
