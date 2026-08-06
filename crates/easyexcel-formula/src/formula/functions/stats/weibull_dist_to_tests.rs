fn weibull_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let alpha = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let beta = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 3) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if x < 0.0 || alpha <= 0.0 || beta <= 0.0 {
        return Value::Error(CellError::Num);
    }
    let ratio = x / beta;
    if cumulative {
        Value::Number(1.0 - (-(ratio.powf(alpha))).exp())
    } else {
        Value::Number(alpha / beta * ratio.powf(alpha - 1.0) * (-(ratio.powf(alpha))).exp())
    }
}

// ---------------------------------------------------------------------------
// CONFIDENCE.T
// ---------------------------------------------------------------------------

fn confidence_t(_: &mut dyn Context, args: &[Value]) -> Value {
    let alpha = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let sd = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let size = match num(args, 2) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if alpha <= 0.0 || alpha >= 1.0 || sd <= 0.0 || size < 1.0 {
        return Value::Error(CellError::Num);
    }
    if size == 1.0 {
        return Value::Error(CellError::Div0);
    }
    let df = size - 1.0;
    // two-tailed t critical value.
    let t = t_inv_left(1.0 - alpha / 2.0, df);
    Value::Number(t * sd / size.sqrt())
}

// ---------------------------------------------------------------------------
// Z.TEST
// ---------------------------------------------------------------------------

fn z_test(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let data = match collect_numbers(ctx, &args[..1], false) {
        Ok(d) => d,
        Err(e) => return Value::Error(e),
    };
    let n = data.len();
    if n == 0 {
        return Value::Error(CellError::Num);
    }
    let mean = data.iter().sum::<f64>() / n as f64;
    let sigma = if args.len() >= 3 {
        match num(args, 2) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        match variance(&data, false) {
            Ok(v) => v.sqrt(),
            Err(e) => return Value::Error(e),
        }
    };
    if sigma <= 0.0 {
        return Value::Error(CellError::Num);
    }
    // Z.TEST = 1 - NORM.S.DIST((mean - x) / (sigma/sqrt(n)))
    let z = (mean - x) / (sigma / (n as f64).sqrt());
    Value::Number(1.0 - norm_cdf(z))
}

// ---------------------------------------------------------------------------
// PROB
// ---------------------------------------------------------------------------

fn prob(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let xs = match collect_numbers(ctx, &args[..1], false) {
        Ok(d) => d,
        Err(e) => return Value::Error(e),
    };
    let ps = match collect_numbers(ctx, &args[1..2], false) {
        Ok(d) => d,
        Err(e) => return Value::Error(e),
    };
    if xs.len() != ps.len() || xs.is_empty() {
        return Value::Error(CellError::NA);
    }
    let total: f64 = ps.iter().sum();
    if (total - 1.0).abs() > 1e-7 {
        return Value::Error(CellError::Num);
    }
    if ps.iter().any(|&p| !(0.0..=1.0).contains(&p)) {
        return Value::Error(CellError::Num);
    }
    let lower = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let upper = if args.len() >= 4 {
        match num(args, 3) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        lower
    };
    let (lo, hi) = if lower <= upper {
        (lower, upper)
    } else {
        (upper, lower)
    };
    let sum: f64 = xs
        .iter()
        .zip(ps.iter())
        .filter(|(x, _)| **x >= lo && **x <= hi)
        .map(|(_, p)| *p)
        .sum();
    Value::Number(sum)
}

// ---------------------------------------------------------------------------
// SKEW.P / STEYX
// ---------------------------------------------------------------------------

fn skew_p(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            let n = ns.len();
            if n < 1 {
                return Value::Error(CellError::Div0);
            }
            let nf = n as f64;
            let mean = ns.iter().sum::<f64>() / nf;
            // population std dev.
            let var = ns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nf;
            let s = var.sqrt();
            if s == 0.0 {
                return Value::Error(CellError::Div0);
            }
            let m3: f64 = ns.iter().map(|x| ((x - mean) / s).powi(3)).sum::<f64>() / nf;
            Value::Number(m3)
        }
        Err(e) => Value::Error(e),
    }
}

