//! benchmark runner 的最小命令行契约。

use std::path::PathBuf;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 单次 runner 调用参数。
#[derive(Debug)]
pub(crate) struct Arguments {
    pub(crate) spec: PathBuf,
    pub(crate) scenario: String,
    pub(crate) rows: u64,
    pub(crate) input: Option<PathBuf>,
    pub(crate) output: Option<PathBuf>,
    pub(crate) worker_count: u32,
    pub(crate) temperature: String,
    pub(crate) warmups: u32,
}

impl Arguments {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 解析显式的 `--key value` 参数。
    pub(crate) fn parse() -> Result<Self, String> {
        let mut values = std::collections::HashMap::new();
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            values.insert(flag, value);
        }
        let spec = required(&mut values, "--spec")?.into();
        let scenario = required(&mut values, "--scenario")?;
        let rows = required(&mut values, "--rows")?
            .parse::<u64>()
            .map_err(|error| format!("invalid --rows: {error}"))?;
        let input = values.remove("--input").map(PathBuf::from);
        let output = values.remove("--output").map(PathBuf::from);
        let worker_count = values
            .remove("--workers")
            .map_or(Ok(1), |value| value.parse::<u32>())
            .map_err(|error| format!("invalid --workers: {error}"))?;
        let temperature = values
            .remove("--temperature")
            .unwrap_or_else(|| "cold".to_owned());
        if !matches!(temperature.as_str(), "cold" | "steady") {
            return Err("--temperature must be cold or steady".to_owned());
        }
        let warmups = values
            .remove("--warmups")
            .map_or(Ok(0), |value| value.parse::<u32>())
            .map_err(|error| format!("invalid --warmups: {error}"))?;
        if temperature == "cold" && warmups != 0 {
            return Err("cold measurements must not execute warmups".to_owned());
        }
        if let Some(flag) = values.keys().next() {
            return Err(format!("unknown argument: {flag}"));
        }
        Ok(Self {
            spec,
            scenario,
            rows,
            input,
            output,
            worker_count,
            temperature,
            warmups,
        })
    }
}

fn required(
    values: &mut std::collections::HashMap<String, String>,
    name: &str,
) -> Result<String, String> {
    values
        .remove(name)
        .ok_or_else(|| format!("missing required argument {name}"))
}
