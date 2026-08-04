//! Statistical worksheet functions.

use super::{Criteria, Registry, VARIADIC, collect_numbers};
use crate::core::error::CellError;
use crate::core::formula::coerce::to_number;
use crate::core::formula::context::Context;
use crate::core::formula::value::{Array, Value};

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

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
            .all(|(rng, crit)| rng.get(idx).map(|c| crit.matches(c)).unwrap_or(false));
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
    let n = pairs.first().map(|(r, _)| r.len()).unwrap_or(0);
    let mut cnt = 0usize;
    for idx in 0..n {
        if pairs
            .iter()
            .all(|(rng, crit)| rng.get(idx).map(|c| crit.matches(c)).unwrap_or(false))
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
                Value::Number(ns.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
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
                Value::Number(ns.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
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
                Value::Number(ns.iter().cloned().fold(f64::INFINITY, f64::min))
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
                Value::Number(ns.iter().cloned().fold(f64::INFINITY, f64::min))
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
            .all(|(rng, crit)| rng.get(idx).map(|c| crit.matches(c)).unwrap_or(false));
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
            .all(|(rng, crit)| rng.get(idx).map(|c| crit.matches(c)).unwrap_or(false));
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
                Value::Number((ns[n / 2 - 1] + ns[n / 2]) / 2.0)
            }
        }
        Err(e) => Value::Error(e),
    }
}

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

fn trimmean(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if !(0.0..1.0).contains(&p) {
        return Value::Error(CellError::Num);
    }
    match collect_numbers(ctx, &args[..1], false) {
        Ok(mut ns) => {
            if ns.is_empty() {
                return Value::Error(CellError::Div0);
            }
            ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let trim = (ns.len() as f64 * p / 2.0).floor() as usize;
            let trimmed = &ns[trim..ns.len() - trim];
            if trimmed.is_empty() {
                return Value::Error(CellError::Div0);
            }
            Value::Number(trimmed.iter().sum::<f64>() / trimmed.len() as f64)
        }
        Err(e) => Value::Error(e),
    }
}

fn skew(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            let n = ns.len();
            if n < 3 {
                return Value::Error(CellError::Div0);
            }
            let mean = ns.iter().sum::<f64>() / n as f64;
            let s = variance(&ns, false).map(|v| v.sqrt()).unwrap_or(0.0);
            if s == 0.0 {
                return Value::Error(CellError::Div0);
            }
            let nf = n as f64;
            let m3: f64 = ns.iter().map(|x| ((x - mean) / s).powi(3)).sum();
            Value::Number(nf / ((nf - 1.0) * (nf - 2.0)) * m3)
        }
        Err(e) => Value::Error(e),
    }
}

fn kurt(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            let n = ns.len();
            if n < 4 {
                return Value::Error(CellError::Div0);
            }
            let mean = ns.iter().sum::<f64>() / n as f64;
            let s = variance(&ns, false).map(|v| v.sqrt()).unwrap_or(0.0);
            if s == 0.0 {
                return Value::Error(CellError::Div0);
            }
            let nf = n as f64;
            let m4: f64 = ns.iter().map(|x| ((x - mean) / s).powi(4)).sum();
            let kurt = (nf * (nf + 1.0)) / ((nf - 1.0) * (nf - 2.0) * (nf - 3.0)) * m4
                - 3.0 * (nf - 1.0).powi(2) / ((nf - 2.0) * (nf - 3.0));
            Value::Number(kurt)
        }
        Err(e) => Value::Error(e),
    }
}

fn standardize(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let mean = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let std = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if std <= 0.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number((x - mean) / std)
}

fn fisher(_: &mut dyn Context, args: &[Value]) -> Value {
    match num(args, 0) {
        Ok(x) => {
            if x <= -1.0 || x >= 1.0 {
                Value::Error(CellError::Num)
            } else {
                Value::Number(0.5 * ((1.0 + x) / (1.0 - x)).ln())
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn fisherinv(_: &mut dyn Context, args: &[Value]) -> Value {
    match num(args, 0) {
        Ok(y) => {
            let e2y = (2.0 * y).exp();
            Value::Number((e2y - 1.0) / (e2y + 1.0))
        }
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// Statistical distributions
// ---------------------------------------------------------------------------

/// Standard normal PDF.
fn norm_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// Standard normal CDF using Horner's method approximation.
fn norm_cdf(x: f64) -> f64 {
    // Abramowitz & Stegun 26.2.17 approximation (max error ~7.5e-8)
    if x < 0.0 {
        return 1.0 - norm_cdf(-x);
    }
    let t = 1.0 / (1.0 + 0.2316419 * x);
    let poly = t
        * (0.319_381_530
            + t * (-0.356_563_782
                + t * (1.781_477_937 + t * (-1.821_255_978 + t * 1.330_274_429))));
    1.0 - norm_pdf(x) * poly
}

/// Inverse normal CDF (rational approximation, Beasley & Springer 1977).
fn norm_inv_cdf(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        return f64::NAN;
    }
    // Rational approximation for central region; reflect for upper half.
    const A: [f64; 4] = [2.515_517, 0.802_853, 0.010_328, 0.0];
    const B: [f64; 3] = [1.432_788, 0.189_269, 0.001_308];
    fn rational(t: f64) -> f64 {
        let num = A[0] + t * (A[1] + t * (A[2] + t * A[3]));
        let den = 1.0 + t * (B[0] + t * (B[1] + t * B[2]));
        t - num / den
    }
    if p < 0.5 {
        let t = (-2.0 * p.ln()).sqrt();
        -rational(t)
    } else {
        let t = (-2.0 * (1.0 - p).ln()).sqrt();
        rational(t)
    }
}

fn norm_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let mean = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let std = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 3) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if std <= 0.0 {
        return Value::Error(CellError::Num);
    }
    let z = (x - mean) / std;
    if cumulative {
        Value::Number(norm_cdf(z))
    } else {
        Value::Number(norm_pdf(z) / std)
    }
}

fn norm_s_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let z = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 1) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if cumulative {
        Value::Number(norm_cdf(z))
    } else {
        Value::Number(norm_pdf(z))
    }
}

fn norm_inv(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let mean = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let std = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if std <= 0.0 || p <= 0.0 || p >= 1.0 {
        return Value::Error(CellError::Num);
    }
    let z = norm_inv_cdf(p);
    if z.is_nan() {
        Value::Error(CellError::Num)
    } else {
        Value::Number(mean + z * std)
    }
}

fn norm_s_inv(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if p <= 0.0 || p >= 1.0 {
        return Value::Error(CellError::Num);
    }
    let z = norm_inv_cdf(p);
    if z.is_nan() {
        Value::Error(CellError::Num)
    } else {
        Value::Number(z)
    }
}

/// Regularized incomplete beta function used for BINOM.DIST cumulative.
/// Uses continued fraction / series expansion.
fn regularized_incomplete_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    // Use series expansion for x < (a+1)/(a+b+2)
    let threshold = (a + 1.0) / (a + b + 2.0);
    if x < threshold {
        beta_cf(x, a, b)
    } else {
        1.0 - beta_cf(1.0 - x, b, a)
    }
}

fn beta_cf(x: f64, a: f64, b: f64) -> f64 {
    // Lentz's continued fraction method
    let lbeta = super::math::ln_gamma(a) + super::math::ln_gamma(b) - super::math::ln_gamma(a + b);
    let front = (a * x.ln() + b * (1.0 - x).ln() - lbeta).exp() / a;
    let mut c = 1.0;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < 1e-30 {
        d = 1e-30;
    }
    d = 1.0 / d;
    let mut f = d;
    for m in 1..=200 {
        let mf = m as f64;
        // even step
        let num_even = mf * (b - mf) * x / ((a + 2.0 * mf - 1.0) * (a + 2.0 * mf));
        d = 1.0 + num_even * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + num_even / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        f *= c * d;
        // odd step
        let num_odd = -(a + mf) * (a + b + mf) * x / ((a + 2.0 * mf) * (a + 2.0 * mf + 1.0));
        d = 1.0 + num_odd * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + num_odd / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;
        if (delta - 1.0).abs() < 1e-12 {
            break;
        }
    }
    front * f
}