fn steyx(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // STEYX(known_ys, known_xs)
    match two_arrays(ctx, args) {
        Ok((ys, xs)) => {
            let n = xs.len();
            if n < 3 {
                return Value::Error(CellError::Div0);
            }
            let (mx, my) = means(&xs, &ys);
            let sxx: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
            let syy: f64 = ys.iter().map(|y| (y - my) * (y - my)).sum();
            let sxy: f64 = xs
                .iter()
                .zip(ys.iter())
                .map(|(x, y)| (x - mx) * (y - my))
                .sum();
            if sxx == 0.0 {
                return Value::Error(CellError::Div0);
            }
            let nf = n as f64;
            // STEYX = sqrt( (1/(n-2)) * (Syy - Sxy^2 / Sxx) )
            let val = (syy - sxy * sxy / sxx) / (nf - 2.0);
            Value::Number(val.max(0.0).sqrt())
        }
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// FREQUENCY
// ---------------------------------------------------------------------------

fn frequency(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let data = match collect_numbers(ctx, &args[..1], false) {
        Ok(d) => d,
        Err(e) => return Value::Error(e),
    };
    let mut bins = match collect_numbers(ctx, &args[1..2], false) {
        Ok(d) => d,
        Err(e) => return Value::Error(e),
    };
    bins.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // counts has bins.len()+1 entries: one per bin plus an overflow bin.
    let mut counts = vec![0u64; bins.len() + 1];
    for &v in &data {
        let mut placed = false;
        for (i, &b) in bins.iter().enumerate() {
            if v <= b {
                counts[i] += 1;
                placed = true;
                break;
            }
        }
        if !placed {
            *counts.last_mut().unwrap() += 1;
        }
    }
    let out: Vec<Value> = counts.iter().map(|&c| Value::Number(c as f64)).collect();
    let rows = out.len();
    Value::Array(crate::formula::value::Array::new(rows, 1, out))
}

// ---------------------------------------------------------------------------
// TREND / GROWTH / LINEST / LOGEST (simple single-variable linear case)
// ---------------------------------------------------------------------------

/// Fit y = b*x + a from parallel arrays; returns (slope, intercept).
fn simple_linear_fit(xs: &[f64], ys: &[f64]) -> Option<(f64, f64)> {
    let n = xs.len();
    if n < 2 || n != ys.len() {
        return None;
    }
    let (mx, my) = means(xs, ys);
    let sxx: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
    if sxx == 0.0 {
        return None;
    }
    let sxy: f64 = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| (x - mx) * (y - my))
        .sum();
    let b = sxy / sxx;
    Some((b, my - b * mx))
}

/// Collect numbers from a single arg, or default to 1..=len if Empty/omitted.
fn xs_or_default(ctx: &mut dyn Context, arg: Option<&Value>, len: usize) -> Option<Vec<f64>> {
    match arg {
        None => Some((1..=len).map(|i| i as f64).collect()),
        Some(Value::Empty) => Some((1..=len).map(|i| i as f64).collect()),
        Some(v) => collect_numbers(ctx, std::slice::from_ref(v), false).ok(),
    }
}

fn trend(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // TREND(known_ys, [known_xs], [new_xs], [const])
    let ys = match collect_numbers(ctx, &args[..1], false) {
        Ok(d) => d,
        Err(e) => return Value::Error(e),
    };
    let xs = match xs_or_default(ctx, args.get(1), ys.len()) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    if xs.len() != ys.len() {
        return Value::Error(CellError::Ref);
    }
    let (b, a) = match simple_linear_fit(&xs, &ys) {
        Some(v) => v,
        None => return Value::Error(CellError::Div0),
    };
    let new_xs = match args.get(2) {
        Some(Value::Empty) | None => xs.clone(),
        Some(v) => match collect_numbers(ctx, std::slice::from_ref(v), false) {
            Ok(d) => d,
            Err(e) => return Value::Error(e),
        },
    };
    let out: Vec<Value> = new_xs.iter().map(|x| Value::Number(b * x + a)).collect();
    let rows = out.len();
    Value::Array(crate::formula::value::Array::new(rows, 1, out))
}

fn growth(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // GROWTH: exponential y = a * b^x → fit ln(y) linearly.
    let ys = match collect_numbers(ctx, &args[..1], false) {
        Ok(d) => d,
        Err(e) => return Value::Error(e),
    };
    if ys.iter().any(|&y| y <= 0.0) {
        return Value::Error(CellError::Num);
    }
    let ln_ys: Vec<f64> = ys.iter().map(|y| y.ln()).collect();
    let xs = match xs_or_default(ctx, args.get(1), ys.len()) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    if xs.len() != ys.len() {
        return Value::Error(CellError::Ref);
    }
    let (b, a) = match simple_linear_fit(&xs, &ln_ys) {
        Some(v) => v,
        None => return Value::Error(CellError::Div0),
    };
    let new_xs = match args.get(2) {
        Some(Value::Empty) | None => xs.clone(),
        Some(v) => match collect_numbers(ctx, std::slice::from_ref(v), false) {
            Ok(d) => d,
            Err(e) => return Value::Error(e),
        },
    };
    let out: Vec<Value> = new_xs
        .iter()
        .map(|x| Value::Number((b * x + a).exp()))
        .collect();
    let rows = out.len();
    Value::Array(crate::formula::value::Array::new(rows, 1, out))
}

fn linest(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // PARITY: only the simple single-x case is implemented; returns the 1x2
    // array {slope, intercept}. Full statistics matrix is not produced.
    let ys = match collect_numbers(ctx, &args[..1], false) {
        Ok(d) => d,
        Err(e) => return Value::Error(e),
    };
    let xs = match xs_or_default(ctx, args.get(1), ys.len()) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    if xs.len() != ys.len() {
        return Value::Error(CellError::Ref);
    }
    let (b, a) = match simple_linear_fit(&xs, &ys) {
        Some(v) => v,
        None => return Value::Error(CellError::Div0),
    };
    Value::Array(crate::formula::value::Array::new(
        1,
        2,
        vec![Value::Number(b), Value::Number(a)],
    ))
}

fn logest(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // PARITY: simple single-x exponential fit y = a*b^x; returns {b, a}.
    let ys = match collect_numbers(ctx, &args[..1], false) {
        Ok(d) => d,
        Err(e) => return Value::Error(e),
    };
    if ys.iter().any(|&y| y <= 0.0) {
        return Value::Error(CellError::Num);
    }
    let ln_ys: Vec<f64> = ys.iter().map(|y| y.ln()).collect();
    let xs = match xs_or_default(ctx, args.get(1), ys.len()) {
        Some(d) => d,
        None => return Value::Error(CellError::Value),
    };
    if xs.len() != ys.len() {
        return Value::Error(CellError::Ref);
    }
    let (slope, intercept) = match simple_linear_fit(&xs, &ln_ys) {
        Some(v) => v,
        None => return Value::Error(CellError::Div0),
    };
    Value::Array(crate::formula::value::Array::new(
        1,
        2,
        vec![Value::Number(slope.exp()), Value::Number(intercept.exp())],
    ))
}

/// PARITY: ETS (Exponential Triple Smoothing / AAA Holt-Winters) is not
/// implemented; these return #N/A.
fn forecast_ets_na(_: &mut dyn Context, _args: &[Value]) -> Value {
    Value::Error(CellError::NA)
}

// ---------------------------------------------------------------------------
// Hypothesis tests
// ---------------------------------------------------------------------------

/// Materialize an argument into a numeric matrix `(rows, cols, data)` in
/// row-major order. Non-numeric cells become NaN.
fn numeric_matrix(ctx: &mut dyn Context, v: &Value) -> (usize, usize, Vec<f64>) {
    let arr = match v {
        Value::Ref(r) => ctx.ref_to_array(*r),
        Value::Array(a) => a.clone(),
        other => crate::formula::value::Array::scalar(other.clone()),
    };
    let data: Vec<f64> = arr
        .data
        .iter()
        .map(|c| match c {
            Value::Number(n) => *n,
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            _ => f64::NAN,
        })
        .collect();
    (arr.rows, arr.cols, data)
}

/// One-tailed upper survival function P(X >= x) for the F distribution.
fn f_sf(x: f64, d1: f64, d2: f64) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    regularized_incomplete_beta(d2 / (d2 + d1 * x), d2 / 2.0, d1 / 2.0)
}

