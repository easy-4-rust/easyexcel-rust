use easyexcel::ExcelRow;
use easyexcel::io::Format;
use easyexcel_web::{ExcelExport, WebExecutionContext};
use rocket::Request;
use rocket::response::{self, Responder, Response};

use crate::ExcelRocketError;

/// Rocket 原生流式 Excel responder。
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
    ) -> Result<Self, ExcelRocketError>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        let request_id = context.request_id().to_string();
        let export = ExcelExport::prepare(rows, format, file_name, sheet_name, context)
            .await
            .map_err(|error| ExcelRocketError::new(error, request_id))?;
        Ok(Self { export })
    }
}

impl<'r, T> Responder<'r, 'static> for ExcelResponse<T>
where
    T: ExcelRow + Send + 'static,
{
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        let content_type = self.export.content_type();
        let content_length = self.export.content_length();
        let encoded = urlencoding::encode(self.export.file_name()).replace('+', "%20");
        Response::build()
            .raw_header("Content-Type", content_type)
            .raw_header("Content-Length", content_length.to_string())
            .raw_header(
                "Content-Disposition",
                format!("attachment;filename*=UTF-8''{encoded}"),
            )
            .streamed_body(self.export)
            .ok()
    }
}
