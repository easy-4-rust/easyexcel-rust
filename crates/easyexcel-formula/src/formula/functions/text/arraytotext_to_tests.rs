/// ARRAYTOTEXT(array, [format])
///
/// format 0 (default) = concise: values joined with ", ".
/// format 1 = strict: `{row1_col1,row1_col2;row2_col1,...}` with text quoted.
fn arraytotext(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let format: i64 = if args.len() >= 2 && !matches!(args[1], Value::Empty) {
        match to_number(&args[1]) {
            Ok(n) => n.trunc() as i64,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };

    let strict = format == 1;

    // Materialise the array
    let arr = match &args[0] {
        Value::Ref(r) => ctx.ref_to_array(*r),
        Value::Array(a) => a.clone(),
        other => {
            // Scalar: just format the single value
            return Value::Text(value_to_text_str(other, strict));
        }
    };

    if !strict {
        // format 0: flat list joined by ", "
        let parts: Vec<String> = arr
            .data
            .iter()
            .map(|v| value_to_text_str(v, false))
            .collect();
        return Value::Text(parts.join(", "));
    }

    // format 1: {col,col;row2...}
    let mut rows: Vec<String> = Vec::with_capacity(arr.rows);
    for r in 0..arr.rows {
        let mut cols: Vec<String> = Vec::with_capacity(arr.cols);
        for c in 0..arr.cols {
            let v = arr.get(r, c).unwrap_or(&Value::Empty);
            cols.push(value_to_text_str(v, true));
        }
        rows.push(cols.join(","));
    }
    Value::Text(format!("{{{}}}", rows.join(";")))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "../text_tests/tests.rs"]
mod tests;
