/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn register(r: &mut Registry) {
    // Averages
    r.add("AVERAGE", 1, VARIADIC, false, average);
    r.add("AVERAGEA", 1, VARIADIC, false, averagea);
    r.add("AVERAGEIF", 2, 3, false, averageif);
    r.add("AVERAGEIFS", 3, VARIADIC, false, averageifs);

    // Counts
    r.add("COUNT", 1, VARIADIC, false, count);
    r.add("COUNTA", 1, VARIADIC, false, counta);
    r.add("COUNTBLANK", 1, 1, false, countblank);
    r.add("COUNTIF", 2, 2, false, countif);
    r.add("COUNTIFS", 2, VARIADIC, false, countifs);

    // Min / Max
    r.add("MAX", 1, VARIADIC, false, max);
    r.add("MAXA", 1, VARIADIC, false, maxa);
    r.add("MIN", 1, VARIADIC, false, min);
    r.add("MINA", 1, VARIADIC, false, mina);
    r.add("MAXIFS", 3, VARIADIC, false, maxifs);
    r.add("MINIFS", 3, VARIADIC, false, minifs);

    // Order statistics
    r.add("MEDIAN", 1, VARIADIC, false, median);
    r.add("MODE.SNGL", 1, VARIADIC, false, mode_sngl);
    r.add("MODE.MULT", 1, VARIADIC, false, mode_mult);
    r.add("LARGE", 2, 2, false, large);
    r.add("SMALL", 2, 2, false, small);

    // Rank
    r.add("RANK.EQ", 2, 3, false, rank_eq);
    r.add("RANK.AVG", 2, 3, false, rank_avg);
    r.alias("RANK", "RANK.EQ");

    // Variance / Std dev
    r.add("STDEV.S", 1, VARIADIC, false, stdev_s);
    r.add("STDEV.P", 1, VARIADIC, false, stdev_p);
    r.add("STDEVA", 1, VARIADIC, false, stdeva);
    r.add("STDEVPA", 1, VARIADIC, false, stdevpa);
    r.add("VAR.S", 1, VARIADIC, false, var_s);
    r.add("VAR.P", 1, VARIADIC, false, var_p);
    r.add("VARA", 1, VARIADIC, false, vara);
    r.add("VARPA", 1, VARIADIC, false, varpa);
    r.alias("STDEV", "STDEV.S");
    r.alias("STDEVP", "STDEV.P");
    r.alias("VAR", "VAR.S");
    r.alias("VARP", "VAR.P");

    // Percentile / Quartile
    r.add("PERCENTILE.INC", 2, 2, false, percentile_inc);
    r.add("PERCENTILE.EXC", 2, 2, false, percentile_exc);
    r.alias("PERCENTILE", "PERCENTILE.INC");
    r.add("QUARTILE.INC", 2, 2, false, quartile_inc);
    r.add("QUARTILE.EXC", 2, 2, false, quartile_exc);
    r.alias("QUARTILE", "QUARTILE.INC");
    r.add("PERCENTRANK.INC", 2, 3, false, percentrank_inc);
    r.add("PERCENTRANK.EXC", 2, 3, false, percentrank_exc);
    r.alias("PERCENTRANK", "PERCENTRANK.INC");

    // Correlation / covariance / regression
    r.add("CORREL", 2, 2, false, correl);
    r.add("COVARIANCE.P", 2, 2, false, covariance_p);
    r.add("COVARIANCE.S", 2, 2, false, covariance_s);
    r.add("PEARSON", 2, 2, false, correl); // same as CORREL
    r.add("SLOPE", 2, 2, false, slope);
    r.add("INTERCEPT", 2, 2, false, intercept);
    r.add("RSQ", 2, 2, false, rsq);
    r.add("FORECAST.LINEAR", 3, 3, false, forecast_linear);
    r.alias("FORECAST", "FORECAST.LINEAR");

    // Descriptive
    r.add("DEVSQ", 1, VARIADIC, false, devsq);
    r.add("AVEDEV", 1, VARIADIC, false, avedev);
    r.add("GEOMEAN", 1, VARIADIC, false, geomean);
    r.add("HARMEAN", 1, VARIADIC, false, harmean);
    r.add("TRIMMEAN", 2, 2, false, trimmean);
    r.add("SKEW", 1, VARIADIC, false, skew);
    r.add("KURT", 1, VARIADIC, false, kurt);
    r.add("STANDARDIZE", 3, 3, false, standardize);
    r.add("FISHER", 1, 1, false, fisher);
    r.add("FISHERINV", 1, 1, false, fisherinv);

    // Distributions
    r.add("NORM.DIST", 4, 4, false, norm_dist);
    r.add("NORM.S.DIST", 2, 2, false, norm_s_dist);
    r.add("NORM.INV", 3, 3, false, norm_inv);
    r.add("NORM.S.INV", 1, 1, false, norm_s_inv);
    r.add("BINOM.DIST", 4, 4, false, binom_dist);
    r.add("POISSON.DIST", 3, 3, false, poisson_dist);
    r.add("EXPON.DIST", 3, 3, false, expon_dist);
    r.add("CONFIDENCE.NORM", 3, 3, false, confidence_norm);
    r.add("GAUSS", 1, 1, false, gauss);
    r.add("PHI", 1, 1, false, phi);

    // Gamma / Beta distributions
    r.add("GAMMA.DIST", 4, 4, false, gamma_dist);
    r.add("GAMMA.INV", 3, 3, false, gamma_inv);
    r.add("BETA.DIST", 4, 6, false, beta_dist);
    r.add("BETA.INV", 3, 5, false, beta_inv);

    // Chi-squared
    r.add("CHISQ.DIST", 3, 3, false, chisq_dist);
    r.add("CHISQ.DIST.RT", 2, 2, false, chisq_dist_rt);
    r.add("CHISQ.INV", 2, 2, false, chisq_inv);
    r.add("CHISQ.INV.RT", 2, 2, false, chisq_inv_rt);

    // F distribution
    r.add("F.DIST", 4, 4, false, f_dist);
    r.add("F.DIST.RT", 3, 3, false, f_dist_rt);
    r.add("F.INV", 3, 3, false, f_inv);
    r.add("F.INV.RT", 3, 3, false, f_inv_rt);

    // Student's t distribution
    r.add("T.DIST", 3, 3, false, t_dist);
    r.add("T.DIST.RT", 2, 2, false, t_dist_rt);
    r.add("T.DIST.2T", 2, 2, false, t_dist_2t);
    r.add("T.INV", 2, 2, false, t_inv);
    r.add("T.INV.2T", 2, 2, false, t_inv_2t);

    // Lognormal
    r.add("LOGNORM.DIST", 4, 4, false, lognorm_dist);
    r.add("LOGNORM.INV", 3, 3, false, lognorm_inv);

    // Discrete distributions
    r.add("NEGBINOM.DIST", 4, 4, false, negbinom_dist);
    r.add("HYPGEOM.DIST", 5, 5, false, hypgeom_dist);
    r.add("BINOM.INV", 3, 3, false, binom_inv);
    r.add("BINOM.DIST.RANGE", 3, 4, false, binom_dist_range);

    // Weibull
    r.add("WEIBULL.DIST", 4, 4, false, weibull_dist);

    // Confidence / tests
    r.add("CONFIDENCE.T", 3, 3, false, confidence_t);
    r.add("Z.TEST", 2, 3, false, z_test);

    // Probability
    r.add("PROB", 3, 4, false, prob);

    // Additional descriptive / regression
    r.add("SKEW.P", 1, VARIADIC, false, skew_p);
    r.add("STEYX", 2, 2, false, steyx);

    // Hypothesis tests
    r.add("CHISQ.TEST", 2, 2, false, chisq_test);
    r.add("CHITEST", 2, 2, false, chisq_test); // legacy alias
    r.add("F.TEST", 2, 2, false, f_test);
    r.add("FTEST", 2, 2, false, f_test); // legacy alias
    r.add("T.TEST", 4, 4, false, t_test);
    r.add("TTEST", 4, 4, false, t_test); // legacy alias

    // ── Legacy compatibility aliases (pure synonyms) ──────────────────────
    r.alias("MODE", "MODE.SNGL");
    r.alias("COVAR", "COVARIANCE.P");
    r.alias("NORMDIST", "NORM.DIST");
    r.alias("NORMINV", "NORM.INV");
    r.alias("NORMSINV", "NORM.S.INV");
    r.alias("LOGINV", "LOGNORM.INV");
    r.alias("BETAINV", "BETA.INV");
    r.alias("GAMMADIST", "GAMMA.DIST");
    r.alias("GAMMAINV", "GAMMA.INV");
    r.alias("BINOMDIST", "BINOM.DIST");
    r.alias("POISSON", "POISSON.DIST");
    r.alias("EXPONDIST", "EXPON.DIST");
    r.alias("WEIBULL", "WEIBULL.DIST");
    r.alias("CHIDIST", "CHISQ.DIST.RT");
    r.alias("CHIINV", "CHISQ.INV.RT");
    r.alias("FDIST", "F.DIST.RT");
    r.alias("FINV", "F.INV.RT");
    r.alias("TINV", "T.INV.2T");
    r.alias("ZTEST", "Z.TEST");
    r.alias("CONFIDENCE", "CONFIDENCE.NORM");
    r.alias("CRITBINOM", "BINOM.INV");

    // ── Legacy compatibility aliases (signature differs → thin wrappers) ───
    // Legacy single-tail/no-cumulative forms map onto the modern functions
    // by supplying the extra argument the modern signature expects.
    r.add("NORMSDIST", 1, 1, false, normsdist); // NORM.S.DIST(z, TRUE)
    r.add("LOGNORMDIST", 3, 3, false, lognormdist); // LOGNORM.DIST(.., TRUE)
    r.add("BETADIST", 3, 5, false, betadist); // BETA.DIST(.., cumulative=TRUE, ..)
    r.add("NEGBINOMDIST", 3, 3, false, negbinomdist); // NEGBINOM.DIST(.., FALSE)
    r.add("HYPGEOMDIST", 4, 4, false, hypgeomdist); // HYPGEOM.DIST(.., FALSE)
    r.add("TDIST", 3, 3, false, tdist); // T.DIST.RT / T.DIST.2T via tails

    // Array regression functions
    r.add("FREQUENCY", 2, 2, false, frequency);
    r.add("TREND", 1, 4, false, trend);
    r.add("GROWTH", 1, 4, false, growth);
    r.add("LINEST", 1, 4, false, linest);
    r.add("LOGEST", 1, 4, false, logest);

    // Exponential smoothing (forecast) — not implemented.
    r.add("FORECAST.ETS", 3, 6, false, forecast_ets_na);
    r.add("FORECAST.ETS.CONFINT", 3, 7, false, forecast_ets_na);
    r.add("FORECAST.ETS.SEASONALITY", 2, 5, false, forecast_ets_na);
    r.add("FORECAST.ETS.STAT", 4, 7, false, forecast_ets_na);
}