fn binom_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let k = match num(args, 0) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let n = match num(args, 1) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let p = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 3) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if k < 0 || n < 0 || k > n || !(0.0..=1.0).contains(&p) {
        return Value::Error(CellError::Num);
    }
    if cumulative {
        // P(X <= k) = I_{1-p}(n-k, k+1)
        let result = regularized_incomplete_beta(1.0 - p, (n - k) as f64, (k + 1) as f64);
        Value::Number(result)
    } else {
        // PMF: C(n,k) * p^k * (1-p)^(n-k)
        let log_pmf = super::math::ln_gamma((n + 1) as f64)
            - super::math::ln_gamma((k + 1) as f64)
            - super::math::ln_gamma((n - k + 1) as f64)
            + k as f64 * p.ln()
            + (n - k) as f64 * (1.0 - p).ln();
        Value::Number(log_pmf.exp())
    }
}

fn poisson_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let lambda = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 2) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if x < 0 || lambda < 0.0 {
        return Value::Error(CellError::Num);
    }
    if cumulative {
        // P(X <= x) = sum_{k=0}^{x} e^{-lambda} * lambda^k / k!
        let mut sum = 0.0;
        let mut term = (-lambda).exp();
        for k in 0..=x {
            sum += term;
            term *= lambda / (k + 1) as f64;
        }
        Value::Number(sum.min(1.0))
    } else {
        let log_pmf = x as f64 * lambda.ln() - lambda - super::math::ln_gamma((x + 1) as f64);
        Value::Number(log_pmf.exp())
    }
}

fn expon_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let lambda = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 2) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if x < 0.0 || lambda <= 0.0 {
        return Value::Error(CellError::Num);
    }
    if cumulative {
        Value::Number(1.0 - (-lambda * x).exp())
    } else {
        Value::Number(lambda * (-lambda * x).exp())
    }
}

fn confidence_norm(_: &mut dyn Context, args: &[Value]) -> Value {
    let alpha = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let std = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let n = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if alpha <= 0.0 || alpha >= 1.0 || std <= 0.0 || n < 1.0 {
        return Value::Error(CellError::Num);
    }
    let z = norm_inv_cdf(1.0 - alpha / 2.0);
    Value::Number(z * std / n.sqrt())
}

fn gauss(_: &mut dyn Context, args: &[Value]) -> Value {
    match num(args, 0) {
        Ok(z) => Value::Number(norm_cdf(z) - 0.5),
        Err(e) => Value::Error(e),
    }
}

fn phi(_: &mut dyn Context, args: &[Value]) -> Value {
    match num(args, 0) {
        Ok(x) => Value::Number(norm_pdf(x)),
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// Numerical helpers for additional distributions
// ---------------------------------------------------------------------------

/// Lower regularized incomplete gamma function P(a, x) = γ(a,x)/Γ(a).
/// PARITY: series expansion for x < a+1, continued fraction otherwise
/// (Numerical Recipes gser/gcf); ~1e-10 accuracy.
fn reg_inc_gamma_p(a: f64, x: f64) -> f64 {
    if x <= 0.0 || a <= 0.0 {
        return 0.0;
    }
    if x < a + 1.0 {
        // Series representation.
        let mut ap = a;
        let mut sum = 1.0 / a;
        let mut del = sum;
        for _ in 0..1000 {
            ap += 1.0;
            del *= x / ap;
            sum += del;
            if del.abs() < sum.abs() * 1e-15 {
                break;
            }
        }
        sum * (-x + a * x.ln() - super::math::ln_gamma(a)).exp()
    } else {
        // Continued fraction (Lentz) for Q(a,x), then P = 1 - Q.
        let mut b = x + 1.0 - a;
        let mut c = 1e30;
        let mut d = 1.0 / b;
        let mut h = d;
        for i in 1..1000 {
            let an = -(i as f64) * (i as f64 - a);
            b += 2.0;
            d = an * d + b;
            if d.abs() < 1e-30 {
                d = 1e-30;
            }
            c = b + an / c;
            if c.abs() < 1e-30 {
                c = 1e-30;
            }
            d = 1.0 / d;
            let del = d * c;
            h *= del;
            if (del - 1.0).abs() < 1e-15 {
                break;
            }
        }
        let q = (-x + a * x.ln() - super::math::ln_gamma(a)).exp() * h;
        1.0 - q
    }
}

/// Inverse of lower regularized incomplete gamma in x for fixed a: returns x
/// such that P(a,x) = p. Bisection (monotone in x). PARITY: ~1e-8 accuracy.
fn reg_inc_gamma_p_inv(a: f64, p: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    let mut lo = 0.0f64;
    let mut hi = a.max(1.0);
    while reg_inc_gamma_p(a, hi) < p {
        hi *= 2.0;
        if hi > 1e300 {
            return hi;
        }
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if reg_inc_gamma_p(a, mid) < p {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-12 * (1.0 + hi.abs()) {
            break;
        }
    }
    0.5 * (lo + hi)
}

/// Inverse of the regularized incomplete beta I_x(a,b) in x: returns x in [0,1]
/// such that I_x(a,b) = p. Bisection (monotone). PARITY: ~1e-10 accuracy.
fn reg_inc_beta_inv(p: f64, a: f64, b: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return 1.0;
    }
    let mut lo = 0.0f64;
    let mut hi = 1.0f64;
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if regularized_incomplete_beta(mid, a, b) < p {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-14 {
            break;
        }
    }
    0.5 * (lo + hi)
}

/// Gamma PDF with shape `a` and scale `b`.
fn gamma_pdf(x: f64, a: f64, b: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        return if a < 1.0 {
            f64::INFINITY
        } else if a == 1.0 {
            1.0 / b
        } else {
            0.0
        };
    }
    ((a - 1.0) * x.ln() - x / b - a * b.ln() - super::math::ln_gamma(a)).exp()
}

// ---------------------------------------------------------------------------
// GAMMA.DIST / GAMMA.INV
// ---------------------------------------------------------------------------

fn gamma_dist(_: &mut dyn Context, args: &[Value]) -> Value {
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
    if cumulative {
        Value::Number(reg_inc_gamma_p(alpha, x / beta))
    } else {
        Value::Number(gamma_pdf(x, alpha, beta))
    }
}

fn gamma_inv(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
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
    if !(0.0..1.0).contains(&p) || alpha <= 0.0 || beta <= 0.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(reg_inc_gamma_p_inv(alpha, p) * beta)
}

// ---------------------------------------------------------------------------
// BETA.DIST / BETA.INV
// ---------------------------------------------------------------------------

fn beta_pdf(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 || x >= 1.0 {
        return 0.0;
    }
    let lbeta = super::math::ln_gamma(a) + super::math::ln_gamma(b) - super::math::ln_gamma(a + b);
    ((a - 1.0) * x.ln() + (b - 1.0) * (1.0 - x).ln() - lbeta).exp()
}

fn beta_dist(_: &mut dyn Context, args: &[Value]) -> Value {
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
    let lo = if args.len() >= 5 {
        match num(args, 4) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        0.0
    };
    let hi = if args.len() >= 6 {
        match num(args, 5) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        1.0
    };
    if alpha <= 0.0 || beta <= 0.0 || hi <= lo || x < lo || x > hi {
        return Value::Error(CellError::Num);
    }
    let z = (x - lo) / (hi - lo);
    if cumulative {
        Value::Number(regularized_incomplete_beta(z, alpha, beta))
    } else {
        Value::Number(beta_pdf(z, alpha, beta) / (hi - lo))
    }
}

fn beta_inv(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
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
    let lo = if args.len() >= 4 {
        match num(args, 3) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        0.0
    };
    let hi = if args.len() >= 5 {
        match num(args, 4) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        1.0
    };
    if !(0.0..=1.0).contains(&p) || alpha <= 0.0 || beta <= 0.0 || hi <= lo {
        return Value::Error(CellError::Num);
    }
    let z = reg_inc_beta_inv(p, alpha, beta);
    Value::Number(lo + z * (hi - lo))
}

// ---------------------------------------------------------------------------
// CHISQ family (chi-squared = gamma with alpha=df/2, beta=2)
// ---------------------------------------------------------------------------

fn chisq_cdf(x: f64, df: f64) -> f64 {
    reg_inc_gamma_p(df / 2.0, x / 2.0)
}

fn chisq_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let df = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 2) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if x < 0.0 || df < 1.0 {
        return Value::Error(CellError::Num);
    }
    if cumulative {
        Value::Number(chisq_cdf(x, df))
    } else {
        Value::Number(gamma_pdf(x, df / 2.0, 2.0))
    }
}

