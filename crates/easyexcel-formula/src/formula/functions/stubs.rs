//! Documented stubs for functions we deliberately do not support: cube (`CUBE*`),
//! web (`WEBSERVICE`, `FILTERXML`, `ENCODEURL`), RTD, and pivot-only functions.
//! These return `#N/A` (or `#NAME?` where Excel itself would not know them) so
//! that opening a workbook that uses them degrades gracefully rather than
//! mis-evaluating.

use super::Registry;
use crate::formula::context::Context;
use crate::formula::value::Value;
use easyexcel_model::error::CellError;
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn register(r: &mut Registry) {
    for name in EXT_FUNCTIONS {
        r.add(name, 0, super::VARIADIC, false, unsupported);
    }
}

fn unsupported(_: &mut dyn Context, _: &[Value]) -> Value {
    // PARITY: external-data/cube/RTD functions cannot be computed offline.
    Value::Error(CellError::NA)
}

/// Functions that require external data, a cube/OLAP connection, RTD, or pivot
/// cache and therefore cannot be evaluated by an offline engine.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const EXT_FUNCTIONS: &[&str] = &[
    "CUBEKPIMEMBER",
    "CUBEMEMBER",
    "CUBEMEMBERPROPERTY",
    "CUBERANKEDMEMBER",
    "CUBESET",
    "CUBESETCOUNT",
    "CUBEVALUE",
    "RTD",
    "WEBSERVICE",
    "FILTERXML",
    "ENCODEURL",
    "GETPIVOTDATA",
    // Locale / East-Asian text and phonetic functions (need locale/phonetic data).
    "ASC",
    "DBCS",
    "PHONETIC",
    "BAHTTEXT",
    "DETECTLANGUAGE",
    "TRANSLATE",
    // Pivot / image / stock / macro functions (need a host application or service).
    "GROUPBY",
    "PIVOTBY",
    "IMAGE",
    "STOCKHISTORY",
    "CALL",
    "REGISTER.ID",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::functions::testutil::TestCtx;

    #[test]
    fn ext_functions_all_registered() {
        let r = crate::formula::functions::Registry::standard();
        for name in EXT_FUNCTIONS {
            assert!(
                r.get(name).is_some(),
                "EXT function {name} should be registered"
            );
        }
    }

    #[test]
    fn unsupported_returns_na() {
        let mut ctx = TestCtx::new();
        for name in EXT_FUNCTIONS {
            let r = unsupported(&mut ctx, &[]);
            assert_eq!(r, Value::Error(CellError::NA), "{name} should return #N/A");
        }
    }

    #[test]
    fn ext_functions_count() {
        // 验证列表不为空且合理
        assert!(EXT_FUNCTIONS.len() >= 20);
    }
}
