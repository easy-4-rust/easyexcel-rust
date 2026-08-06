/// 对应 Java：无直接对应对象；Rust 架构扩展。 A cell value ready for BIFF8 emission, with an XF index for date formats.
#[derive(Debug, Clone)]
pub struct Biff8Cell {
    /// Logical value.
    pub value: Biff8Value,
    /// XF index (`XF_GENERAL` / `XF_DATE` / `XF_DATETIME` / custom ≥ 18).
    pub xf: u16,
}

impl Biff8Cell {
    /// Creates a general-format cell.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn general(value: Biff8Value) -> Self {
        Self {
            value,
            xf: XF_GENERAL,
        }
    }

    /// Creates a date-formatted numeric serial cell.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn date_serial(serial: f64) -> Self {
        Self {
            value: Biff8Value::Number(serial),
            xf: XF_DATE,
        }
    }

    /// Creates a datetime-formatted numeric serial cell.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn datetime_serial(serial: f64) -> Self {
        Self {
            value: Biff8Value::Number(serial),
            xf: XF_DATETIME,
        }
    }

    /// Returns a copy with a different XF index (styled date/general cells).
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn with_xf(mut self, xf: u16) -> Self {
        self.xf = xf;
        self
    }
}