// ---------------------------------------------------------------------------
// Legacy-alias wrappers (signature differs from the modern function)
// ---------------------------------------------------------------------------

/// Legacy `NORMSDIST(z)` = modern `NORM.S.DIST(z, TRUE)` (cumulative only).
fn normsdist(ctx: &mut dyn Context, args: &[Value]) -> Value {
    norm_s_dist(ctx, &[args[0].clone(), Value::Bool(true)])
}

/// Legacy `LOGNORMDIST(x, mean, sd)` = `LOGNORM.DIST(x, mean, sd, TRUE)`.
fn lognormdist(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let mut a = args.to_vec();
    a.push(Value::Bool(true));
    lognorm_dist(ctx, &a)
}

/// Legacy `BETADIST(x, alpha, beta, [A], [B])` = cumulative `BETA.DIST` with
/// the `cumulative` flag forced TRUE inserted at position 3.
fn betadist(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let mut a = Vec::with_capacity(args.len() + 1);
    a.extend_from_slice(&args[0..3]);
    a.push(Value::Bool(true));
    a.extend_from_slice(&args[3..]);
    beta_dist(ctx, &a)
}

/// Legacy `NEGBINOMDIST(f, s, p)` = `NEGBINOM.DIST(f, s, p, FALSE)` (pmf).
fn negbinomdist(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let mut a = args.to_vec();
    a.push(Value::Bool(false));
    negbinom_dist(ctx, &a)
}

