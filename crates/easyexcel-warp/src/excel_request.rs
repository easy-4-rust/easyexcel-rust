use bytes::Buf;
use easyexcel::ExcelRow;
use easyexcel_web::{ExcelImport, ExcelRequestMetadata, ExcelRows, ExcelWebRuntime};
use futures_util::TryStreamExt;
use warp::{Filter, Rejection};

use crate::ExcelWarpRejection;

/// Warp 原生 Excel 请求 filter 输出。
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

/// 创建读取原始 Excel 请求体的 Warp filter。
#[must_use]
pub fn excel_request<T>(
    runtime: ExcelWebRuntime,
) -> impl Filter<Extract = (ExcelRequest<T>,), Error = Rejection> + Clone
where
    T: ExcelRow + Send + 'static,
{
    warp::header::optional::<String>("x-excel-file-name")
        .and(warp::header::optional::<String>("content-disposition"))
        .and(warp::header::optional::<String>("content-type"))
        .and(warp::header::optional::<String>("x-request-id"))
        .and(warp::body::stream())
        .and_then(
            move |file_name, disposition, content_type, request_id, stream| {
                let runtime = runtime.clone();
                async move {
                    receive::<T, _, _>(
                        runtime,
                        file_name,
                        disposition,
                        content_type,
                        request_id,
                        stream,
                    )
                    .await
                }
            },
        )
}

async fn receive<T, S, B>(
    runtime: ExcelWebRuntime,
    explicit_file_name: Option<String>,
    content_disposition: Option<String>,
    content_type: Option<String>,
    request_id: Option<String>,
    stream: S,
) -> Result<ExcelRequest<T>, Rejection>
where
    T: ExcelRow + Send + 'static,
    S: futures_util::Stream<Item = Result<B, warp::Error>>,
    B: Buf,
{
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
    .map_err(|error| warp::reject::custom(ExcelWarpRejection::new(error, context.request_id())))?;
    let bytes = stream.map_ok(|mut buffer| buffer.copy_to_bytes(buffer.remaining()));
    let import = ExcelImport::receive(
        bytes,
        metadata.extension(),
        metadata.file_name().map(ToOwned::to_owned),
        context.clone(),
    )
    .await
    .map_err(|error| warp::reject::custom(ExcelWarpRejection::new(error, context.request_id())))?;
    Ok(ExcelRequest { import })
}