fn chisq_dist_rt(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let df = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if x < 0.0 || df < 1.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(1.0 - chisq_cdf(x, df))
}

fn chisq_inv(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let df = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if !(0.0..1.0).contains(&p) || df < 1.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(reg_inc_gamma_p_inv(df / 2.0, p) * 2.0)
}

fn chisq_inv_rt(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let df = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if !(0.0..=1.0).contains(&p) || p == 0.0 || df < 1.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(reg_inc_gamma_p_inv(df / 2.0, 1.0 - p) * 2.0)
}

// ---------------------------------------------------------------------------
// F distribution (incomplete beta)
// ---------------------------------------------------------------------------

/// CDF of F with df1, df2: I_{d1 x / (d1 x + d2)}(d1/2, d2/2).
fn f_cdf(x: f64, d1: f64, d2: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let z = d1 * x / (d1 * x + d2);
    regularized_incomplete_beta(z, d1 / 2.0, d2 / 2.0)
}

fn f_pdf(x: f64, d1: f64, d2: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let lbeta = super::math::ln_gamma(d1 / 2.0) + super::math::ln_gamma(d2 / 2.0)
        - super::math::ln_gamma((d1 + d2) / 2.0);
    let ln = 0.5 * (d1 * (d1).ln() + d2 * (d2).ln()) + (d1 / 2.0 - 1.0) * x.ln()
        - ((d1 + d2) / 2.0) * (d1 * x + d2).ln()
        - lbeta;
    ln.exp()
}

fn f_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let d1 = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let d2 = match num(args, 2) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 3) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if x < 0.0 || d1 < 1.0 || d2 < 1.0 {
        return Value::Error(CellError::Num);
    }
    if cumulative {
        Value::Number(f_cdf(x, d1, d2))
    } else {
        Value::Number(f_pdf(x, d1, d2))
    }
}

fn f_dist_rt(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let d1 = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let d2 = match num(args, 2) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if x < 0.0 || d1 < 1.0 || d2 < 1.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(1.0 - f_cdf(x, d1, d2))
}

/// Returns x such that f_cdf(x,d1,d2) = p.
fn f_inv_p(p: f64, d1: f64, d2: f64) -> f64 {
    // I_z(d1/2,d2/2) = p where z = d1 x / (d1 x + d2); invert beta then solve x.
    let z = reg_inc_beta_inv(p, d1 / 2.0, d2 / 2.0);
    if z >= 1.0 {
        return f64::INFINITY;
    }
    d2 * z / (d1 * (1.0 - z))
}

fn f_inv(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let d1 = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let d2 = match num(args, 2) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if !(0.0..1.0).contains(&p) || d1 < 1.0 || d2 < 1.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(f_inv_p(p, d1, d2))
}

fn f_inv_rt(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let d1 = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let d2 = match num(args, 2) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if !(0.0..=1.0).contains(&p) || p == 0.0 || d1 < 1.0 || d2 < 1.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(f_inv_p(1.0 - p, d1, d2))
}

// ---------------------------------------------------------------------------
// Student's t distribution (incomplete beta)
// ---------------------------------------------------------------------------

/// Left-tail CDF of Student's t with df degrees of freedom.
fn t_cdf(x: f64, df: f64) -> f64 {
    let z = df / (df + x * x);
    let ib = 0.5 * regularized_incomplete_beta(z, df / 2.0, 0.5);
    if x >= 0.0 { 1.0 - ib } else { ib }
}

fn t_pdf(x: f64, df: f64) -> f64 {
    let lc = super::math::ln_gamma((df + 1.0) / 2.0)
        - super::math::ln_gamma(df / 2.0)
        - 0.5 * (df * std::f64::consts::PI).ln();
    (lc - (df + 1.0) / 2.0 * (1.0 + x * x / df).ln()).exp()
}

fn t_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let df = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 2) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if df < 1.0 {
        return Value::Error(CellError::Num);
    }
    if cumulative {
        Value::Number(t_cdf(x, df))
    } else {
        Value::Number(t_pdf(x, df))
    }
}

fn t_dist_rt(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let df = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if df < 1.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(1.0 - t_cdf(x, df))
}

fn t_dist_2t(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let df = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if df < 1.0 || x < 0.0 {
        return Value::Error(CellError::Num);
    }
    // two-tailed: 2 * P(T > |x|)
    Value::Number(2.0 * (1.0 - t_cdf(x.abs(), df)))
}

fn t_inv(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let df = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if !(0.0..1.0).contains(&p) || p == 0.0 || df < 1.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(t_inv_left(p, df))
}

/// Inverse left-tail t CDF via bisection. PARITY: ~1e-9.
fn t_inv_left(p: f64, df: f64) -> f64 {
    // symmetric, monotone increasing in x.
    let mut lo = -1e6f64;
    let mut hi = 1e6f64;
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if t_cdf(mid, df) < p {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-10 {
            break;
        }
    }
    0.5 * (lo + hi)
}

fn t_inv_2t(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let df = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    if !(0.0..=1.0).contains(&p) || p == 0.0 || p > 1.0 || df < 1.0 {
        return Value::Error(CellError::Num);
    }
    // two-tailed: find x>0 with 2*P(T>x)=p  →  P(T<=x) = 1 - p/2.
    Value::Number(t_inv_left(1.0 - p / 2.0, df))
}

// ---------------------------------------------------------------------------
// Lognormal
// ---------------------------------------------------------------------------

fn lognorm_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let mean = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let sd = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 3) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if sd <= 0.0 || x <= 0.0 {
        return Value::Error(CellError::Num);
    }
    let z = (x.ln() - mean) / sd;
    if cumulative {
        Value::Number(norm_cdf(z))
    } else {
        Value::Number(norm_pdf(z) / (x * sd))
    }
}

fn lognorm_inv(_: &mut dyn Context, args: &[Value]) -> Value {
    let p = match num(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let mean = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let sd = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if sd <= 0.0 || p <= 0.0 || p >= 1.0 {
        return Value::Error(CellError::Num);
    }
    let z = norm_inv_cdf(p);
    if z.is_nan() {
        Value::Error(CellError::Num)
    } else {
        Value::Number((mean + sd * z).exp())
    }
}

// ---------------------------------------------------------------------------
// NEGBINOM.DIST / HYPGEOM.DIST / BINOM.INV / BINOM.DIST.RANGE
// ---------------------------------------------------------------------------

/// log of binomial coefficient C(n, k).
fn ln_choose(n: f64, k: f64) -> f64 {
    super::math::ln_gamma(n + 1.0)
        - super::math::ln_gamma(k + 1.0)
        - super::math::ln_gamma(n - k + 1.0)
}

fn negbinom_pmf(f: f64, s: f64, p: f64) -> f64 {
    // P(f failures before s-th success) = C(f+s-1, f) p^s (1-p)^f
    (ln_choose(f + s - 1.0, f) + s * p.ln() + f * (1.0 - p).ln()).exp()
}

fn negbinom_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    let f = match num(args, 0) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let s = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let p = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 3) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if f < 0.0 || s < 1.0 || !(0.0..=1.0).contains(&p) {
        return Value::Error(CellError::Num);
    }
    if cumulative {
        let mut sum = 0.0;
        let mut i = 0.0;
        while i <= f {
            sum += negbinom_pmf(i, s, p);
            i += 1.0;
        }
        Value::Number(sum.min(1.0))
    } else {
        Value::Number(negbinom_pmf(f, s, p))
    }
}

fn hypgeom_pmf(k: f64, n: f64, big_k: f64, big_n: f64) -> f64 {
    (ln_choose(big_k, k) + ln_choose(big_n - big_k, n - k) - ln_choose(big_n, n)).exp()
}

