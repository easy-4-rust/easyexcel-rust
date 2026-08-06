fn pop_stdev(nums: &[f64]) -> Value {
    match pop_var(nums) {
        Value::Number(v) => Value::Number(v.sqrt()),
        other => other,
    }
}

// --- AGGREGATE -------------------------------------------------------------

/// `AGGREGATE(function_num`, options, ref1/array, [k])
///
/// `function_num` 1-11: same as SUBTOTAL (1=AVERAGE … 11=VARP)
/// `function_num` 12=MEDIAN, 13=MODE.SNGL
/// `function_num` 14=LARGE, 15=SMALL (require k arg)
/// `function_num` 16=PERCENTILE.INC, 17=QUARTILE.INC (require k arg)
/// `function_num` 18=PERCENTILE.EXC, 19=QUARTILE.EXC (require k arg)
///
/// options:
///   0 = ignore nested subtotal/aggregate (not tracked — PARITY)
///   1 = ignore hidden rows (not supported — PARITY)
///   2 = ignore hidden+errors
///   3 = ignore hidden+errors+nested (not fully tracked — PARITY: skip errors)
///   4 = ignore nothing
///   5 = ignore hidden (PARITY: treated as 0)
///   6 = ignore errors
///   7 = ignore errors+nested (PARITY: skip errors)
///
/// Implementation: options {2,3,6,7} → skip error values; others → propagate errors.
fn aggregate(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let fn_num = match n(args, 0) {
        Ok(v) => v as u32,
        Err(e) => return Value::Error(e),
    };
    let opts = match n(args, 1) {
        Ok(v) => v as u32,
        Err(e) => return Value::Error(e),
    };
    // options that mean "skip errors"
    let skip_errors = matches!(opts, 2 | 3 | 6 | 7);

    // data refs start at index 2; k (if needed) is the last arg
    let needs_k = (14..=19).contains(&fn_num);
    let (data_args, k_arg) = if needs_k && args.len() >= 4 {
        (&args[2..args.len() - 1], Some(&args[args.len() - 1]))
    } else {
        (&args[2..], None)
    };

    // Collect flat numeric values
    let mut nums: Vec<f64> = Vec::new();
    for v in data_args {
        for cell in ctx.flatten(v) {
            match cell {
                Value::Number(num) => nums.push(num),
                Value::Error(e) if !skip_errors => return Value::Error(e),
                _ => {}
            }
        }
    }

    match fn_num {
        1 => {
            if nums.is_empty() {
                Value::Error(CellError::Div0)
            } else {
                Value::Number(nums.iter().sum::<f64>() / nums.len() as f64)
            }
        }
        2 => Value::Number(nums.len() as f64),
        3 => {
            // COUNTA — count non-empty (for aggregate, same as count numeric after filtering)
            Value::Number(nums.len() as f64)
        }
        4 => nums
            .iter()
            .copied()
            .reduce(f64::max)
            .map_or(Value::Number(0.0), Value::Number),
        5 => nums
            .iter()
            .copied()
            .reduce(f64::min)
            .map_or(Value::Number(0.0), Value::Number),
        6 => Value::Number(if nums.is_empty() {
            0.0
        } else {
            nums.iter().product()
        }),
        7 => sample_stdev(&nums),
        8 => pop_stdev(&nums),
        9 => Value::Number(nums.iter().sum()),
        10 => sample_var(&nums),
        11 => pop_var(&nums),
        12 => {
            // MEDIAN
            agg_median(&nums)
        }
        13 => {
            // MODE.SNGL
            agg_mode(&nums)
        }
        14 => {
            // LARGE(array, k)
            let k = match k_arg.map(|a| n(std::slice::from_ref(a), 0)) {
                Some(Ok(v)) => v as usize,
                Some(Err(e)) => return Value::Error(e),
                None => return Value::Error(CellError::Value),
            };
            agg_large(&nums, k)
        }
        15 => {
            // SMALL(array, k)
            let k = match k_arg.map(|a| n(std::slice::from_ref(a), 0)) {
                Some(Ok(v)) => v as usize,
                Some(Err(e)) => return Value::Error(e),
                None => return Value::Error(CellError::Value),
            };
            agg_small(&nums, k)
        }
        16 => {
            // PERCENTILE.INC
            let k = match k_arg.map(|a| n(std::slice::from_ref(a), 0)) {
                Some(Ok(v)) => v,
                Some(Err(e)) => return Value::Error(e),
                None => return Value::Error(CellError::Value),
            };
            agg_percentile_inc(&nums, k)
        }
        17 => {
            // QUARTILE.INC
            let k = match k_arg.map(|a| n(std::slice::from_ref(a), 0)) {
                Some(Ok(v)) => v as u32,
                Some(Err(e)) => return Value::Error(e),
                None => return Value::Error(CellError::Value),
            };
            agg_quartile_inc(&nums, k)
        }
        18 => {
            // PERCENTILE.EXC
            let k = match k_arg.map(|a| n(std::slice::from_ref(a), 0)) {
                Some(Ok(v)) => v,
                Some(Err(e)) => return Value::Error(e),
                None => return Value::Error(CellError::Value),
            };
            agg_percentile_exc(&nums, k)
        }
        19 => {
            // QUARTILE.EXC
            let k = match k_arg.map(|a| n(std::slice::from_ref(a), 0)) {
                Some(Ok(v)) => v as u32,
                Some(Err(e)) => return Value::Error(e),
                None => return Value::Error(CellError::Value),
            };
            agg_quartile_exc(&nums, k)
        }
        _ => Value::Error(CellError::Value),
    }
}

