use actix_web::body::BoxBody;
use actix_web::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use actix_web::{HttpRequest, HttpResponse, Responder};
use easyexcel::ExcelRow;
use easyexcel::io::Format;
use easyexcel_web::{ExcelExport, WebExecutionContext};
use tokio_util::io::ReaderStream;

use crate::ExcelActixError;

/// Actix Web 原生流式 Excel responder。
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
    ) -> Result<Self, ExcelActixError>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        let request_id = context.request_id().to_string();
        let export = ExcelExport::prepare(rows, format, file_name, sheet_name, context)
            .await
            .map_err(|error| ExcelActixError::new(error, request_id))?;
        Ok(Self { export })
    }
}

impl<T> Responder for ExcelResponse<T>
where
    T: ExcelRow + Send + 'static,
{
    type Body = BoxBody;

    fn respond_to(self, _request: &HttpRequest) -> HttpResponse<Self::Body> {
        let disposition =
            easyexcel_web::excel_attachment_content_disposition(self.export.file_name());
        let chunk_size = self.export.io_chunk_size();
        HttpResponse::Ok()
            .insert_header((CONTENT_TYPE, self.export.content_type()))
            .insert_header((CONTENT_LENGTH, self.export.content_length()))
            .insert_header((CONTENT_DISPOSITION, disposition))
            .streaming(ReaderStream::with_capacity(self.export, chunk_size))
    }
}
