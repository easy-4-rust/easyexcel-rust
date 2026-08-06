use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest, web};
use easyexcel::ExcelRow;
use easyexcel_web::{ExcelImport, ExcelRequestMetadata, ExcelRows, ExcelWebRuntime};
use futures_util::FutureExt;
use futures_util::future::LocalBoxFuture;

use crate::ExcelActixError;

/// Actix Web 原生 Excel 请求 extractor。
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

    /// 返回请求标识。
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.import.context().request_id()
    }
}

impl<T> FromRequest for ExcelRequest<T>
where
    T: ExcelRow + Send + 'static,
{
    type Error = ExcelActixError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(request: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let runtime = request
            .app_data::<web::Data<ExcelWebRuntime>>()
            .map(|runtime| runtime.get_ref().clone());
        let explicit_file_name = header(request, "x-excel-file-name");
        let content_disposition = header(request, "content-disposition");
        let content_type = header(request, "content-type");
        let request_id = header(request, "x-request-id");
        let body = payload.take();

        async move {
            let runtime = runtime
                .unwrap_or_else(|| ExcelWebRuntime::new(easyexcel_web::ExcelWebPolicy::default()));
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
            .map_err(|error| ExcelActixError::new(error, context.request_id()))?;
            let import = ExcelImport::receive(
                body,
                metadata.extension(),
                metadata.file_name().map(ToOwned::to_owned),
                context.clone(),
            )
            .await
            .map_err(|error| ExcelActixError::new(error, context.request_id()))?;
            Ok(Self { import })
        }
        .boxed_local()
    }
}

fn header(request: &HttpRequest, name: &str) -> Option<String> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}
