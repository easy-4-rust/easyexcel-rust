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
        - super::math::ln_gamma(f64::midpoint(d1, d2));
    let ln = 0.5 * (d1 * (d1).ln() + d2 * (d2).ln()) + (d1 / 2.0 - 1.0) * x.ln()
        - f64::midpoint(d1, d2) * (d1 * x + d2).ln()
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

/// Returns x such that `f_cdf(x,d1,d2)` = p.
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
    let lc = super::math::ln_gamma(f64::midpoint(df, 1.0))
        - super::math::ln_gamma(df / 2.0)
        - 0.5 * (df * std::f64::consts::PI).ln();
    (lc - f64::midpoint(df, 1.0) * (1.0 + x * x / df).ln()).exp()
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