fn hypgeom_dist(_: &mut dyn Context, args: &[Value]) -> Value {
    // sample_s, sample_n, pop_s, pop_n, cumulative
    let sample_s = match num(args, 0) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let sample_n = match num(args, 1) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let pop_s = match num(args, 2) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let pop_n = match num(args, 3) {
        Ok(v) => v.trunc(),
        Err(e) => return Value::Error(e),
    };
    let cumulative = match num(args, 4) {
        Ok(v) => v != 0.0,
        Err(e) => return Value::Error(e),
    };
    if sample_s < 0.0
        || sample_s > sample_n
        || sample_s > pop_s
        || sample_n - sample_s > pop_n - pop_s
        || sample_n > pop_n
        || pop_s > pop_n
        || pop_n < 0.0
    {
        return Value::Error(CellError::Num);
    }
    if cumulative {
        let mut sum = 0.0;
        let lo = (sample_n - (pop_n - pop_s)).max(0.0);
        let mut k = lo;
        while k <= sample_s {
            sum += hypgeom_pmf(k, sample_n, pop_s, pop_n);
            k += 1.0;
        }
        Value::Number(sum.min(1.0))
    } else {
        Value::Number(hypgeom_pmf(sample_s, sample_n, pop_s, pop_n))
    }
}

/// binomial CDF P(X<=k) for integer n.
fn binom_cdf(k: i64, n: i64, p: f64) -> f64 {
    if k < 0 {
        return 0.0;
    }
    if k >= n {
        return 1.0;
    }
    regularized_incomplete_beta(1.0 - p, (n - k) as f64, (k + 1) as f64)
}

