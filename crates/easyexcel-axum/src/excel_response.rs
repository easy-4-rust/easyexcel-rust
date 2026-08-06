use axum::body::Body;
use axum::response::{IntoResponse, Response};
use easyexcel::ExcelRow;
use easyexcel::io::Format;
use easyexcel_web::{ExcelExport, WebExecutionContext};
use http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use http::{HeaderValue, StatusCode};
use tokio_util::io::ReaderStream;

use crate::ExcelRejection;

/// Axum 原生流式 Excel responder。
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
    /// 生成失败、超时、取消或超过资源限制时返回统一拒绝响应。
    pub async fn prepare<I>(
        rows: I,
        format: Format,
        file_name: impl Into<String>,
        sheet_name: impl Into<String>,
        context: WebExecutionContext,
    ) -> Result<Self, ExcelRejection>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        let request_id = context.request_id().to_string();
        let export = ExcelExport::prepare(rows, format, file_name, sheet_name, context)
            .await
            .map_err(|error| ExcelRejection::new(error, request_id))?;
        Ok(Self { export })
    }
}

impl<T> IntoResponse for ExcelResponse<T>
where
    T: ExcelRow + Send + 'static,
{
    fn into_response(self) -> Response {
        let content_type = HeaderValue::from_static(self.export.content_type());
        let content_length = HeaderValue::from_str(&self.export.content_length().to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("0"));
        let encoded = urlencoding::encode(self.export.file_name()).replace('+', "%20");
        let disposition = HeaderValue::from_str(&format!("attachment;filename*=UTF-8''{encoded}"))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment;filename=download.xlsx"));
        let chunk_size = self.export.io_chunk_size();
        let stream = ReaderStream::with_capacity(self.export, chunk_size);
        let mut response = Response::new(Body::from_stream(stream));
        *response.status_mut() = StatusCode::OK;
        response.headers_mut().insert(CONTENT_TYPE, content_type);
        response
            .headers_mut()
            .insert(CONTENT_LENGTH, content_length);
        response
            .headers_mut()
            .insert(CONTENT_DISPOSITION, disposition);
        response
    }
}
