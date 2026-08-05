//! `easyexcel` 门面与基础引擎 crate 的依赖边界审计。

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::TaskResult;

const FACADE_MANIFEST: &str = "crates/easyexcel/Cargo.toml";
const FACADE_LIB: &str = "crates/easyexcel/src/lib.rs";
const EHCACHE_COMPAT: &str = "crates/easyexcel/src/cache/ehcache.rs";
const MOKA_ADAPTER: &str = "crates/easyexcel/src/cache/moka_cache.rs";
const OUTPUT_STREAM_COMPAT: &str = "crates/easyexcel/src/write/excel_output_stream.rs";

const REQUIRED_ENGINE_DEPENDENCIES: &[&str] = &[
    "easyexcel-cache",
    "easyexcel-csv",
    "easyexcel-format",
    "easyexcel-formula",
    "easyexcel-io",
    "easyexcel-model",
    "easyexcel-tabular",
    "easyexcel-utils",
    "easyexcel-xls",
    "easyexcel-xlsx",
];

const FORBIDDEN_FACADE_DEPENDENCIES: &[&str] = &[
    "aes",
    "calamine",
    "cfb",
    "csv",
    "encoding_rs",
    "encoding_rs_io",
    "flate2",
    "md-5",
    "moka",
    "ms-offcrypto-writer",
    "office-crypto",
    "quick-xml",
    "rand",
    "rust_xlsxwriter",
    "sha1",
    "sha2",
    "ssfmt",
    "tempfile",
    "zip",
];

/// 校验门面只依赖基础引擎，不直接依赖格式、压缩、加密或缓存实现库。
pub(crate) fn audit() -> TaskResult {
    let manifest = read(FACADE_MANIFEST)?;
    let dependencies = dependency_names(&manifest);

    let missing = REQUIRED_ENGINE_DEPENDENCIES
        .iter()
        .copied()
        .filter(|name| !dependencies.contains(*name))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "facade is missing required engine dependencies: {}",
            missing.join(", ")
        )
        .into());
    }

    let forbidden = FORBIDDEN_FACADE_DEPENDENCIES
        .iter()
        .copied()
        .filter(|name| dependencies.contains(*name))
        .collect::<Vec<_>>();
    if !forbidden.is_empty() {
        return Err(format!(
            "facade directly depends on low-level implementation crates: {}",
            forbidden.join(", ")
        )
        .into());
    }

    let facade = read(FACADE_LIB)?;
    for module in ["csv", "formula", "format", "io", "model", "tabular", "xls", "xlsx"] {
        require_contains(
            FACADE_LIB,
            &facade,
            &format!("pub mod {module};"),
            "foundation API facade module",
        )?;
    }

    let ehcache = read(EHCACHE_COMPAT)?;
    require_contains(
        EHCACHE_COMPAT,
        &ehcache,
        "MokaCache as Ehcache",
        "Java-compatible alias",
    )?;
    require_absent(EHCACHE_COMPAT, &ehcache, "struct Ehcache", "Ehcache implementation")?;
    require_absent(EHCACHE_COMPAT, &ehcache, "moka::", "direct Moka dependency")?;

    let moka_adapter = read(MOKA_ADAPTER)?;
    require_contains(
        MOKA_ADAPTER,
        &moka_adapter,
        "SharedStringCachePolicy",
        "engine-owned cache policy",
    )?;
    require_absent(MOKA_ADAPTER, &moka_adapter, "moka::", "direct Moka implementation")?;

    let output_stream = read(OUTPUT_STREAM_COMPAT)?;
    require_contains(
        OUTPUT_STREAM_COMPAT,
        &output_stream,
        "easyexcel_io::CloseableOutputStream<W>",
        "engine-owned output stream",
    )?;
    require_absent(
        OUTPUT_STREAM_COMPAT,
        &output_stream,
        "Arc<Mutex",
        "shared output implementation",
    )?;

    println!(
        "facade-boundary-audit ok: {} engine dependencies, no low-level direct dependencies",
        REQUIRED_ENGINE_DEPENDENCIES.len()
    );
    Ok(())
}

fn read(path: &str) -> TaskResult<String> {
    if !Path::new(path).is_file() {
        return Err(format!("missing {path}").into());
    }
    Ok(fs::read_to_string(path)?)
}

fn dependency_names(manifest: &str) -> BTreeSet<&str> {
    let mut dependencies = BTreeSet::new();
    let mut in_dependencies = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_dependencies = line == "[dependencies]";
            continue;
        }
        if !in_dependencies || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            dependencies.insert(name.trim());
        }
    }
    dependencies
}

fn require_contains(path: &str, source: &str, needle: &str, purpose: &str) -> TaskResult {
    if source.contains(needle) {
        return Ok(());
    }
    Err(format!("{path} must contain {needle:?} ({purpose})").into())
}

fn require_absent(path: &str, source: &str, needle: &str, purpose: &str) -> TaskResult {
    if !source.contains(needle) {
        return Ok(());
    }
    Err(format!("{path} must not contain {needle:?} ({purpose})").into())
}
