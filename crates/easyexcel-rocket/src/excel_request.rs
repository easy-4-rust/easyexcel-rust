use easyexcel::ExcelRow;
use easyexcel_web::{ExcelImport, ExcelRequestMetadata, ExcelRows, ExcelWebRuntime};
use rocket::Request;
use rocket::data::{ByteUnit, Data, FromData, Outcome};
use rocket::http::Status;
use tokio_util::io::ReaderStream;

use crate::ExcelRocketError;

/// Rocket 原生 Excel data guard。
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

#[rocket::async_trait]
impl<'r, T> FromData<'r> for ExcelRequest<T>
where
    T: ExcelRow + Send + 'static,
{
    type Error = ExcelRocketError;

    async fn from_data(request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let runtime = request
            .rocket()
            .state::<ExcelWebRuntime>()
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
        let metadata = match ExcelRequestMetadata::resolve(
            explicit_file_name.as_deref(),
            content_disposition.as_deref(),
            content_type.as_deref(),
            request_id.as_deref(),
        ) {
            Ok(metadata) => metadata,
            Err(error) => {
                let error = ExcelRocketError::new(error, context.request_id());
                return Outcome::Error((error.status(), error));
            }
        };
        let limit = context
            .policy()
            .resource_limits()
            .max_file_bytes()
            .saturating_add(1);
        let stream = ReaderStream::new(data.open(ByteUnit::Byte(limit)));
        match ExcelImport::receive(
            stream,
            metadata.extension(),
            metadata.file_name().map(ToOwned::to_owned),
            context.clone(),
        )
        .await
        {
            Ok(import) => Outcome::Success(Self { import }),
            Err(error) => {
                let error = ExcelRocketError::new(error, context.request_id());
                Outcome::Error((Status::new(error.status().code), error))
            }
        }
    }
}

fn header(request: &Request<'_>, name: &str) -> Option<String> {
    request.headers().get_one(name).map(ToOwned::to_owned)
}