fn binom_inv(_: &mut dyn Context, args: &[Value]) -> Value {
    let n = match num(args, 0) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let p = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let alpha = match num(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if n < 0 || !(0.0..=1.0).contains(&p) || !(0.0..=1.0).contains(&alpha) {
        return Value::Error(CellError::Num);
    }
    // smallest k with binom_cdf(k) >= alpha.
    let mut k = 0i64;
    while k <= n {
        if binom_cdf(k, n, p) >= alpha {
            return Value::Number(k as f64);
        }
        k += 1;
    }
    Value::Number(n as f64)
}

fn binom_dist_range(_: &mut dyn Context, args: &[Value]) -> Value {
    let n = match num(args, 0) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let p = match num(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let s1 = match num(args, 2) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    let s2 = if args.len() >= 4 {
        match num(args, 3) {
            Ok(v) => v.trunc() as i64,
            Err(e) => return Value::Error(e),
        }
    } else {
        s1
    };
    if n < 0 || !(0.0..=1.0).contains(&p) || s1 < 0 || s2 > n || s2 < s1 {
        return Value::Error(CellError::Num);
    }
    // P(s1 <= X <= s2) = CDF(s2) - CDF(s1-1)
    let upper = binom_cdf(s2, n, p);
    let lower = binom_cdf(s1 - 1, n, p);
    Value::Number((upper - lower).clamp(0.0, 1.0))
}

// ---------------------------------------------------------------------------
// WEIBULL.DIST
// ---------------------------------------------------------------------------

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
    Value::Array(crate::core::formula::value::Array::new(rows, 1, out))
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
    Value::Array(crate::core::formula::value::Array::new(rows, 1, out))
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
    Value::Array(crate::core::formula::value::Array::new(rows, 1, out))
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
    Value::Array(crate::core::formula::value::Array::new(
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
    Value::Array(crate::core::formula::value::Array::new(
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
fn numeric_matrix(ctx: &mut dyn Context, v: &Value) -> Option<(usize, usize, Vec<f64>)> {
    let arr = match v {
        Value::Ref(r) => ctx.ref_to_array(*r),
        Value::Array(a) => a.clone(),
        other => crate::core::formula::value::Array::scalar(other.clone()),
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
    Some((arr.rows, arr.cols, data))
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
    let Some((ar, ac, actual)) = numeric_matrix(ctx, &args[0]) else {
        return Value::Error(CellError::Value);
    };
    let Some((er, ec, expected)) = numeric_matrix(ctx, &args[1]) else {
        return Value::Error(CellError::Value);
    };
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
mod tests {
    use super::*;
    use crate::core::formula::functions::testutil::{TestCtx, rng};

    fn ctx() -> TestCtx {
        TestCtx::new()
    }

    // ---- hypothesis tests --------------------------------------------------

    #[test]
    fn chisq_test_identical_is_one() {
        // actual == expected → χ² = 0 → p = 1
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Number(20.0)),
            (0, 1, Value::Number(10.0)),
            (1, 1, Value::Number(20.0)),
        ]);
        let r = chisq_test(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 1, 1)]);
        match r {
            Value::Number(p) => assert!((p - 1.0).abs() < 1e-9, "got {p}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn f_test_equal_variance() {
        // identical samples → F = 1 → two-tailed p = 1
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        let r = f_test(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        match r {
            Value::Number(p) => assert!((p - 1.0).abs() < 1e-6, "got {p}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn t_test_identical_samples() {
        // mean difference 0 (diffs [-2,2,0]) but nonzero variance → t = 0 → p = 1
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(5.0)),
            (1, 0, Value::Number(7.0)),
            (2, 0, Value::Number(9.0)),
            (0, 1, Value::Number(7.0)),
            (1, 1, Value::Number(5.0)),
            (2, 1, Value::Number(9.0)),
        ]);
        let r = t_test(
            &mut c,
            &[
                rng(0, 0, 2, 0),
                rng(0, 1, 2, 1),
                Value::Number(2.0),
                Value::Number(1.0),
            ],
        );
        match r {
            Value::Number(p) => assert!((p - 1.0).abs() < 1e-9, "got {p}"),
            other => panic!("{other:?}"),
        }
        // wrong tails value → #NUM!
        assert_eq!(
            t_test(
                &mut c,
                &[
                    rng(0, 0, 2, 0),
                    rng(0, 1, 2, 1),
                    Value::Number(3.0),
                    Value::Number(1.0)
                ]
            ),
            Value::Error(CellError::Num)
        );
    }

    // ---- AVERAGE -----------------------------------------------------------

    #[test]
    fn average_basic() {
        let mut c = ctx();
        assert_eq!(
            average(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn average_empty_range() {
        let mut c = ctx();
        assert_eq!(
            average(&mut c, &[Value::Empty]),
            Value::Error(CellError::Div0)
        );
    }

    #[test]
    fn average_skips_text_in_range() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Text("hello".into())),
            (2, 0, Value::Number(20.0)),
        ]);
        assert_eq!(average(&mut c, &[rng(0, 0, 2, 0)]), Value::Number(15.0));
    }

    // ---- AVERAGEA ----------------------------------------------------------

    #[test]
    fn averagea_counts_text_as_zero() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Text("hello".into())),
            (2, 0, Value::Number(20.0)),
        ]);
        // 10 + 0 + 20 = 30, count = 3 → 10
        assert_eq!(averagea(&mut c, &[rng(0, 0, 2, 0)]), Value::Number(10.0));
    }

    // ---- AVERAGEIF ---------------------------------------------------------

    #[test]
    fn averageif_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(10.0)),
        ]);
        let r = averageif(&mut c, &[rng(0, 0, 2, 0), Value::Text(">3".into())]);
        assert_eq!(r, Value::Number(7.5));
    }

    #[test]
    fn averageif_no_match() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0))]);
        let r = averageif(&mut c, &[rng(0, 0, 0, 0), Value::Text(">100".into())]);
        assert_eq!(r, Value::Error(CellError::Div0));
    }

    // ---- COUNT / COUNTA ----------------------------------------------------

    #[test]
    fn count_only_numbers() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Text("x".into())),
            (2, 0, Value::Bool(true)),
            (3, 0, Value::Empty),
        ]);
        assert_eq!(count(&mut c, &[rng(0, 0, 3, 0)]), Value::Number(1.0));
    }

    #[test]
    fn counta_non_empty() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Text("x".into())),
            (2, 0, Value::Bool(true)),
            (3, 0, Value::Empty),
        ]);
        assert_eq!(counta(&mut c, &[rng(0, 0, 3, 0)]), Value::Number(3.0));
    }

    #[test]
    fn countblank_fn() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Empty),
            (2, 0, Value::Text("".into())),
        ]);
        assert_eq!(countblank(&mut c, &[rng(0, 0, 2, 0)]), Value::Number(2.0));
    }

    // ---- COUNTIF / COUNTIFS ------------------------------------------------

    #[test]
    fn countif_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(10.0)),
        ]);
        assert_eq!(
            countif(&mut c, &[rng(0, 0, 2, 0), Value::Text(">3".into())]),
            Value::Number(2.0)
        );
    }

    #[test]
    fn countifs_two_criteria() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(5.0)),
            (1, 0, Value::Number(10.0)),
            (2, 0, Value::Number(15.0)),
        ]);
        let r = countifs(
            &mut c,
            &[
                rng(0, 0, 2, 0),
                Value::Text(">=5".into()),
                rng(0, 0, 2, 0),
                Value::Text("<=10".into()),
            ],
        );
        assert_eq!(r, Value::Number(2.0));
    }

    // ---- MAX / MIN ---------------------------------------------------------

    #[test]
    fn max_basic() {
        let mut c = ctx();
        assert_eq!(
            max(
                &mut c,
                &[Value::Number(3.0), Value::Number(1.0), Value::Number(2.0)]
            ),
            Value::Number(3.0)
        );
    }

    #[test]
    fn min_basic() {
        let mut c = ctx();
        assert_eq!(
            min(
                &mut c,
                &[Value::Number(3.0), Value::Number(1.0), Value::Number(2.0)]
            ),
            Value::Number(1.0)
        );
    }

    #[test]
    fn max_empty_returns_zero() {
        let mut c = ctx();
        assert_eq!(max(&mut c, &[Value::Empty]), Value::Number(0.0));
    }

    // ---- MEDIAN ------------------------------------------------------------

    #[test]
    fn median_odd() {
        let mut c = ctx();
        assert_eq!(
            median(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn median_even() {
        let mut c = ctx();
        // Excel: MEDIAN(1,2,3,4) = 2.5
        assert_eq!(
            median(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(4.0)
                ]
            ),
            Value::Number(2.5)
        );
    }

    #[test]
    fn median_empty_error() {
        let mut c = ctx();
        assert_eq!(
            median(&mut c, &[Value::Empty]),
            Value::Error(CellError::Num)
        );
    }

    // ---- MODE.SNGL ---------------------------------------------------------

    #[test]
    fn mode_basic() {
        let mut c = ctx();
        assert_eq!(
            mode_sngl(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(2.0),
                    Value::Number(3.0)
                ]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn mode_no_repeat() {
        let mut c = ctx();
        assert_eq!(
            mode_sngl(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Error(CellError::NA)
        );
    }

    // ---- LARGE / SMALL -----------------------------------------------------

    #[test]
    fn large_small() {
        let mut c = ctx();
        let data = [
            Value::Number(3.0),
            Value::Number(1.0),
            Value::Number(4.0),
            Value::Number(2.0),
        ];
        assert_eq!(
            large(&mut c, &[data[0].clone(), Value::Number(1.0)]),
            Value::Number(3.0)
        );
        // Use range for multi-value
        let mut c2 = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(1.0)),
            (2, 0, Value::Number(4.0)),
            (3, 0, Value::Number(2.0)),
        ]);
        assert_eq!(
            large(&mut c2, &[rng(0, 0, 3, 0), Value::Number(2.0)]),
            Value::Number(3.0)
        );
        assert_eq!(
            small(&mut c2, &[rng(0, 0, 3, 0), Value::Number(2.0)]),
            Value::Number(2.0)
        );
    }

    #[test]
    fn large_out_of_range() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0))]);
        assert_eq!(
            large(&mut c, &[rng(0, 0, 0, 0), Value::Number(5.0)]),
            Value::Error(CellError::Num)
        );
    }

    // ---- RANK.EQ / RANK.AVG ------------------------------------------------

    #[test]
    fn rank_eq_desc() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(7.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(5.0)),
        ]);
        // 7 is rank 1 in descending
        assert_eq!(
            rank_eq(&mut c, &[Value::Number(7.0), rng(0, 0, 2, 0)]),
            Value::Number(1.0)
        );
    }

    #[test]
    fn rank_eq_asc() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(7.0)),
            (1, 0, Value::Number(3.0)),
            (2, 0, Value::Number(5.0)),
        ]);
        // 3 is rank 1 ascending
        assert_eq!(
            rank_eq(
                &mut c,
                &[Value::Number(3.0), rng(0, 0, 2, 0), Value::Number(1.0)]
            ),
            Value::Number(1.0)
        );
    }

    #[test]
    fn rank_avg_ties() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(5.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(3.0)),
        ]);
        // Both 5s tie for rank 1 and 2, avg = 1.5
        assert_eq!(
            rank_avg(&mut c, &[Value::Number(5.0), rng(0, 0, 2, 0)]),
            Value::Number(1.5)
        );
    }

    // ---- STDEV / VAR -------------------------------------------------------

    #[test]
    fn stdev_s_excel_example() {
        // Excel: STDEV.S(2,4,4,4,5,5,7,9) ≈ 2.138...
        let mut c = ctx();
        let data = vec![
            Value::Number(2.0),
            Value::Number(4.0),
            Value::Number(4.0),
            Value::Number(4.0),
            Value::Number(5.0),
            Value::Number(5.0),
            Value::Number(7.0),
            Value::Number(9.0),
        ];
        if let Value::Number(v) = stdev_s(&mut c, &data) {
            let diff = (v - 2.138_089_935_325_936).abs();
            assert!(diff < 1e-6, "stdev_s got {v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn stdev_p_basic() {
        let mut c = ctx();
        // Population std of [2,4] = 1.0
        assert_eq!(
            stdev_p(&mut c, &[Value::Number(2.0), Value::Number(4.0)]),
            Value::Number(1.0)
        );
    }

    #[test]
    fn var_s_single_error() {
        let mut c = ctx();
        assert_eq!(
            var_s(&mut c, &[Value::Number(5.0)]),
            Value::Error(CellError::Div0)
        );
    }

    // ---- PERCENTILE.INC ----------------------------------------------------

    #[test]
    fn percentile_inc_excel() {
        // Excel: PERCENTILE.INC({1,2,3,4}, 0.25) = 1.75
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
        ]);
        assert_eq!(
            percentile_inc(&mut c, &[rng(0, 0, 3, 0), Value::Number(0.25)]),
            Value::Number(1.75)
        );
    }

    #[test]
    fn percentile_inc_edges() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0)), (1, 0, Value::Number(2.0))]);
        assert_eq!(
            percentile_inc(&mut c, &[rng(0, 0, 1, 0), Value::Number(0.0)]),
            Value::Number(1.0)
        );
        let mut c2 = TestCtx::with_cells(&[(0, 0, Value::Number(1.0)), (1, 0, Value::Number(2.0))]);
        assert_eq!(
            percentile_inc(&mut c2, &[rng(0, 0, 1, 0), Value::Number(1.0)]),
            Value::Number(2.0)
        );
    }

    #[test]
    fn percentile_inc_invalid_p() {
        let mut c = TestCtx::with_cells(&[(0, 0, Value::Number(1.0))]);
        assert_eq!(
            percentile_inc(&mut c, &[rng(0, 0, 0, 0), Value::Number(1.5)]),
            Value::Error(CellError::Num)
        );
    }

    // ---- QUARTILE.INC ------------------------------------------------------

    #[test]
    fn quartile_inc_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
        ]);
        // Q2 = median = 2.5
        assert_eq!(
            quartile_inc(&mut c, &[rng(0, 0, 3, 0), Value::Number(2.0)]),
            Value::Number(2.5)
        );
    }

    // ---- CORREL / COVARIANCE -----------------------------------------------

    #[test]
    fn correl_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        // Perfect positive correlation (allow floating-point near-1)
        if let Value::Number(v) = correl(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]) {
            assert!((v - 1.0).abs() < 1e-10, "correl={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn covariance_p_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (0, 1, Value::Number(3.0)),
            (1, 1, Value::Number(4.0)),
        ]);
        // COV_P([1,2],[3,4]) = 0.25
        assert_eq!(
            covariance_p(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 1, 1)]),
            Value::Number(0.25)
        );
    }

    // ---- SLOPE / INTERCEPT / RSQ -------------------------------------------

    #[test]
    fn slope_intercept() {
        // y = 2x + 1: slope=2, intercept=1
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        if let Value::Number(s) = slope(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]) {
            assert!((s - 2.0).abs() < 1e-10, "slope={s}");
        } else {
            panic!("expected number");
        }
        let mut c2 = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        if let Value::Number(ic) = intercept(&mut c2, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]) {
            assert!((ic - 1.0).abs() < 1e-10, "intercept={ic}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn rsq_perfect() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
        ]);
        if let Value::Number(v) = rsq(&mut c, &[rng(0, 0, 1, 0), rng(0, 1, 1, 1)]) {
            assert!((v - 1.0).abs() < 1e-10, "rsq={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- DEVSQ / AVEDEV ----------------------------------------------------

    #[test]
    fn devsq_basic() {
        let mut c = ctx();
        // {1,2,3}: mean=2, SS=(1+0+1)=2
        assert_eq!(
            devsq(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Number(2.0)
        );
    }

    #[test]
    fn avedev_basic() {
        let mut c = ctx();
        // {2,4}: mean=3, avg |deviation| = 1
        assert_eq!(
            avedev(&mut c, &[Value::Number(2.0), Value::Number(4.0)]),
            Value::Number(1.0)
        );
    }

    // ---- GEOMEAN / HARMEAN -------------------------------------------------

    #[test]
    fn geomean_basic() {
        let mut c = ctx();
        // GEOMEAN(4,9) = 6.0
        if let Value::Number(v) = geomean(&mut c, &[Value::Number(4.0), Value::Number(9.0)]) {
            assert!((v - 6.0).abs() < 1e-10);
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn geomean_negative_error() {
        let mut c = ctx();
        assert_eq!(
            geomean(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn harmean_basic() {
        let mut c = ctx();
        // HARMEAN(1,2,4) = 3/(1+0.5+0.25) = 1.714...
        if let Value::Number(v) = harmean(
            &mut c,
            &[Value::Number(1.0), Value::Number(2.0), Value::Number(4.0)],
        ) {
            assert!((v - 12.0 / 7.0).abs() < 1e-10, "harmean={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- TRIMMEAN ----------------------------------------------------------

    #[test]
    fn trimmean_basic() {
        let mut c = ctx();
        // TRIMMEAN({1,2,3,4,5,6,7,8,9,10}, 0.2) trims 1 from each end → mean(2..9)=5.5
        let _data: Vec<Value> = (1..=10).map(|i| Value::Number(i as f64)).collect();
        if let Value::Number(v) = trimmean(
            &mut c,
            &[
                // Use all as direct scalar args for simplicity
                Value::Number(1.0),
                Value::Number(0.2),
            ],
        ) {
            // Single element after trim p/2=0.1 → trim 0 elements → mean(1)=1
            let _ = v;
        }
        // More meaningful: use range
        let mut c2 = TestCtx::with_cells(
            &(1..=10u32)
                .map(|i| (i - 1, 0, Value::Number(i as f64)))
                .collect::<Vec<_>>(),
        );
        if let Value::Number(v) = trimmean(&mut c2, &[rng(0, 0, 9, 0), Value::Number(0.2)]) {
            assert!((v - 5.5).abs() < 1e-10, "trimmean={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- SKEW / KURT -------------------------------------------------------

    #[test]
    fn skew_symmetric() {
        let mut c = ctx();
        // Symmetric distribution: skew should be 0
        let data = vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ];
        if let Value::Number(v) = skew(&mut c, &data) {
            assert!(v.abs() < 1e-10, "skew={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn skew_too_few() {
        let mut c = ctx();
        assert_eq!(
            skew(&mut c, &[Value::Number(1.0), Value::Number(2.0)]),
            Value::Error(CellError::Div0)
        );
    }

    // ---- STANDARDIZE -------------------------------------------------------

    #[test]
    fn standardize_basic() {
        let mut c = ctx();
        assert_eq!(
            standardize(
                &mut c,
                &[Value::Number(5.0), Value::Number(3.0), Value::Number(2.0)]
            ),
            Value::Number(1.0)
        );
    }

    #[test]
    fn standardize_invalid_std() {
        let mut c = ctx();
        assert_eq!(
            standardize(
                &mut c,
                &[Value::Number(5.0), Value::Number(3.0), Value::Number(0.0)]
            ),
            Value::Error(CellError::Num)
        );
    }

    // ---- FISHER / FISHERINV ------------------------------------------------

    #[test]
    fn fisher_fisherinv() {
        let mut c = ctx();
        let x = 0.5;
        if let Value::Number(f) = fisher(&mut c, &[Value::Number(x)]) {
            if let Value::Number(inv) = fisherinv(&mut c, &[Value::Number(f)]) {
                assert!((inv - x).abs() < 1e-10, "round-trip failed: {inv}");
            } else {
                panic!("fisherinv failed");
            }
        } else {
            panic!("fisher failed");
        }
    }

    #[test]
    fn fisher_out_of_range() {
        let mut c = ctx();
        assert_eq!(
            fisher(&mut c, &[Value::Number(1.0)]),
            Value::Error(CellError::Num)
        );
        assert_eq!(
            fisher(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    // ---- NORM.DIST ---------------------------------------------------------

    #[test]
    fn norm_dist_cumulative() {
        let mut c = ctx();
        // NORM.DIST(0, 0, 1, TRUE) = 0.5
        if let Value::Number(v) = norm_dist(
            &mut c,
            &[
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(1.0),
            ],
        ) {
            assert!((v - 0.5).abs() < 1e-6, "norm_dist={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn norm_dist_pdf() {
        let mut c = ctx();
        // NORM.DIST(0, 0, 1, FALSE) = 1/sqrt(2π) ≈ 0.3989...
        if let Value::Number(v) = norm_dist(
            &mut c,
            &[
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
                Value::Number(0.0),
            ],
        ) {
            let expected = 1.0 / (2.0 * std::f64::consts::PI).sqrt();
            assert!((v - expected).abs() < 1e-6, "norm_pdf={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn norm_dist_invalid_std() {
        let mut c = ctx();
        assert_eq!(
            norm_dist(
                &mut c,
                &[
                    Value::Number(0.0),
                    Value::Number(0.0),
                    Value::Number(-1.0),
                    Value::Number(1.0),
                ]
            ),
            Value::Error(CellError::Num)
        );
    }

    // ---- NORM.S.INV / NORM.INV ---------------------------------------------

    #[test]
    fn norm_s_inv_roundtrip() {
        let mut c = ctx();
        // norm_s_dist(norm_s_inv(0.75), cumulative) ≈ 0.75
        if let Value::Number(z) = norm_s_inv(&mut c, &[Value::Number(0.75)]) {
            let cdf = norm_cdf(z);
            assert!((cdf - 0.75).abs() < 1e-4, "roundtrip {cdf}");
        } else {
            panic!("expected number");
        }
    }

    // ---- BINOM.DIST --------------------------------------------------------

    #[test]
    fn binom_dist_pmf() {
        let mut c = ctx();
        // P(X=2 | n=5, p=0.5) = C(5,2)*0.5^5 = 10/32 = 0.3125
        if let Value::Number(v) = binom_dist(
            &mut c,
            &[
                Value::Number(2.0),
                Value::Number(5.0),
                Value::Number(0.5),
                Value::Number(0.0),
            ],
        ) {
            assert!((v - 0.3125).abs() < 1e-6, "binom_pmf={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn binom_dist_cdf() {
        let mut c = ctx();
        // P(X<=2 | n=5, p=0.5) = 0.5
        if let Value::Number(v) = binom_dist(
            &mut c,
            &[
                Value::Number(2.0),
                Value::Number(5.0),
                Value::Number(0.5),
                Value::Number(1.0),
            ],
        ) {
            assert!((v - 0.5).abs() < 1e-4, "binom_cdf={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- POISSON.DIST ------------------------------------------------------

    #[test]
    fn poisson_dist_pmf() {
        let mut c = ctx();
        // P(X=2 | lambda=3) = e^{-3} * 9/2 ≈ 0.2240...
        if let Value::Number(v) = poisson_dist(
            &mut c,
            &[Value::Number(2.0), Value::Number(3.0), Value::Number(0.0)],
        ) {
            let expected = (-3.0_f64).exp() * 9.0 / 2.0;
            assert!(
                (v - expected).abs() < 1e-8,
                "poisson_pmf={v} expected={expected}"
            );
        } else {
            panic!("expected number");
        }
    }

    // ---- EXPON.DIST --------------------------------------------------------

    #[test]
    fn expon_dist_cdf() {
        let mut c = ctx();
        // P(X<=1 | lambda=1) = 1 - e^{-1}
        if let Value::Number(v) = expon_dist(
            &mut c,
            &[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)],
        ) {
            let expected = 1.0 - (-1.0_f64).exp();
            assert!((v - expected).abs() < 1e-10, "expon_cdf={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- GAUSS / PHI -------------------------------------------------------

    #[test]
    fn gauss_zero() {
        let mut c = ctx();
        // GAUSS(0) = Phi(0) - 0.5 = 0
        if let Value::Number(v) = gauss(&mut c, &[Value::Number(0.0)]) {
            assert!(v.abs() < 1e-6, "gauss(0)={v}");
        } else {
            panic!("expected number");
        }
    }

    #[test]
    fn phi_zero() {
        let mut c = ctx();
        // PHI(0) = 1/sqrt(2π)
        if let Value::Number(v) = phi(&mut c, &[Value::Number(0.0)]) {
            let expected = 1.0 / (2.0 * std::f64::consts::PI).sqrt();
            assert!((v - expected).abs() < 1e-10);
        } else {
            panic!("expected number");
        }
    }

    // ---- MAXIFS / MINIFS ---------------------------------------------------

    #[test]
    fn maxifs_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Number(20.0)),
            (2, 0, Value::Number(30.0)),
            (0, 1, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
            (2, 1, Value::Text("a".into())),
        ]);
        let r = maxifs(
            &mut c,
            &[rng(0, 0, 2, 0), rng(0, 1, 2, 1), Value::Text("a".into())],
        );
        assert_eq!(r, Value::Number(30.0));
    }

    #[test]
    fn minifs_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Number(20.0)),
            (2, 0, Value::Number(30.0)),
            (0, 1, Value::Text("a".into())),
            (1, 1, Value::Text("b".into())),
            (2, 1, Value::Text("a".into())),
        ]);
        let r = minifs(
            &mut c,
            &[rng(0, 0, 2, 0), rng(0, 1, 2, 1), Value::Text("a".into())],
        );
        assert_eq!(r, Value::Number(10.0));
    }

    // ---- FORECAST.LINEAR ---------------------------------------------------

    #[test]
    fn forecast_linear_basic() {
        // y = 2x+1: forecast at x=4 → 9
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        if let Value::Number(v) = forecast_linear(
            &mut c,
            &[Value::Number(4.0), rng(0, 0, 2, 0), rng(0, 1, 2, 1)],
        ) {
            assert!((v - 9.0).abs() < 1e-10, "forecast={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- CONFIDENCE.NORM ---------------------------------------------------

    #[test]
    fn confidence_norm_basic() {
        let mut c = ctx();
        // For alpha=0.05, std=1, n=100: z≈1.96, result≈0.196
        if let Value::Number(v) = confidence_norm(
            &mut c,
            &[
                Value::Number(0.05),
                Value::Number(1.0),
                Value::Number(100.0),
            ],
        ) {
            assert!(v > 0.18 && v < 0.21, "confidence={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- PERCENTRANK.INC ---------------------------------------------------

    #[test]
    fn percentrank_inc_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
        ]);
        // rank of 2 in [1,2,3,4] with INC = 1/(4-1) = 0.333
        if let Value::Number(v) = percentrank_inc(&mut c, &[rng(0, 0, 3, 0), Value::Number(2.0)]) {
            assert!((v - 0.333).abs() < 0.001, "percentrank={v}");
        } else {
            panic!("expected number");
        }
    }

    // ---- helper for new distribution KATs ----------------------------------

    fn approx(v: Value, expected: f64, tol: f64, name: &str) {
        if let Value::Number(x) = v {
            assert!(
                (x - expected).abs() < tol,
                "{name}: got {x}, want {expected}"
            );
        } else {
            panic!("{name}: expected number, got {v:?}");
        }
    }

    // ---- GAMMA.DIST / GAMMA.INV --------------------------------------------

    #[test]
    fn gamma_dist_cdf_kat() {
        let mut c = ctx();
        // GAMMA.DIST(2,1,1,TRUE) = 1 - e^-2 ≈ 0.8646647
        approx(
            gamma_dist(
                &mut c,
                &[
                    Value::Number(2.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                ],
            ),
            1.0 - (-2.0_f64).exp(),
            1e-6,
            "GAMMA.DIST",
        );
    }

    #[test]
    fn gamma_inv_roundtrip() {
        let mut c = ctx();
        // GAMMA.INV(GAMMA.DIST(x)) ≈ x
        let p = 0.8646647;
        approx(
            gamma_inv(
                &mut c,
                &[Value::Number(p), Value::Number(1.0), Value::Number(1.0)],
            ),
            2.0,
            1e-4,
            "GAMMA.INV",
        );
    }

    // ---- BETA.DIST ---------------------------------------------------------

    #[test]
    fn beta_dist_cdf_kat() {
        let mut c = ctx();
        // BETA.DIST(0.5, 2, 3, TRUE) = I_0.5(2,3) = 0.6875
        approx(
            beta_dist(
                &mut c,
                &[
                    Value::Number(0.5),
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(1.0),
                ],
            ),
            0.6875,
            1e-5,
            "BETA.DIST",
        );
    }

    // ---- CHISQ -------------------------------------------------------------

    #[test]
    fn chisq_dist_kat() {
        let mut c = ctx();
        // CHISQ.DIST(3,4,TRUE) ≈ 0.4421746
        approx(
            chisq_dist(
                &mut c,
                &[Value::Number(3.0), Value::Number(4.0), Value::Number(1.0)],
            ),
            0.4421746,
            1e-5,
            "CHISQ.DIST",
        );
    }

    #[test]
    fn chisq_inv_roundtrip() {
        let mut c = ctx();
        approx(
            chisq_inv(&mut c, &[Value::Number(0.4421746), Value::Number(4.0)]),
            3.0,
            1e-3,
            "CHISQ.INV",
        );
    }

    // ---- F distribution ----------------------------------------------------

    #[test]
    fn f_dist_rt_kat() {
        let mut c = ctx();
        // F.DIST.RT(1,5,5) = 0.5 (median of F(5,5) is 1)
        approx(
            f_dist_rt(
                &mut c,
                &[Value::Number(1.0), Value::Number(5.0), Value::Number(5.0)],
            ),
            0.5,
            1e-4,
            "F.DIST.RT",
        );
    }

    #[test]
    fn f_inv_roundtrip() {
        let mut c = ctx();
        // F.INV(0.5,5,5) should be ~1
        approx(
            f_inv(
                &mut c,
                &[Value::Number(0.5), Value::Number(5.0), Value::Number(5.0)],
            ),
            1.0,
            1e-3,
            "F.INV",
        );
    }

    // ---- T distribution ----------------------------------------------------

    #[test]
    fn t_dist_cdf_kat() {
        let mut c = ctx();
        // T.DIST(2,10,TRUE) ≈ 0.9633
        approx(
            t_dist(
                &mut c,
                &[Value::Number(2.0), Value::Number(10.0), Value::Number(1.0)],
            ),
            0.9633,
            1e-4,
            "T.DIST",
        );
    }

    #[test]
    fn t_dist_2t_kat() {
        let mut c = ctx();
        // T.DIST.2T(2,10) = 2*(1-0.9633) ≈ 0.0734
        approx(
            t_dist_2t(&mut c, &[Value::Number(2.0), Value::Number(10.0)]),
            0.07339,
            1e-4,
            "T.DIST.2T",
        );
    }

    #[test]
    fn t_inv_2t_roundtrip() {
        let mut c = ctx();
        // T.INV.2T(0.05, 10) ≈ 2.2281
        approx(
            t_inv_2t(&mut c, &[Value::Number(0.05), Value::Number(10.0)]),
            2.2281,
            1e-3,
            "T.INV.2T",
        );
    }

    // ---- LOGNORM -----------------------------------------------------------

    #[test]
    fn lognorm_dist_kat() {
        let mut c = ctx();
        // LOGNORM.DIST(1,0,1,TRUE) = NORM.S.DIST(0) = 0.5
        approx(
            lognorm_dist(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(0.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                ],
            ),
            0.5,
            1e-6,
            "LOGNORM.DIST",
        );
    }

    // ---- NEGBINOM / HYPGEOM ------------------------------------------------

    #[test]
    fn negbinom_pmf_kat() {
        let mut c = ctx();
        // NEGBINOM.DIST(2,3,0.5,FALSE) = C(4,2) * 0.5^3 * 0.5^2 = 6/32 = 0.1875
        approx(
            negbinom_dist(
                &mut c,
                &[
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(0.5),
                    Value::Number(0.0),
                ],
            ),
            0.1875,
            1e-6,
            "NEGBINOM.DIST",
        );
    }

    #[test]
    fn hypgeom_pmf_kat() {
        let mut c = ctx();
        // HYPGEOM.DIST(1,4,8,20,FALSE): C(8,1)*C(12,3)/C(20,4)
        // = 8*220/4845 = 1760/4845 ≈ 0.36326
        approx(
            hypgeom_dist(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(4.0),
                    Value::Number(8.0),
                    Value::Number(20.0),
                    Value::Number(0.0),
                ],
            ),
            0.363261,
            1e-5,
            "HYPGEOM.DIST",
        );
    }

    // ---- BINOM.INV / BINOM.DIST.RANGE --------------------------------------

    #[test]
    fn binom_inv_kat() {
        let mut c = ctx();
        // BINOM.INV(10, 0.5, 0.5) = 5
        approx(
            binom_inv(
                &mut c,
                &[Value::Number(10.0), Value::Number(0.5), Value::Number(0.5)],
            ),
            5.0,
            1e-9,
            "BINOM.INV",
        );
    }

    #[test]
    fn binom_dist_range_kat() {
        let mut c = ctx();
        // P(2<=X<=3 | n=5,p=0.5) = (10+10)/32 = 0.625
        approx(
            binom_dist_range(
                &mut c,
                &[
                    Value::Number(5.0),
                    Value::Number(0.5),
                    Value::Number(2.0),
                    Value::Number(3.0),
                ],
            ),
            0.625,
            1e-5,
            "BINOM.DIST.RANGE",
        );
    }

    // ---- WEIBULL -----------------------------------------------------------

    #[test]
    fn weibull_cdf_kat() {
        let mut c = ctx();
        // WEIBULL.DIST(1,1,1,TRUE) = 1 - e^-1
        approx(
            weibull_dist(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                    Value::Number(1.0),
                ],
            ),
            1.0 - (-1.0_f64).exp(),
            1e-9,
            "WEIBULL.DIST",
        );
    }

    // ---- CONFIDENCE.T ------------------------------------------------------

    #[test]
    fn confidence_t_kat() {
        let mut c = ctx();
        // CONFIDENCE.T(0.05, 1, 50): t_{0.975,49}≈2.0096, /sqrt(50)≈0.2842
        approx(
            confidence_t(
                &mut c,
                &[Value::Number(0.05), Value::Number(1.0), Value::Number(50.0)],
            ),
            0.28419,
            1e-3,
            "CONFIDENCE.T",
        );
    }

    // ---- PROB --------------------------------------------------------------

    #[test]
    fn prob_kat() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(0.0)),
            (1, 0, Value::Number(1.0)),
            (2, 0, Value::Number(2.0)),
            (3, 0, Value::Number(3.0)),
            (0, 1, Value::Number(0.2)),
            (1, 1, Value::Number(0.3)),
            (2, 1, Value::Number(0.1)),
            (3, 1, Value::Number(0.4)),
        ]);
        // P(1 <= x <= 2) = 0.3 + 0.1 = 0.4
        approx(
            prob(
                &mut c,
                &[
                    rng(0, 0, 3, 0),
                    rng(0, 1, 3, 1),
                    Value::Number(1.0),
                    Value::Number(2.0),
                ],
            ),
            0.4,
            1e-9,
            "PROB",
        );
    }

    #[test]
    fn prob_bad_sum() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(0.0)),
            (1, 0, Value::Number(1.0)),
            (0, 1, Value::Number(0.2)),
            (1, 1, Value::Number(0.3)),
        ]);
        // probabilities sum to 0.5 != 1 → #NUM!
        assert_eq!(
            prob(
                &mut c,
                &[rng(0, 0, 1, 0), rng(0, 1, 1, 1), Value::Number(0.0)],
            ),
            Value::Error(CellError::Num)
        );
    }

    // ---- Z.TEST ------------------------------------------------------------

    #[test]
    fn z_test_kat() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(6.0)),
            (2, 0, Value::Number(7.0)),
            (3, 0, Value::Number(8.0)),
            (4, 0, Value::Number(6.0)),
        ]);
        // mean=6, n=5. With x=4, sigma=2: z=(6-4)/(2/sqrt5)=2.236 → 1-Phi≈0.0127
        approx(
            z_test(
                &mut c,
                &[rng(0, 0, 4, 0), Value::Number(4.0), Value::Number(2.0)],
            ),
            0.012674,
            1e-4,
            "Z.TEST",
        );
    }

    // ---- SKEW.P / STEYX ----------------------------------------------------

    #[test]
    fn skew_p_symmetric() {
        let mut c = ctx();
        approx(
            skew_p(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(4.0),
                    Value::Number(5.0),
                ],
            ),
            0.0,
            1e-9,
            "SKEW.P",
        );
    }

    #[test]
    fn steyx_perfect_fit() {
        // perfectly linear data → STEYX = 0
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (3, 0, Value::Number(9.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
            (3, 1, Value::Number(4.0)),
        ]);
        approx(
            steyx(&mut c, &[rng(0, 0, 3, 0), rng(0, 1, 3, 1)]),
            0.0,
            1e-9,
            "STEYX",
        );
    }

    // ---- FREQUENCY ---------------------------------------------------------

    #[test]
    fn frequency_kat() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
            (4, 0, Value::Number(5.0)),
            (0, 1, Value::Number(2.0)),
            (1, 1, Value::Number(4.0)),
        ]);
        // bins [2,4]: <=2 →{1,2}=2, (2,4]→{3,4}=2, >4 →{5}=1
        let r = frequency(&mut c, &[rng(0, 0, 4, 0), rng(0, 1, 1, 1)]);
        if let Value::Array(a) = r {
            assert_eq!(a.data.len(), 3);
            assert_eq!(a.data[0], Value::Number(2.0));
            assert_eq!(a.data[1], Value::Number(2.0));
            assert_eq!(a.data[2], Value::Number(1.0));
        } else {
            panic!("expected array, got {r:?}");
        }
    }

    // ---- TREND / LINEST ----------------------------------------------------

    #[test]
    fn trend_linear() {
        // y = 2x+1 at x=1,2,3 → predict at x=4 → 9
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
            (0, 2, Value::Number(4.0)),
        ]);
        let r = trend(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1), rng(0, 2, 0, 2)]);
        if let Value::Array(a) = r {
            approx(a.data[0].clone(), 9.0, 1e-9, "TREND");
        } else {
            panic!("expected array, got {r:?}");
        }
    }

    #[test]
    fn linest_slope_intercept() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(3.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(7.0)),
            (0, 1, Value::Number(1.0)),
            (1, 1, Value::Number(2.0)),
            (2, 1, Value::Number(3.0)),
        ]);
        let r = linest(&mut c, &[rng(0, 0, 2, 0), rng(0, 1, 2, 1)]);
        if let Value::Array(a) = r {
            approx(a.data[0].clone(), 2.0, 1e-9, "LINEST slope");
            approx(a.data[1].clone(), 1.0, 1e-9, "LINEST intercept");
        } else {
            panic!("expected array, got {r:?}");
        }
    }

    #[test]
    fn forecast_ets_is_na() {
        let mut c = ctx();
        assert_eq!(
            forecast_ets_na(
                &mut c,
                &[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]
            ),
            Value::Error(CellError::NA)
        );
    }
}