fn agg_median(nums: &[f64]) -> Value {
    if nums.is_empty() {
        return Value::Error(CellError::Num);
    }
    let mut s = nums.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let len = s.len();
    let v = if len % 2 == 1 {
        s[len / 2]
    } else {
        f64::midpoint(s[len / 2 - 1], s[len / 2])
    };
    Value::Number(v)
}

fn agg_mode(nums: &[f64]) -> Value {
    if nums.is_empty() {
        return Value::Error(CellError::NA);
    }
    // Use bit representation for frequency map
    let mut counts: Vec<(u64, usize)> = Vec::new();
    for &x in nums {
        let key = x.to_bits();
        if let Some(entry) = counts.iter_mut().find(|(k, _)| *k == key) {
            entry.1 += 1;
        } else {
            counts.push((key, 1));
        }
    }
    let best = counts.iter().max_by_key(|(_, c)| *c).unwrap();
    if best.1 < 2 {
        return Value::Error(CellError::NA);
    }
    Value::Number(f64::from_bits(best.0))
}

fn agg_large(nums: &[f64], k: usize) -> Value {
    if k == 0 || k > nums.len() {
        return Value::Error(CellError::Num);
    }
    let mut s = nums.to_vec();
    s.sort_by(|a, b| b.partial_cmp(a).unwrap()); // descending
    Value::Number(s[k - 1])
}

fn agg_small(nums: &[f64], k: usize) -> Value {
    if k == 0 || k > nums.len() {
        return Value::Error(CellError::Num);
    }
    let mut s = nums.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap()); // ascending
    Value::Number(s[k - 1])
}

fn agg_percentile_inc(nums: &[f64], k: f64) -> Value {
    if !(0.0..=1.0).contains(&k) || nums.is_empty() {
        return Value::Error(CellError::Num);
    }
    let mut s = nums.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = s.len();
    let rank = k * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let frac = rank - lo as f64;
    let v = s[lo] + frac * (s[hi.min(n - 1)] - s[lo]);
    Value::Number(v)
}

fn agg_quartile_inc(nums: &[f64], q: u32) -> Value {
    if q > 4 {
        return Value::Error(CellError::Num);
    }
    agg_percentile_inc(nums, f64::from(q) / 4.0)
}

