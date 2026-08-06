fn read(path: &str) -> TaskResult<String> {
    if !Path::new(path).is_file() {
        return Err(format!("missing {path}").into());
    }
    Ok(fs::read_to_string(path)?)
}

/// 读取主模块及其同名实现子目录，防止通过 `include!` 拆分绕过边界审计。
fn read_module_family(path: &str) -> TaskResult<String> {
    let mut source = read(path)?;
    let implementation_root = Path::new(path).with_extension("");
    if !implementation_root.is_dir() {
        return Ok(source);
    }

    let mut pending = vec![implementation_root];
    let mut implementation_files = Vec::new();
    while let Some(candidate) = pending.pop() {
        if candidate.is_dir() {
            for entry in fs::read_dir(candidate)? {
                pending.push(entry?.path());
            }
        } else if candidate.extension().and_then(|value| value.to_str()) == Some("rs") {
            implementation_files.push(candidate);
        }
    }
    implementation_files.sort();
    for implementation_file in implementation_files {
        source.push('\n');
        source.push_str(&fs::read_to_string(implementation_file)?);
    }
    Ok(source)
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
            let name = name.trim();
            dependencies.insert(name.strip_suffix(".workspace").unwrap_or(name));
        }
    }
    dependencies
}

fn production_prefix(source: &str) -> &str {
    source
        .split_once("#[cfg(test)]")
        .map_or(source, |(production, _)| production)
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

fn require_no_wildcard_imports(path: &str, source: &str) -> TaskResult {
    let wildcard_import = source.lines().map(str::trim).find(|line| {
        (line.starts_with("use ") || line.starts_with("pub use ")) && line.contains("::*")
    });
    if wildcard_import.is_none() {
        return Ok(());
    }
    Err(format!(
        "{path} must not contain wildcard imports: {}",
        wildcard_import.unwrap_or_default()
    )
    .into())
}

fn require_path_absent(path: &str, purpose: &str) -> TaskResult {
    if !Path::new(path).exists() {
        return Ok(());
    }
    Err(format!("{path} must not exist ({purpose})").into())
}

fn require_tree_absent_case_insensitive(root: &str, needle: &str, purpose: &str) -> TaskResult {
    let needle = needle.to_ascii_lowercase();
    let mut pending = vec![Path::new(root).to_path_buf()];
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                pending.push(entry?.path());
            }
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path)?;
        if source.to_ascii_lowercase().contains(&needle) {
            return Err(
                format!("{} must not contain {needle:?} ({purpose})", path.display()).into(),
            );
        }
    }
    Ok(())
}