/// Legacy `HYPGEOMDIST(sample_s, sample_n, pop_s, pop_n)` =
/// `HYPGEOM.DIST(.., FALSE)` (pmf only).
fn hypgeomdist(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let mut a = args.to_vec();
    a.push(Value::Bool(false));
    hypgeom_dist(ctx, &a)
}

/// Legacy `TDIST(x, df, tails)`: `tails`=1 → `T.DIST.RT`, `tails`=2 → `T.DIST.2T`.
/// Legacy TDIST requires `x >= 0` (returns `#NUM!` otherwise).
fn tdist(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let x = match to_number(&args[0]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let tails = match to_number(&args[2]) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if x < 0.0 {
        return Value::Error(CellError::Num);
    }
    match tails as i64 {
        1 => t_dist_rt(ctx, &args[0..2]),
        2 => t_dist_2t(ctx, &args[0..2]),
        _ => Value::Error(CellError::Num),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Flatten a value to scalars for criteria-range iteration.
fn flat(ctx: &mut dyn Context, v: &Value) -> Vec<Value> {
    ctx.flatten(v)
}

/// Get scalar from a possible array/ref.
fn single(v: &Value) -> Value {
    match v {
        Value::Array(a) => a.data.first().cloned().unwrap_or(Value::Empty),
        other => other.clone(),
    }
}

fn num(args: &[Value], i: usize) -> Result<f64, CellError> {
    to_number(&args[i])
}

/// Collect all numbers from args, treating Bools as 0/1 (A-variant includes
/// bools in ranges too).
fn collect_a(ctx: &mut dyn Context, args: &[Value]) -> Result<Vec<f64>, CellError> {
    let mut out = Vec::new();
    for arg in args {
        match arg {
            Value::Ref(_) | Value::Array(_) => {
                for v in ctx.flatten(arg) {
                    match v {
                        Value::Number(n) => out.push(n),
                        Value::Bool(b) => out.push(if b { 1.0 } else { 0.0 }),
                        Value::Text(_) => out.push(0.0),
                        Value::Empty => {}
                        Value::Error(e) => return Err(e),
                        _ => {}
                    }
                }
            }
            Value::Number(n) => out.push(*n),
            Value::Bool(b) => out.push(if *b { 1.0 } else { 0.0 }),
            Value::Text(_) => out.push(0.0),
            Value::Empty => {}
            Value::Error(e) => return Err(*e),
            Value::Lambda(_) => {}
        }
    }
    Ok(out)
}

fn variance(data: &[f64], population: bool) -> Result<f64, CellError> {
    let n = data.len();
    if n == 0 {
        return Err(CellError::Div0);
    }
    if !population && n < 2 {
        return Err(CellError::Div0);
    }
    let mean = data.iter().sum::<f64>() / n as f64;
    let ss: f64 = data.iter().map(|x| (x - mean) * (x - mean)).sum();
    let denom = if population { n as f64 } else { (n - 1) as f64 };
    Ok(ss / denom)
}

// ---------------------------------------------------------------------------
// Average family
// ---------------------------------------------------------------------------

fn average(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.is_empty() {
                Value::Error(CellError::Div0)
            } else {
                Value::Number(ns.iter().sum::<f64>() / ns.len() as f64)
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn averagea(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_a(ctx, args) {
        Ok(ns) => {
            if ns.is_empty() {
                Value::Error(CellError::Div0)
            } else {
                Value::Number(ns.iter().sum::<f64>() / ns.len() as f64)
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn averageif(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let range = flat(ctx, &args[0]);
    let crit = Criteria::parse(&single(&args[1]));
    let avg_range = if args.len() == 3 {
        flat(ctx, &args[2])
    } else {
        range.clone()
    };
    let mut total = 0.0;
    let mut cnt = 0usize;
    for (i, c) in range.iter().enumerate() {
        if crit.matches(c) {
            match avg_range.get(i) {
                Some(Value::Number(x)) => {
                    total += x;
                    cnt += 1;
                }
                Some(Value::Bool(b)) => {
                    total += if *b { 1.0 } else { 0.0 };
                    cnt += 1;
                }
                _ => {}
            }
        }
    }
    if cnt == 0 {
        Value::Error(CellError::Div0)
    } else {
        Value::Number(total / cnt as f64)
    }
}

fn averageifs(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // args: avg_range, (crit_range, crit)+
    if args.len() < 3 || !(args.len() - 1).is_multiple_of(2) {
        return Value::Error(CellError::Value);
    }
    let avg_range = flat(ctx, &args[0]);
    let mut pairs: Vec<(Vec<Value>, Criteria)> = Vec::new();
    let mut i = 1;
    while i + 1 < args.len() {
        let rng = flat(ctx, &args[i]);
        let crit = Criteria::parse(&single(&args[i + 1]));
        pairs.push((rng, crit));
        i += 2;
    }
    let mut total = 0.0;
    let mut cnt = 0usize;
    for (idx, sv) in avg_range.iter().enumerate() {
        let ok = pairs
            .iter()
            .all(|(rng, crit)| rng.get(idx).is_some_and(|c| crit.matches(c)));
        if ok && let Value::Number(x) = sv {
            total += x;
            cnt += 1;
        }
    }
    if cnt == 0 {
        Value::Error(CellError::Div0)
    } else {
        Value::Number(total / cnt as f64)
    }
}

// ---------------------------------------------------------------------------
// Count family
// ---------------------------------------------------------------------------

fn count(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let mut cnt = 0u64;
    for arg in args {
        match arg {
            Value::Ref(_) | Value::Array(_) => {
                for v in ctx.flatten(arg) {
                    if matches!(v, Value::Number(_)) {
                        cnt += 1;
                    }
                }
            }
            Value::Number(_) => cnt += 1,
            Value::Bool(_) => {} // direct bool literals not counted by COUNT
            Value::Text(s) => {
                if super::super::coerce::parse_number_text(s).is_some() {
                    cnt += 1;
                }
            }
            Value::Empty => {}
            Value::Error(_) => {}
            Value::Lambda(_) => {}
        }
    }
    Value::Number(cnt as f64)
}

fn counta(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let mut cnt = 0u64;
    for arg in args {
        match arg {
            Value::Ref(_) | Value::Array(_) => {
                for v in ctx.flatten(arg) {
                    if !matches!(v, Value::Empty) {
                        cnt += 1;
                    }
                }
            }
            Value::Empty => {}
            _ => cnt += 1,
        }
    }
    Value::Number(cnt as f64)
}

fn countblank(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let cells = flat(ctx, &args[0]);
    let cnt = cells
        .iter()
        .filter(|v| matches!(v, Value::Empty) || matches!(v, Value::Text(s) if s.is_empty()))
        .count();
    Value::Number(cnt as f64)
}

fn countif(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let range = flat(ctx, &args[0]);
    let crit = Criteria::parse(&single(&args[1]));
    let cnt = range.iter().filter(|v| crit.matches(v)).count();
    Value::Number(cnt as f64)
}

fn countifs(ctx: &mut dyn Context, args: &[Value]) -> Value {
    if args.len() < 2 || !args.len().is_multiple_of(2) {
        return Value::Error(CellError::Value);
    }
    let mut pairs: Vec<(Vec<Value>, Criteria)> = Vec::new();
    let mut i = 0;
    while i + 1 < args.len() {
        let rng = flat(ctx, &args[i]);
        let crit = Criteria::parse(&single(&args[i + 1]));
        pairs.push((rng, crit));
        i += 2;
    }
    let n = pairs.first().map_or(0, |(r, _)| r.len());
    let mut cnt = 0usize;
    for idx in 0..n {
        if pairs
            .iter()
            .all(|(rng, crit)| rng.get(idx).is_some_and(|c| crit.matches(c)))
        {
            cnt += 1;
        }
    }
    Value::Number(cnt as f64)
}

// ---------------------------------------------------------------------------
// Min / Max
// ---------------------------------------------------------------------------

fn max(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.is_empty() {
                Value::Number(0.0)
            } else {
                Value::Number(ns.iter().copied().fold(f64::NEG_INFINITY, f64::max))
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn maxa(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_a(ctx, args) {
        Ok(ns) => {
            if ns.is_empty() {
                Value::Number(0.0)
            } else {
                Value::Number(ns.iter().copied().fold(f64::NEG_INFINITY, f64::max))
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn min(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.is_empty() {
                Value::Number(0.0)
            } else {
                Value::Number(ns.iter().copied().fold(f64::INFINITY, f64::min))
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn mina(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_a(ctx, args) {
        Ok(ns) => {
            if ns.is_empty() {
                Value::Number(0.0)
            } else {
                Value::Number(ns.iter().copied().fold(f64::INFINITY, f64::min))
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn maxifs(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // args: max_range, (crit_range, crit)+
    if args.len() < 3 || !(args.len() - 1).is_multiple_of(2) {
        return Value::Error(CellError::Value);
    }
    let max_range = flat(ctx, &args[0]);
    let mut pairs: Vec<(Vec<Value>, Criteria)> = Vec::new();
    let mut i = 1;
    while i + 1 < args.len() {
        let rng = flat(ctx, &args[i]);
        let crit = Criteria::parse(&single(&args[i + 1]));
        pairs.push((rng, crit));
        i += 2;
    }
    let mut result = f64::NEG_INFINITY;
    let mut any = false;
    for (idx, sv) in max_range.iter().enumerate() {
        let ok = pairs
            .iter()
            .all(|(rng, crit)| rng.get(idx).is_some_and(|c| crit.matches(c)));
        if ok && let Value::Number(x) = sv {
            result = result.max(*x);
            any = true;
        }
    }
    Value::Number(if any { result } else { 0.0 })
}

fn minifs(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // args: min_range, (crit_range, crit)+
    if args.len() < 3 || !(args.len() - 1).is_multiple_of(2) {
        return Value::Error(CellError::Value);
    }
    let min_range = flat(ctx, &args[0]);
    let mut pairs: Vec<(Vec<Value>, Criteria)> = Vec::new();
    let mut i = 1;
    while i + 1 < args.len() {
        let rng = flat(ctx, &args[i]);
        let crit = Criteria::parse(&single(&args[i + 1]));
        pairs.push((rng, crit));
        i += 2;
    }
    let mut result = f64::INFINITY;
    let mut any = false;
    for (idx, sv) in min_range.iter().enumerate() {
        let ok = pairs
            .iter()
            .all(|(rng, crit)| rng.get(idx).is_some_and(|c| crit.matches(c)));
        if ok && let Value::Number(x) = sv {
            result = result.min(*x);
            any = true;
        }
    }
    Value::Number(if any { result } else { 0.0 })
}

// ---------------------------------------------------------------------------
// Median / Mode / Large / Small
// ---------------------------------------------------------------------------

fn median(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(mut ns) => {
            if ns.is_empty() {
                return Value::Error(CellError::Num);
            }
            ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = ns.len();
            if n % 2 == 1 {
                Value::Number(ns[n / 2])
            } else {
                Value::Number(f64::midpoint(ns[n / 2 - 1], ns[n / 2]))
            }
        }
        Err(e) => Value::Error(e),
    }
}