/// One-tailed upper survival function P(T >= t) for Student's t (t >= 0).
fn t_sf(t: f64, df: f64) -> f64 {
    let t = t.abs();
    0.5 * regularized_incomplete_beta(df / (df + t * t), df / 2.0, 0.5)
}

/// CHISQ.TEST — chi-squared test of independence between an observed and an
/// expected range. Returns the right-tail p-value.
fn chisq_test(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let (ar, ac, actual) = numeric_matrix(ctx, &args[0]);
    let (er, ec, expected) = numeric_matrix(ctx, &args[1]);
    if ar != er || ac != ec || actual.is_empty() {
        return Value::Error(CellError::NA);
    }
    let mut chi = 0.0;
    let mut count = 0usize;
    for (a, e) in actual.iter().zip(expected.iter()) {
        if a.is_nan() || e.is_nan() {
            continue;
        }
        if *e == 0.0 {
            return Value::Error(CellError::Div0);
        }
        chi += (a - e) * (a - e) / e;
        count += 1;
    }
    if count == 0 {
        return Value::Error(CellError::NA);
    }
    // Degrees of freedom: (r-1)(c-1) for a 2D table, else n-1 for a vector.
    let df = if ar > 1 && ac > 1 {
        ((ar - 1) * (ac - 1)) as f64
    } else {
        (count - 1) as f64
    };
    if df <= 0.0 {
        return Value::Error(CellError::Div0);
    }
    Value::Number(1.0 - chisq_cdf(chi, df))
}

