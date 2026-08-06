/// MODE.MULT — all most-frequent values as a vertical (spilled) array.
fn mode_mult(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let ns = match collect_numbers(ctx, args, false) {
        Ok(ns) => ns,
        Err(e) => return Value::Error(e),
    };
    if ns.is_empty() {
        return Value::Error(CellError::NA);
    }
    // Highest frequency across all values.
    let freq = |x: f64| ns.iter().filter(|&&y| y == x).count();
    let best = ns.iter().map(|&x| freq(x)).max().unwrap_or(0);
    if best < 2 {
        return Value::Error(CellError::NA);
    }
    // All values at that frequency, in first-appearance order, de-duplicated.
    let mut seen: Vec<f64> = Vec::new();
    let mut out: Vec<Value> = Vec::new();
    for &x in &ns {
        if freq(x) == best && !seen.contains(&x) {
            seen.push(x);
            out.push(Value::Number(x));
        }
    }
    let n = out.len();
    Value::Array(Array::new(n, 1, out))
}

fn mode_sngl(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.is_empty() {
                return Value::Error(CellError::NA);
            }
            // Find the value that appears most often (first one in case of tie).
            let mut best_val = ns[0];
            let mut best_cnt = 0usize;
            // Simple O(n^2) — acceptable for worksheet data sizes.
            for &x in &ns {
                let cnt = ns.iter().filter(|&&y| y == x).count();
                if cnt > best_cnt {
                    best_cnt = cnt;
                    best_val = x;
                }
            }
            if best_cnt < 2 {
                Value::Error(CellError::NA)
            } else {
                Value::Number(best_val)
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn large(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let k = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match collect_numbers(ctx, &args[..1], false) {
        Ok(mut ns) => {
            let k = k.trunc() as usize;
            if k == 0 || k > ns.len() {
                return Value::Error(CellError::Num);
            }
            ns.sort_by(|a, b| b.partial_cmp(a).unwrap()); // descending
            Value::Number(ns[k - 1])
        }
        Err(e) => Value::Error(e),
    }
}

fn small(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let k = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match collect_numbers(ctx, &args[..1], false) {
        Ok(mut ns) => {
            let k = k.trunc() as usize;
            if k == 0 || k > ns.len() {
                return Value::Error(CellError::Num);
            }
            ns.sort_by(|a, b| a.partial_cmp(b).unwrap()); // ascending
            Value::Number(ns[k - 1])
        }
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// Rank
// ---------------------------------------------------------------------------

fn get_rank_order(args: &[Value]) -> bool {
    // order arg: 0 or omitted → descending (largest=1), non-zero → ascending
    if args.len() >= 3 {
        match to_number(&args[2]) {
            Ok(v) => v != 0.0,
            Err(_) => false,
        }
    } else {
        false
    }
}

fn rank_eq(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let ascending = get_rank_order(args);
    match collect_numbers(ctx, &args[1..2], false) {
        Ok(ns) => {
            if !ns.contains(&x) {
                return Value::Error(CellError::NA);
            }
            let rank = if ascending {
                ns.iter().filter(|&&y| y < x).count() + 1
            } else {
                ns.iter().filter(|&&y| y > x).count() + 1
            };
            Value::Number(rank as f64)
        }
        Err(e) => Value::Error(e),
    }
}

fn rank_avg(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let ascending = get_rank_order(args);
    match collect_numbers(ctx, &args[1..2], false) {
        Ok(ns) => {
            if !ns.contains(&x) {
                return Value::Error(CellError::NA);
            }
            let eq_count = ns.iter().filter(|&&y| y == x).count();
            let rank_first = if ascending {
                ns.iter().filter(|&&y| y < x).count() + 1
            } else {
                ns.iter().filter(|&&y| y > x).count() + 1
            };
            let avg_rank = rank_first as f64 + (eq_count - 1) as f64 / 2.0;
            Value::Number(avg_rank)
        }
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// Variance / Standard deviation
// ---------------------------------------------------------------------------

fn var_s(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => match variance(&ns, false) {
            Ok(v) => Value::Number(v),
            Err(e) => Value::Error(e),
        },
        Err(e) => Value::Error(e),
    }
}

fn var_p(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => match variance(&ns, true) {
            Ok(v) => Value::Number(v),
            Err(e) => Value::Error(e),
        },
        Err(e) => Value::Error(e),
    }
}

fn vara(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_a(ctx, args) {
        Ok(ns) => match variance(&ns, false) {
            Ok(v) => Value::Number(v),
            Err(e) => Value::Error(e),
        },
        Err(e) => Value::Error(e),
    }
}

fn varpa(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_a(ctx, args) {
        Ok(ns) => match variance(&ns, true) {
            Ok(v) => Value::Number(v),
            Err(e) => Value::Error(e),
        },
        Err(e) => Value::Error(e),
    }
}

fn stdev_s(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match var_s(ctx, args) {
        Value::Number(v) => Value::Number(v.sqrt()),
        other => other,
    }
}

fn stdev_p(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match var_p(ctx, args) {
        Value::Number(v) => Value::Number(v.sqrt()),
        other => other,
    }
}

fn stdeva(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match vara(ctx, args) {
        Value::Number(v) => Value::Number(v.sqrt()),
        other => other,
    }
}

fn stdevpa(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match varpa(ctx, args) {
        Value::Number(v) => Value::Number(v.sqrt()),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Percentile / Quartile
// ---------------------------------------------------------------------------

/// PERCENTILE.INC: rank = p*(n-1), linear interpolation.
fn percentile_inc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if !(0.0..=1.0).contains(&p) {
        return Value::Error(CellError::Num);
    }
    match collect_numbers(ctx, &args[..1], false) {
        Ok(mut ns) => {
            if ns.is_empty() {
                return Value::Error(CellError::Num);
            }
            ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = ns.len();
            if n == 1 {
                return Value::Number(ns[0]);
            }
            let rank = p * (n - 1) as f64;
            let lo = rank.floor() as usize;
            let hi = rank.ceil() as usize;
            if lo == hi {
                Value::Number(ns[lo])
            } else {
                let frac = rank - lo as f64;
                Value::Number(ns[lo] + frac * (ns[hi] - ns[lo]))
            }
        }
        Err(e) => Value::Error(e),
    }
}

/// PERCENTILE.EXC: rank = p*(n+1)-1, 1/(n+1) < p < n/(n+1).
fn percentile_exc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    match collect_numbers(ctx, &args[..1], false) {
        Ok(mut ns) => {
            let n = ns.len();
            if n == 0 {
                return Value::Error(CellError::Num);
            }
            let lo_p = 1.0 / (n + 1) as f64;
            let hi_p = n as f64 / (n + 1) as f64;
            if p <= 0.0 || p >= 1.0 || p < lo_p || p > hi_p {
                return Value::Error(CellError::Num);
            }
            ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let rank = p * (n + 1) as f64 - 1.0;
            let lo = rank.floor() as usize;
            let hi = (lo + 1).min(n - 1);
            let frac = rank - lo as f64;
            Value::Number(ns[lo] + frac * (ns[hi] - ns[lo]))
        }
        Err(e) => Value::Error(e),
    }
}

fn quartile_inc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let q = match num(args, 1) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let p = match q {
        0 => 0.0,
        1 => 0.25,
        2 => 0.5,
        3 => 0.75,
        4 => 1.0,
        _ => return Value::Error(CellError::Num),
    };
    // Reuse percentile_inc
    percentile_inc(ctx, &[args[0].clone(), Value::Number(p)])
}

fn quartile_exc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let q = match num(args, 1) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let p = match q {
        1 => 0.25,
        2 => 0.5,
        3 => 0.75,
        _ => return Value::Error(CellError::Num),
    };
    percentile_exc(ctx, &[args[0].clone(), Value::Number(p)])
}

/// PERCENTRANK.INC: returns the rank of x within array as a fraction [0,1].
fn percentrank_inc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let significance = if args.len() >= 3 {
        match num(args, 2) {
            Ok(v) => v.trunc() as usize,
            Err(e) => return Value::Error(e),
        }
    } else {
        3
    };
    match collect_numbers(ctx, &args[..1], false) {
        Ok(mut ns) => {
            if ns.is_empty() {
                return Value::Error(CellError::NA);
            }
            ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = ns.len();
            if x < ns[0] || x > ns[n - 1] {
                return Value::Error(CellError::NA);
            }
            // linear interpolation
            let lo = ns.iter().filter(|&&y| y <= x).count() - 1;
            let hi = ns.iter().position(|&y| y >= x).unwrap_or(n - 1);
            let rank = if lo == hi {
                lo as f64 / (n - 1) as f64
            } else {
                let frac = (x - ns[lo]) / (ns[hi] - ns[lo]);
                (lo as f64 + frac) / (n - 1) as f64
            };
            let factor = 10f64.powi(significance as i32);
            Value::Number((rank * factor).floor() / factor)
        }
        Err(e) => Value::Error(e),
    }
}

/// PERCENTRANK.EXC: rank is (position+1)/(n+1) style.
fn percentrank_exc(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let significance = if args.len() >= 3 {
        match num(args, 2) {
            Ok(v) => v.trunc() as usize,
            Err(e) => return Value::Error(e),
        }
    } else {
        3
    };
    match collect_numbers(ctx, &args[..1], false) {
        Ok(mut ns) => {
            if ns.is_empty() {
                return Value::Error(CellError::NA);
            }
            ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = ns.len();
            if x <= ns[0] || x >= ns[n - 1] {
                return Value::Error(CellError::NA);
            }
            let lo_count = ns.iter().filter(|&&y| y < x).count();
            let ge_count = ns.iter().filter(|&&y| y >= x).count();
            // interpolate between lo and hi positions
            let lo_rank = lo_count as f64 / (n + 1) as f64;
            let hi_rank = (lo_count + 1) as f64 / (n + 1) as f64;
            let rank = if ge_count > 0 && ns[lo_count] != x {
                let frac = (x - ns[lo_count - 1]) / (ns[lo_count] - ns[lo_count - 1]);
                lo_rank + frac * (hi_rank - lo_rank)
            } else {
                lo_rank
            };
            let factor = 10f64.powi(significance as i32);
            Value::Number((rank * factor).floor() / factor)
        }
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// Correlation / Covariance / Regression
// ---------------------------------------------------------------------------

/// Collect two parallel numeric arrays from two range-valued args.
fn two_arrays(ctx: &mut dyn Context, args: &[Value]) -> Result<(Vec<f64>, Vec<f64>), CellError> {
    let xs: Vec<Value> = ctx.flatten(&args[0]);
    let ys: Vec<Value> = ctx.flatten(&args[1]);
    if xs.len() != ys.len() {
        return Err(CellError::NA);
    }
    let mut xv = Vec::new();
    let mut yv = Vec::new();
    for (x, y) in xs.iter().zip(ys.iter()) {
        match (x, y) {
            (Value::Number(a), Value::Number(b)) => {
                xv.push(*a);
                yv.push(*b);
            }
            (Value::Error(e), _) | (_, Value::Error(e)) => return Err(*e),
            _ => {} // skip non-numeric pairs
        }
    }
    if xv.is_empty() {
        return Err(CellError::Div0);
    }
    Ok((xv, yv))
}

fn means(xs: &[f64], ys: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n)
}

fn correl(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match two_arrays(ctx, args) {
        Ok((xs, ys)) => {
            let n = xs.len();
            if n < 2 {
                return Value::Error(CellError::Div0);
            }
            let (mx, my) = means(&xs, &ys);
            let cov: f64 = xs
                .iter()
                .zip(ys.iter())
                .map(|(x, y)| (x - mx) * (y - my))
                .sum();
            let sx: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum::<f64>().sqrt();
            let sy: f64 = ys.iter().map(|y| (y - my) * (y - my)).sum::<f64>().sqrt();
            if sx == 0.0 || sy == 0.0 {
                Value::Error(CellError::Div0)
            } else {
                Value::Number(cov / (sx * sy))
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn covariance_p(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match two_arrays(ctx, args) {
        Ok((xs, ys)) => {
            let n = xs.len() as f64;
            if n == 0.0 {
                return Value::Error(CellError::Div0);
            }
            let (mx, my) = means(&xs, &ys);
            let cov: f64 = xs
                .iter()
                .zip(ys.iter())
                .map(|(x, y)| (x - mx) * (y - my))
                .sum::<f64>()
                / n;
            Value::Number(cov)
        }
        Err(e) => Value::Error(e),
    }
}

fn covariance_s(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match two_arrays(ctx, args) {
        Ok((xs, ys)) => {
            let n = xs.len();
            if n < 2 {
                return Value::Error(CellError::Div0);
            }
            let (mx, my) = means(&xs, &ys);
            let cov: f64 = xs
                .iter()
                .zip(ys.iter())
                .map(|(x, y)| (x - mx) * (y - my))
                .sum::<f64>()
                / (n - 1) as f64;
            Value::Number(cov)
        }
        Err(e) => Value::Error(e),
    }
}

fn slope(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // SLOPE(known_ys, known_xs)
    match two_arrays(ctx, args) {
        Ok((ys, xs)) => {
            let n = xs.len();
            if n < 2 {
                return Value::Error(CellError::Div0);
            }
            let (mx, my) = means(&xs, &ys);
            let sxx: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
            let sxy: f64 = xs
                .iter()
                .zip(ys.iter())
                .map(|(x, y)| (x - mx) * (y - my))
                .sum();
            if sxx == 0.0 {
                Value::Error(CellError::Div0)
            } else {
                Value::Number(sxy / sxx)
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn intercept(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // INTERCEPT(known_ys, known_xs)
    match two_arrays(ctx, args) {
        Ok((ys, xs)) => {
            let (mx, my) = means(&xs, &ys);
            let sxx: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
            let sxy: f64 = xs
                .iter()
                .zip(ys.iter())
                .map(|(x, y)| (x - mx) * (y - my))
                .sum();
            if sxx == 0.0 {
                Value::Error(CellError::Div0)
            } else {
                let b = sxy / sxx;
                Value::Number(my - b * mx)
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn rsq(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match correl(ctx, args) {
        Value::Number(r) => Value::Number(r * r),
        other => other,
    }
}

fn forecast_linear(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // FORECAST.LINEAR(x, known_ys, known_xs)
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let slope_val = match two_arrays(ctx, &args[1..]) {
        Ok((ys, xs)) => {
            let (mx, my) = means(&xs, &ys);
            let sxx: f64 = xs.iter().map(|xi| (xi - mx) * (xi - mx)).sum();
            let sxy: f64 = xs
                .iter()
                .zip(ys.iter())
                .map(|(xi, y)| (xi - mx) * (y - my))
                .sum();
            if sxx == 0.0 {
                return Value::Error(CellError::Div0);
            }
            let b = sxy / sxx;
            let a = my - b * mx;
            (b, a)
        }
        Err(e) => return Value::Error(e),
    };
    Value::Number(slope_val.0 * x + slope_val.1)
}

// ---------------------------------------------------------------------------
// Descriptive statistics
// ---------------------------------------------------------------------------

fn devsq(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.is_empty() {
                return Value::Number(0.0);
            }
            let mean = ns.iter().sum::<f64>() / ns.len() as f64;
            let ss: f64 = ns.iter().map(|x| (x - mean) * (x - mean)).sum();
            Value::Number(ss)
        }
        Err(e) => Value::Error(e),
    }
}

fn avedev(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.is_empty() {
                return Value::Error(CellError::Div0);
            }
            let mean = ns.iter().sum::<f64>() / ns.len() as f64;
            let ad: f64 = ns.iter().map(|x| (x - mean).abs()).sum::<f64>() / ns.len() as f64;
            Value::Number(ad)
        }
        Err(e) => Value::Error(e),
    }
}

fn geomean(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.is_empty() {
                return Value::Error(CellError::Num);
            }
            if ns.iter().any(|&x| x <= 0.0) {
                return Value::Error(CellError::Num);
            }
            let log_sum: f64 = ns.iter().map(|x| x.ln()).sum();
            Value::Number((log_sum / ns.len() as f64).exp())
        }
        Err(e) => Value::Error(e),
    }
}

fn harmean(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.is_empty() {
                return Value::Error(CellError::Num);
            }
            if ns.iter().any(|&x| x <= 0.0) {
                return Value::Error(CellError::Num);
            }
            let recip_sum: f64 = ns.iter().map(|x| 1.0 / x).sum();
            Value::Number(ns.len() as f64 / recip_sum)
        }
        Err(e) => Value::Error(e),
    }
}