fn agg_percentile_exc(nums: &[f64], k: f64) -> Value {
    let n = nums.len();
    if n == 0 || !(0.0..1.0).contains(&k) || k <= 0.0 || k >= 1.0 {
        return Value::Error(CellError::Num);
    }
    let mut s = nums.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rank = k * (n + 1) as f64 - 1.0;
    if rank < 0.0 || rank > (n - 1) as f64 {
        return Value::Error(CellError::Num);
    }
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let frac = rank - lo as f64;
    let v = s[lo] + frac * (s[hi.min(n - 1)] - s[lo]);
    Value::Number(v)
}

fn agg_quartile_exc(nums: &[f64], q: u32) -> Value {
    if q == 0 || q > 3 {
        return Value::Error(CellError::Num);
    }
    agg_percentile_exc(nums, f64::from(q) / 4.0)
}

// --- matrix helpers --------------------------------------------------------

/// Extract a 2D f64 matrix from a Value. Returns (rows, cols, data) or an error.
fn extract_matrix(ctx: &mut dyn Context, v: &Value) -> Result<(usize, usize, Vec<f64>), CellError> {
    let arr = match v {
        Value::Array(a) => a.clone(),
        Value::Ref(r) => ctx.ref_to_array(*r),
        other => {
            // scalar → 1×1 matrix
            let x = to_number(other)?;
            return Ok((1, 1, vec![x]));
        }
    };
    let mut data = Vec::with_capacity(arr.rows * arr.cols);
    for v in &arr.data {
        match v {
            Value::Number(x) => data.push(*x),
            Value::Bool(b) => data.push(if *b { 1.0 } else { 0.0 }),
            Value::Empty => data.push(0.0),
            Value::Error(e) => return Err(*e),
            _ => return Err(CellError::Value),
        }
    }
    Ok((arr.rows, arr.cols, data))
}

/// In-place LU decomposition (Gaussian elimination with partial pivoting).
/// Returns the sign of the permutation for determinant computation.
fn lu_decompose(m: &mut [f64], n: usize) -> Result<f64, CellError> {
    let mut sign = 1.0f64;
    for col in 0..n {
        // Find pivot
        let pivot_row = (col..n).max_by(|&a, &b| {
            m[a * n + col]
                .abs()
                .partial_cmp(&m[b * n + col].abs())
                .unwrap()
        });
        let pivot_row = pivot_row.unwrap();
        if m[pivot_row * n + col].abs() < 1e-15 {
            return Err(CellError::Value); // singular
        }
        if pivot_row != col {
            for c in 0..n {
                m.swap(col * n + c, pivot_row * n + c);
            }
            sign = -sign;
        }
        let pivot = m[col * n + col];
        for row in (col + 1)..n {
            let factor = m[row * n + col] / pivot;
            for c in col..n {
                let sub = factor * m[col * n + c];
                m[row * n + c] -= sub;
            }
        }
    }
    Ok(sign)
}

// --- MDETERM ---------------------------------------------------------------

fn mdeterm(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match extract_matrix(ctx, &args[0]) {
        Ok((rows, cols, mut data)) => {
            if rows != cols || rows == 0 {
                return Value::Error(CellError::Value);
            }
            let n = rows;
            let sign = match lu_decompose(&mut data, n) {
                Ok(s) => s,
                Err(_) => return Value::Number(0.0), // singular → det = 0
            };
            let det = sign * (0..n).map(|i| data[i * n + i]).product::<f64>();
            Value::Number(det)
        }
        Err(e) => Value::Error(e),
    }
}

// --- MINVERSE --------------------------------------------------------------

