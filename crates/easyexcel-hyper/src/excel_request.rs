use bytes::Bytes;
use easyexcel::ExcelRow;
use easyexcel_web::{ExcelImport, ExcelRequestMetadata, ExcelRows, ExcelWebRuntime};
use http::Request;
use http_body_util::BodyExt;
use hyper::body::Body;

use crate::ExcelHyperError;

/// Hyper 原生 Excel 请求桥接对象。
#[derive(Debug)]
pub struct ExcelRequest<T> {
    import: ExcelImport<T>,
}

impl<T> ExcelRequest<T>
where
    T: ExcelRow + Send + 'static,
{
    /// 从任意 Hyper HTTP body 接收 Excel 请求。
    ///
    /// # Errors
    ///
    /// 请求元数据、传输、资源限制或临时存储失败时返回统一错误。
    pub async fn from_request<B>(
        request: Request<B>,
        runtime: &ExcelWebRuntime,
    ) -> Result<Self, ExcelHyperError>
    where
        B: Body<Data = Bytes> + Send + 'static,
        B::Error: std::fmt::Display,
    {
        let (parts, body) = request.into_parts();
        let explicit_file_name = header(&parts.headers, "x-excel-file-name");
        let content_disposition = header(&parts.headers, "content-disposition");
        let content_type = header(&parts.headers, "content-type");
        let request_id = header(&parts.headers, "x-request-id");
        let context = request_id.as_deref().map_or_else(
            || runtime.generated_context(),
            |request_id| runtime.context(request_id.to_string()),
        );
        let metadata = ExcelRequestMetadata::resolve(
            explicit_file_name.as_deref(),
            content_disposition.as_deref(),
            content_type.as_deref(),
            request_id.as_deref(),
        )
        .map_err(|error| ExcelHyperError::new(error, context.request_id()))?;
        let import = ExcelImport::receive(
            body.into_data_stream(),
            metadata.extension(),
            metadata.file_name().map(ToOwned::to_owned),
            context.clone(),
        )
        .await
        .map_err(|error| ExcelHyperError::new(error, context.request_id()))?;
        Ok(Self { import })
    }

    /// 将请求转换为具有背压的类型化行流。
    #[must_use]
    pub fn into_rows(self) -> ExcelRows<T> {
        self.import.rows()
    }

    /// 返回请求标识。
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.import.context().request_id()
    }
}

fn header(headers: &http::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}
