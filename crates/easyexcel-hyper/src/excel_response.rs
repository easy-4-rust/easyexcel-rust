use std::io;

use bytes::Bytes;
use easyexcel::ExcelRow;
use easyexcel::io::Format;
use easyexcel_web::{ExcelExport, WebExecutionContext};
use futures_util::TryStreamExt;
use http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use http::{HeaderValue, Response, StatusCode};
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use tokio_util::io::ReaderStream;

use crate::ExcelHyperError;

/// Hyper 成功与失败响应共享的类型擦除 body。
pub type ResponseBody = BoxBody<Bytes, io::Error>;

/// Hyper 原生流式 Excel 响应。
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
    ) -> Result<Self, ExcelHyperError>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        let request_id = context.request_id().to_string();
        let export = ExcelExport::prepare(rows, format, file_name, sheet_name, context)
            .await
            .map_err(|error| ExcelHyperError::new(error, request_id))?;
        Ok(Self { export })
    }

    /// 转换为 Hyper 流式响应。
    #[must_use]
    pub fn into_response(self) -> Response<ResponseBody> {
        let disposition =
            easyexcel_web::excel_attachment_content_disposition(self.export.file_name());
        let content_type = self.export.content_type();
        let content_length = self.export.content_length();
        let chunk_size = self.export.io_chunk_size();
        let stream = ReaderStream::with_capacity(self.export, chunk_size).map_ok(Frame::data);
        let body = StreamBody::new(stream).boxed();
        let mut response = Response::new(body);
        *response.status_mut() = StatusCode::OK;
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