fn minverse(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match extract_matrix(ctx, &args[0]) {
        Ok((rows, cols, data)) => {
            if rows != cols || rows == 0 {
                return Value::Error(CellError::Value);
            }
            let n = rows;
            // Gauss-Jordan elimination on [A | I]
            let mut aug: Vec<f64> = Vec::with_capacity(n * 2 * n);
            for r in 0..n {
                for c in 0..n {
                    aug.push(data[r * n + c]);
                }
                for c in 0..n {
                    aug.push(if r == c { 1.0 } else { 0.0 });
                }
            }
            let w = 2 * n;
            for col in 0..n {
                // Find pivot
                let pivot_row = (col..n)
                    .max_by(|&a, &b| {
                        aug[a * w + col]
                            .abs()
                            .partial_cmp(&aug[b * w + col].abs())
                            .unwrap()
                    })
                    .unwrap();
                if aug[pivot_row * w + col].abs() < 1e-15 {
                    return Value::Error(CellError::Value); // singular
                }
                if pivot_row != col {
                    for c in 0..w {
                        aug.swap(col * w + c, pivot_row * w + c);
                    }
                }
                let pivot = aug[col * w + col];
                for c in 0..w {
                    aug[col * w + c] /= pivot;
                }
                for row in 0..n {
                    if row == col {
                        continue;
                    }
                    let factor = aug[row * w + col];
                    for c in 0..w {
                        let sub = factor * aug[col * w + c];
                        aug[row * w + c] -= sub;
                    }
                }
            }
            // Extract right half
            let mut result = Vec::with_capacity(n * n);
            for r in 0..n {
                for c in n..(2 * n) {
                    result.push(Value::Number(aug[r * w + c]));
                }
            }
            Value::Array(crate::formula::value::Array::new(n, n, result))
        }
        Err(e) => Value::Error(e),
    }
}

// --- MMULT -----------------------------------------------------------------

fn mmult(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (ar, ac, a) = match extract_matrix(ctx, &args[0]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let (br, bc, b) = match extract_matrix(ctx, &args[1]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if ac != br {
        return Value::Error(CellError::Value);
    }
    let mut result = Vec::with_capacity(ar * bc);
    for r in 0..ar {
        for c in 0..bc {
            let mut s = 0.0;
            for k in 0..ac {
                s += a[r * ac + k] * b[k * bc + c];
            }
            result.push(Value::Number(s));
        }
    }
    Value::Array(crate::formula::value::Array::new(ar, bc, result))
}

// --- MUNIT -----------------------------------------------------------------

fn munit(_: &mut dyn Context, args: &[Value]) -> Value {
    let size = match n(args, 0) {
        Ok(v) => v as usize,
        Err(e) => return Value::Error(e),
    };
    if size == 0 {
        return Value::Error(CellError::Value);
    }
    let mut data = Vec::with_capacity(size * size);
    for r in 0..size {
        for c in 0..size {
            data.push(Value::Number(if r == c { 1.0 } else { 0.0 }));
        }
    }
    Value::Array(crate::formula::value::Array::new(size, size, data))
}

// --- GAMMA -----------------------------------------------------------------

/// Γ(x) via `exp(ln_gamma)` with proper sign.
/// PARITY: #NUM! for non-positive integers.
fn gamma(_: &mut dyn Context, args: &[Value]) -> Value {
    match n(args, 0) {
        Ok(x) => {
            // Non-positive integers → #NUM!
            if x <= 0.0 && x == x.trunc() {
                return Value::Error(CellError::Num);
            }
            // Use reflection for negative x: Γ(x) = π / (sin(πx) * Γ(1-x))
            if x < 0.5 {
                let pi = std::f64::consts::PI;
                let sin_val = (pi * x).sin();
                if sin_val.abs() < 1e-15 {
                    return Value::Error(CellError::Num);
                }
                let g = pi / (sin_val * ln_gamma(1.0 - x).exp());
                if g.is_finite() {
                    Value::Number(g)
                } else {
                    Value::Error(CellError::Num)
                }
            } else {
                let g = ln_gamma(x).exp();
                if g.is_finite() {
                    Value::Number(g)
                } else {
                    Value::Error(CellError::Num)
                }
            }
        }
        Err(e) => Value::Error(e),
    }
}

/// Lanczos approximation of ln(Γ(x)).
#[must_use]
pub fn ln_gamma(x: f64) -> f64 {
    const G: f64 = 7.0;
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        let pi = std::f64::consts::PI;
        (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x)
    } else {
        let x = x - 1.0;
        let mut a = C[0];
        let t = x + G + 0.5;
        for (i, &c) in C.iter().enumerate().skip(1) {
            a += c / (x + i as f64);
        }
        0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
    }
}

#[cfg(test)]
#[path = "../math_tests/tests.rs"]
mod tests;
