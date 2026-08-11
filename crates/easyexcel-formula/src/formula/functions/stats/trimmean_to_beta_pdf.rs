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
            // 修复: NaN 时 partial_cmp 返回 None，unwrap 会 panic；NaN 视为 Equal
            ns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
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
            let s = variance(&ns, false).map_or(0.0, f64::sqrt);
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
            let s = variance(&ns, false).map_or(0.0, f64::sqrt);
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
    let t = 1.0 / (1.0 + 0.231_641_9 * x);
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
        let mf = f64::from(m);
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
            let an = -f64::from(i) * (f64::from(i) - a);
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

/// Inverse of the regularized incomplete beta `I_x(a,b)` in x: returns x in [0,1]
/// such that `I_x(a,b)` = p. Bisection (monotone). PARITY: ~1e-10 accuracy.
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

