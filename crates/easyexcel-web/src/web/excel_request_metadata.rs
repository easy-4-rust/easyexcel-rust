use std::path::Path;

use super::ExcelWebError;

/// 从不同 Web 框架请求头归一化得到的 Excel 上传元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelRequestMetadata {
    file_name: Option<String>,
    extension: String,
    request_id: Option<String>,
}

impl ExcelRequestMetadata {
    /// 从显式文件名、`Content-Disposition`、`Content-Type` 和请求标识解析元数据。
    ///
    /// 文件名优先于媒体类型；无法确定受支持格式时返回统一格式错误。
    ///
    /// # Errors
    ///
    /// 请求没有受支持的 XLSX、XLS 或 CSV 格式信息时返回错误。
    pub fn resolve(
        explicit_file_name: Option<&str>,
        content_disposition: Option<&str>,
        content_type: Option<&str>,
        request_id: Option<&str>,
    ) -> Result<Self, ExcelWebError> {
        let file_name = explicit_file_name
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| content_disposition.and_then(file_name_from_content_disposition));
        let extension = file_name
            .as_deref()
            .and_then(extension_from_file_name)
            .or_else(|| content_type.and_then(extension_from_content_type))
            .ok_or_else(|| ExcelWebError::UnsupportedMediaType {
                extension: "unknown".to_string(),
            })?;
        Ok(Self {
            file_name,
            extension: extension.to_string(),
            request_id: request_id
                .filter(|value| !value.trim().is_empty())
                .map(ToOwned::to_owned),
        })
    }

    /// 返回原始安全文件名（如果请求提供）。
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    /// 返回不带点号的小写扩展名。
    #[must_use]
    pub fn extension(&self) -> &str {
        &self.extension
    }

    /// 返回调用方请求标识。
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }
}

fn extension_from_file_name(file_name: &str) -> Option<&'static str> {
    match Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())?
        .to_ascii_lowercase()
        .as_str()
    {
        "xlsx" | "xlsm" => Some("xlsx"),
        "xls" => Some("xls"),
        "csv" | "tsv" | "txt" => Some("csv"),
        _ => None,
    }
}

fn extension_from_content_type(content_type: &str) -> Option<&'static str> {
    let media_type = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match media_type.as_str() {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        | "application/vnd.ms-excel.sheet.macroenabled.12" => Some("xlsx"),
        "application/vnd.ms-excel" => Some("xls"),
        "text/csv" | "text/tab-separated-values" | "text/plain" => Some("csv"),
        _ => None,
    }
}

fn file_name_from_content_disposition(value: &str) -> Option<String> {
    value.split(';').skip(1).find_map(|part| {
        let (name, raw_value) = part.trim().split_once('=')?;
        if !name.trim().eq_ignore_ascii_case("filename") {
            return None;
        }
        let file_name = raw_value.trim().trim_matches('"');
        (!file_name.is_empty()).then(|| file_name.to_string())
    })
}
