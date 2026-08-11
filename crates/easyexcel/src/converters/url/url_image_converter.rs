//! 对应 Java：`com.alibaba.excel.converters.url.UrlImageConverter` with
//! Java's default timeout values (1s connect, 5s read).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use url::Url;

use crate::converters::Converter;
use crate::core::cell_value::CellValue;
use crate::core::convert_context::ConvertContext;
use crate::core::excel_error::ExcelError;
use crate::core::into_excel_cell::IntoExcelCell;
use crate::core::write_converter_context::WriteConverterContext;
use crate::write::write_cell_data::WriteCellData;

/// Java `UrlImageConverter.urlConnectTimeout` 的线程安全毫秒配置。
pub static URL_CONNECT_TIMEOUT: AtomicU64 = AtomicU64::new(1_000);

/// Java `UrlImageConverter.urlReadTimeout` 的线程安全毫秒配置。
pub static URL_READ_TIMEOUT: AtomicU64 = AtomicU64::new(5_000);

/// 对应 Java：com.alibaba.excel.converters.url.UrlImageConverter。 Java `UrlImageConverter` equivalent with Java's default timeout values.
///
/// Uses the `ureq` crate for HTTP; defaulting to 1s connect and 5s read
/// matches Java `EasyExcel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UrlImageConverter {
    connect_timeout: Duration,
    read_timeout: Duration,
}

impl UrlImageConverter {
    /// Java `EasyExcel`'s default URL connection timeout.
    pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
    /// Java `EasyExcel`'s default URL response-read timeout.
    pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(5);

    /// Creates a converter with explicit connection and response-read timeouts.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.url.UrlImageConverter。
    pub const fn new(connect_timeout: Duration, read_timeout: Duration) -> Self {
        Self {
            connect_timeout,
            read_timeout,
        }
    }

    /// Returns the configured connection timeout. (Java `getConnectTimeout()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.url.UrlImageConverter。
    pub const fn connect_timeout(self) -> Duration {
        self.connect_timeout
    }

    /// Returns the configured response-read timeout. (Java `getReadTimeout()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.converters.url.UrlImageConverter。
    pub const fn read_timeout(self) -> Duration {
        self.read_timeout
    }

    fn download(self, value: &Url) -> Result<Vec<u8>, ExcelError> {
        easyexcel_io::io::http_fetch::download_bytes(
            value.as_str(),
            self.connect_timeout,
            self.read_timeout,
        )
        .map_err(ExcelError::from)
    }
}

impl Default for UrlImageConverter {
    fn default() -> Self {
        Self::new(
            Duration::from_millis(URL_CONNECT_TIMEOUT.load(Ordering::Relaxed)),
            Duration::from_millis(URL_READ_TIMEOUT.load(Ordering::Relaxed)),
        )
    }
}

impl Converter<Url> for UrlImageConverter {
    fn convert_to_excel_data(
        &self,
        context: &WriteConverterContext<'_, Url>,
    ) -> Result<WriteCellData, ExcelError> {
        self.download(context.value())
            .map(WriteCellData::from_image)
    }
}

impl IntoExcelCell for Url {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        UrlImageConverter::default()
            .download(self)
            .map(CellValue::Image)
    }
}