/// F.TEST — two-tailed probability that two samples' variances are equal.
fn f_test(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let x = match collect_a(ctx, &args[0..1]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let y = match collect_a(ctx, &args[1..2]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if x.len() < 2 || y.len() < 2 {
        return Value::Error(CellError::Div0);
    }
    let v1 = match variance(&x, false) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let v2 = match variance(&y, false) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if v1 == 0.0 || v2 == 0.0 {
        return Value::Error(CellError::Div0);
    }
    let f = v1 / v2;
    let df1 = (x.len() - 1) as f64;
    let df2 = (y.len() - 1) as f64;
    let mut p1 = f_sf(f, df1, df2);
    if p1 > 0.5 {
        p1 = 1.0 - p1;
    }
    Value::Number((2.0 * p1).min(1.0))
}

/// T.TEST — Student's t-test. `tails` ∈ {1,2}; `kind` ∈ {1 paired, 2 two-sample
/// equal variance, 3 two-sample unequal variance (Welch)}.
fn t_test(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let x = match collect_a(ctx, &args[0..1]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let y = match collect_a(ctx, &args[1..2]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let tails = match to_number(&args[2]) {
        Ok(n) => n.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let kind = match to_number(&args[3]) {
        Ok(n) => n.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    if tails != 1 && tails != 2 {
        return Value::Error(CellError::Num);
    }
    let (t, df) = match kind {
        1 => {
            // Paired: requires equal-length samples.
            if x.len() != y.len() || x.len() < 2 {
                return Value::Error(CellError::NA);
            }
            let diffs: Vec<f64> = x.iter().zip(y.iter()).map(|(a, b)| a - b).collect();
            let n = diffs.len() as f64;
            let md = diffs.iter().sum::<f64>() / n;
            let vd = match variance(&diffs, false) {
                Ok(v) => v,
                Err(e) => return Value::Error(e),
            };
            if vd == 0.0 {
                return Value::Error(CellError::Div0);
            }
            (md / (vd / n).sqrt(), n - 1.0)
        }
        2 => {
            // Two-sample, equal variance (pooled).
            let (n1, n2) = (x.len() as f64, y.len() as f64);
            if n1 < 2.0 || n2 < 2.0 {
                return Value::Error(CellError::Div0);
            }
            let (m1, m2) = (x.iter().sum::<f64>() / n1, y.iter().sum::<f64>() / n2);
            let v1 = variance(&x, false).unwrap_or(0.0);
            let v2 = variance(&y, false).unwrap_or(0.0);
            let sp = ((n1 - 1.0) * v1 + (n2 - 1.0) * v2) / (n1 + n2 - 2.0);
            let se = (sp * (1.0 / n1 + 1.0 / n2)).sqrt();
            if se == 0.0 {
                return Value::Error(CellError::Div0);
            }
            ((m1 - m2) / se, n1 + n2 - 2.0)
        }
        3 => {
            // Two-sample, unequal variance (Welch).
            let (n1, n2) = (x.len() as f64, y.len() as f64);
            if n1 < 2.0 || n2 < 2.0 {
                return Value::Error(CellError::Div0);
            }
            let (m1, m2) = (x.iter().sum::<f64>() / n1, y.iter().sum::<f64>() / n2);
            let v1 = variance(&x, false).unwrap_or(0.0);
            let v2 = variance(&y, false).unwrap_or(0.0);
            let s = v1 / n1 + v2 / n2;
            if s == 0.0 {
                return Value::Error(CellError::Div0);
            }
            let df = s * s / ((v1 / n1).powi(2) / (n1 - 1.0) + (v2 / n2).powi(2) / (n2 - 1.0));
            ((m1 - m2) / s.sqrt(), df)
        }
        _ => return Value::Error(CellError::Num),
    };
    Value::Number((tails as f64 * t_sf(t, df)).min(1.0))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "../stats_tests/tests.rs"]
mod tests;
