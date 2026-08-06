use easyexcel::ExcelRow;
use easyexcel::io::Format;
use easyexcel_web::{ExcelExport, WebExecutionContext};
use poem::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use poem::http::{HeaderValue, StatusCode};
use poem::{Body, IntoResponse, Response};

use crate::ExcelPoemError;

/// Poem 原生流式 Excel responder。
#[derive(Debug)]
pub struct ExcelResponse<T> {
    export: ExcelExport<T>,
}

impl<T> ExcelResponse<T>
where
    T: ExcelRow + Send + 'static,
{
    /// 在发送响应头之前完成受控文件生成。
    ///
    /// # Errors
    ///
    /// 生成失败、超时、取消或超过资源限制时返回统一错误。
    pub async fn prepare<I>(
        rows: I,
        format: Format,
        file_name: impl Into<String>,
        sheet_name: impl Into<String>,
        context: WebExecutionContext,
    ) -> Result<Self, ExcelPoemError>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        let request_id = context.request_id().to_string();
        let export = ExcelExport::prepare(rows, format, file_name, sheet_name, context)
            .await
            .map_err(|error| ExcelPoemError::new(error, request_id))?;
        Ok(Self { export })
    }
}

impl<T> IntoResponse for ExcelResponse<T>
where
    T: ExcelRow + Send + 'static,
{
    fn into_response(self) -> Response {
        let content_type = self.export.content_type();
        let content_length = self.export.content_length();
        let encoded = urlencoding::encode(self.export.file_name()).replace('+', "%20");
        let disposition = format!("attachment;filename*=UTF-8''{encoded}");
        let mut response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from_async_read(self.export));
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
        if let Ok(value) = HeaderValue::from_str(&content_length.to_string()) {
            response.headers_mut().insert(CONTENT_LENGTH, value);
        }
        if let Ok(value) = HeaderValue::from_str(&disposition) {
            response.headers_mut().insert(CONTENT_DISPOSITION, value);
        }
        response
    }
}
