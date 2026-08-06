use axum::body::Body;
use axum::extract::{FromRef, FromRequest, Request};
use easyexcel::ExcelRow;
use easyexcel_web::{ExcelImport, ExcelRequestMetadata, ExcelRows, ExcelWebRuntime};

use crate::ExcelRejection;

/// Axum 原生 Excel 请求 extractor。
#[derive(Debug)]
pub struct ExcelRequest<T> {
    import: ExcelImport<T>,
}

impl<T> ExcelRequest<T>
where
    T: ExcelRow + Send + 'static,
{
    /// 将请求转换为具有背压的类型化行流。
    #[must_use]
    pub fn into_rows(self) -> ExcelRows<T> {
        self.import.rows()
    }

    /// 返回上传文件名。
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.import.file_name()
    }

    /// 返回已接收字节数。
    #[must_use]
    pub const fn received_bytes(&self) -> u64 {
        self.import.received_bytes()
    }

    /// 返回请求标识。
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.import.context().request_id()
    }
}

impl<S, T> FromRequest<S> for ExcelRequest<T>
where
    S: Send + Sync,
    ExcelWebRuntime: FromRef<S>,
    T: ExcelRow + Send + 'static,
{
    type Rejection = ExcelRejection;

    async fn from_request(request: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let runtime = ExcelWebRuntime::from_ref(state);
        let headers = request.headers();
        let explicit_file_name = header(headers, "x-excel-file-name");
        let content_disposition = header(headers, http::header::CONTENT_DISPOSITION.as_str());
        let content_type = header(headers, http::header::CONTENT_TYPE.as_str());
        let request_id = header(headers, "x-request-id");
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
        .map_err(|error| ExcelRejection::new(error, context.request_id()))?;
        let stream = request.into_body().into_data_stream();
        let import = ExcelImport::receive(
            stream,
            metadata.extension(),
            metadata.file_name().map(ToOwned::to_owned),
            context.clone(),
        )
        .await
        .map_err(|error| ExcelRejection::new(error, context.request_id()))?;
        Ok(Self { import })
    }
}

fn header(headers: &http::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}
