use easyexcel::ExcelRow;
use easyexcel_web::{ExcelImport, ExcelRequestMetadata, ExcelRows, ExcelWebRuntime};
use poem::Request;
use poem::web::{FromRequest, RequestBody};

use crate::ExcelPoemError;

/// Poem 原生 Excel 请求 extractor。
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

    /// 返回请求标识。
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.import.context().request_id()
    }
}

impl<'a, T> FromRequest<'a> for ExcelRequest<T>
where
    T: ExcelRow + Send + 'static,
{
    async fn from_request(request: &'a Request, body: &mut RequestBody) -> poem::Result<Self> {
        let runtime = request
            .data::<ExcelWebRuntime>()
            .cloned()
            .unwrap_or_else(|| ExcelWebRuntime::new(easyexcel_web::ExcelWebPolicy::default()));
        let explicit_file_name = header(request, "x-excel-file-name");
        let content_disposition = header(request, "content-disposition");
        let content_type = header(request, "content-type");
        let request_id = header(request, "x-request-id");
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
        .map_err(|error| poem::Error::from(ExcelPoemError::new(error, context.request_id())))?;
        let stream = body.take()?.into_bytes_stream();
        let import = ExcelImport::receive(
            stream,
            metadata.extension(),
            metadata.file_name().map(ToOwned::to_owned),
            context.clone(),
        )
        .await
        .map_err(|error| poem::Error::from(ExcelPoemError::new(error, context.request_id())))?;
        Ok(Self { import })
    }
}

fn header(request: &Request, name: &str) -> Option<String> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}
