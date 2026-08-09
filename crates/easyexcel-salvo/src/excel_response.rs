use easyexcel::ExcelRow;
use easyexcel::io::Format;
use easyexcel_web::{ExcelExport, WebExecutionContext};
use salvo::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use salvo::http::{HeaderValue, StatusCode};
use salvo::{Depot, Request, Response, Writer, async_trait};
use tokio_util::io::ReaderStream;

use crate::ExcelSalvoError;

/// Salvo 原生流式 Excel writer。
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
    ) -> Result<Self, ExcelSalvoError>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        let request_id = context.request_id().to_string();
        let export = ExcelExport::prepare(rows, format, file_name, sheet_name, context)
            .await
            .map_err(|error| ExcelSalvoError::new(error, request_id))?;
        Ok(Self { export })
    }
}

#[async_trait]
impl<T> Writer for ExcelResponse<T>
where
    T: ExcelRow + Send + 'static,
{
    async fn write(self, _request: &mut Request, _depot: &mut Depot, response: &mut Response) {
        let content_type = self.export.content_type();
        let content_length = self.export.content_length();
        let disposition =
            easyexcel_web::excel_attachment_content_disposition(self.export.file_name());
        let chunk_size = self.export.io_chunk_size();
        response.status_code = Some(StatusCode::OK);
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
        if let Ok(value) = HeaderValue::from_str(&content_length.to_string()) {
            response.headers_mut().insert(CONTENT_LENGTH, value);
        }
        if let Ok(value) = HeaderValue::from_str(&disposition) {
            response.headers_mut().insert(CONTENT_DISPOSITION, value);
        }
        response.stream(ReaderStream::with_capacity(self.export, chunk_size));
    }
}
