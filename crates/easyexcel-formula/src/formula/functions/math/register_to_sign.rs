/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn register(r: &mut Registry) {
    // aggregation
    r.add("SUM", 1, VARIADIC, false, sum);
    r.add("PRODUCT", 1, VARIADIC, false, product);
    r.add("SUMSQ", 1, VARIADIC, false, sumsq);
    r.add("SUMIF", 2, 3, false, sumif);
    r.add("SUMIFS", 3, VARIADIC, false, sumifs);
    r.add("SUMPRODUCT", 1, VARIADIC, false, sumproduct);
    r.add("PERCENTOF", 2, 2, false, percentof);

    // rounding
    r.add("ROUND", 2, 2, false, round);
    r.add("ROUNDUP", 2, 2, false, roundup);
    r.add("ROUNDDOWN", 2, 2, false, rounddown);
    r.add("MROUND", 2, 2, false, mround);
    r.add("TRUNC", 1, 2, false, trunc);
    r.add("INT", 1, 1, false, int_fn);
    r.add("CEILING.MATH", 1, 3, false, ceiling_math);
    r.add("FLOOR.MATH", 1, 3, false, floor_math);
    r.add("CEILING", 2, 2, false, ceiling);
    r.add("FLOOR", 2, 2, false, floor);
    r.add("CEILING.PRECISE", 1, 2, false, ceiling_precise);
    r.add("FLOOR.PRECISE", 1, 2, false, floor_precise);
    r.add("ISO.CEILING", 1, 2, false, ceiling_precise);
    r.add("EVEN", 1, 1, false, even);
    r.add("ODD", 1, 1, false, odd);

    // basic
    r.add("ABS", 1, 1, false, |_, a| unary(a, f64::abs));
    r.add("SIGN", 1, 1, false, sign);
    r.add("MOD", 2, 2, false, mod_fn);
    r.add("QUOTIENT", 2, 2, false, quotient);
    r.add("POWER", 2, 2, false, power);
    r.add("SQRT", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if x < 0.0 {
                Err(CellError::Num)
            } else {
                Ok(x.sqrt())
            }
        })
    });
    r.add("SQRTPI", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if x < 0.0 {
                Err(CellError::Num)
            } else {
                Ok((x * std::f64::consts::PI).sqrt())
            }
        })
    });
    r.add("EXP", 1, 1, false, |_, a| unary(a, f64::exp));
    r.add("LN", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if x <= 0.0 {
                Err(CellError::Num)
            } else {
                Ok(x.ln())
            }
        })
    });
    r.add("LOG10", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if x <= 0.0 {
                Err(CellError::Num)
            } else {
                Ok(x.log10())
            }
        })
    });
    r.add("LOG", 1, 2, false, log);

    // constants
    r.add("PI", 0, 0, false, |_, _| {
        Value::Number(std::f64::consts::PI)
    });

    // trig
    r.add("SIN", 1, 1, false, |_, a| unary(a, f64::sin));
    r.add("COS", 1, 1, false, |_, a| unary(a, f64::cos));
    r.add("TAN", 1, 1, false, |_, a| unary(a, f64::tan));
    r.add("ASIN", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if (-1.0..=1.0).contains(&x) {
                Ok(x.asin())
            } else {
                Err(CellError::Num)
            }
        })
    });
    r.add("ACOS", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if (-1.0..=1.0).contains(&x) {
                Ok(x.acos())
            } else {
                Err(CellError::Num)
            }
        })
    });
    r.add("ATAN", 1, 1, false, |_, a| unary(a, f64::atan));
    r.add("ATAN2", 2, 2, false, atan2);
    r.add("SINH", 1, 1, false, |_, a| unary(a, f64::sinh));
    r.add("COSH", 1, 1, false, |_, a| unary(a, f64::cosh));
    r.add("TANH", 1, 1, false, |_, a| unary(a, f64::tanh));
    r.add("ASINH", 1, 1, false, |_, a| unary(a, f64::asinh));
    r.add("ACOSH", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if x < 1.0 {
                Err(CellError::Num)
            } else {
                Ok(x.acosh())
            }
        })
    });
    r.add("ATANH", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if x.abs() >= 1.0 {
                Err(CellError::Num)
            } else {
                Ok(x.atanh())
            }
        })
    });
    r.add("SEC", 1, 1, false, |_, a| unary(a, |x| 1.0 / x.cos()));
    r.add("CSC", 1, 1, false, |_, a| unary(a, |x| 1.0 / x.sin()));
    r.add("COT", 1, 1, false, |_, a| unary(a, |x| 1.0 / x.tan()));
    r.add("SECH", 1, 1, false, |_, a| unary(a, |x| 1.0 / x.cosh()));
    r.add("CSCH", 1, 1, false, |_, a| unary(a, |x| 1.0 / x.sinh()));
    r.add("COTH", 1, 1, false, |_, a| unary(a, |x| 1.0 / x.tanh()));
    r.add("ACOT", 1, 1, false, |_, a| {
        unary(a, |x| std::f64::consts::FRAC_PI_2 - x.atan())
    });
    r.add("ACOTH", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if x.abs() <= 1.0 {
                Err(CellError::Num)
            } else {
                Ok((1.0 / x).atanh())
            }
        })
    });
    r.add("DEGREES", 1, 1, false, |_, a| unary(a, f64::to_degrees));
    r.add("RADIANS", 1, 1, false, |_, a| unary(a, f64::to_radians));

    // combinatorics
    r.add("FACT", 1, 1, false, fact);
    r.add("FACTDOUBLE", 1, 1, false, factdouble);
    r.add("COMBIN", 2, 2, false, combin);
    r.add("COMBINA", 2, 2, false, combina);
    r.add("PERMUT", 2, 2, false, permut);
    r.add("PERMUTATIONA", 2, 2, false, permutationa);
    r.add("GCD", 1, VARIADIC, false, gcd);
    r.add("LCM", 1, VARIADIC, false, lcm);

    // random (volatile)
    r.add("RAND", 0, 0, true, |_, _| Value::Number(next_rand()));
    r.add("RANDBETWEEN", 2, 2, true, randbetween);

    // misc
    r.add("BASE", 2, 3, false, base);
    r.add("DECIMAL", 2, 2, false, decimal);
    r.add("ARABIC", 1, 1, false, arabic);
    r.add("ROMAN", 1, 2, false, roman);
    r.add("GAMMALN", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if x <= 0.0 {
                Err(CellError::Num)
            } else {
                Ok(ln_gamma(x))
            }
        })
    });

    // combinatorics (additional)
    r.add("MULTINOMIAL", 1, VARIADIC, false, multinomial);

    // paired-array functions
    r.add("SUMX2MY2", 2, 2, false, sumx2my2);
    r.add("SUMX2PY2", 2, 2, false, sumx2py2);
    r.add("SUMXMY2", 2, 2, false, sumxmy2);

    // series
    r.add("SERIESSUM", 4, 4, false, seriessum);

    // aggregation
    r.add("SUBTOTAL", 2, VARIADIC, false, subtotal);
    r.add("AGGREGATE", 3, VARIADIC, false, aggregate);

    // matrix
    r.add("MDETERM", 1, 1, false, mdeterm);
    r.add("MINVERSE", 1, 1, false, minverse);
    r.add("MMULT", 2, 2, false, mmult);
    r.add("MUNIT", 1, 1, false, munit);

    // gamma
    r.add("GAMMA", 1, 1, false, gamma);
    r.add("GAMMALN.PRECISE", 1, 1, false, |_, a| {
        // PARITY: identical to GAMMALN; Excel treats these as equivalent.
        guarded(a, |x| {
            if x <= 0.0 {
                Err(CellError::Num)
            } else {
                Ok(ln_gamma(x))
            }
        })
    });
}

// --- helpers ---------------------------------------------------------------

fn n(args: &[Value], i: usize) -> Result<f64, CellError> {
    to_number(&args[i])
}

fn unary(args: &[Value], f: impl Fn(f64) -> f64) -> Value {
    match n(args, 0) {
        Ok(x) => {
            let r = f(x);
            if r.is_finite() {
                Value::Number(r)
            } else {
                Value::Error(CellError::Num)
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn guarded(args: &[Value], f: impl Fn(f64) -> Result<f64, CellError>) -> Value {
    match n(args, 0) {
        Ok(x) => match f(x) {
            Ok(r) if r.is_finite() => Value::Number(r),
            Ok(_) => Value::Error(CellError::Num),
            Err(e) => Value::Error(e),
        },
        Err(e) => Value::Error(e),
    }
}

// --- aggregation -----------------------------------------------------------

fn sum(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => Value::Number(ns.iter().sum()),
        Err(e) => Value::Error(e),
    }
}

/// `PERCENTOF(data_subset, data_all)` = SUM(subset) / SUM(all). Returns
/// `#DIV/0!` when the total sums to zero.
fn percentof(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let subset = match collect_numbers(ctx, &args[0..1], false) {
        Ok(ns) => ns.iter().sum::<f64>(),
        Err(e) => return Value::Error(e),
    };
    let all = match collect_numbers(ctx, &args[1..2], false) {
        Ok(ns) => ns.iter().sum::<f64>(),
        Err(e) => return Value::Error(e),
    };
    if all == 0.0 {
        return Value::Error(CellError::Div0);
    }
    Value::Number(subset / all)
}

fn product(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.is_empty() {
                Value::Number(0.0)
            } else {
                Value::Number(ns.iter().product())
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn sumsq(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => Value::Number(ns.iter().map(|x| x * x).sum()),
        Err(e) => Value::Error(e),
    }
}

/// Materialize a value to a flat list of scalars for criteria evaluation.
fn flat(ctx: &mut dyn Context, v: &Value) -> Vec<Value> {
    ctx.flatten(v)
}

fn sumif(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let range = flat(ctx, &args[0]);
    let crit = Criteria::parse(&single(&args[1]));
    let sum_range = if args.len() == 3 {
        flat(ctx, &args[2])
    } else {
        range.clone()
    };
    let mut total = 0.0;
    for (i, c) in range.iter().enumerate() {
        if crit.matches(c) {
            if let Some(Value::Number(x)) = sum_range.get(i) {
                total += x;
            } else if let Some(Value::Bool(b)) = sum_range.get(i) {
                total += if *b { 1.0 } else { 0.0 };
            }
        }
    }
    Value::Number(total)
}

fn sumifs(ctx: &mut dyn Context, args: &[Value]) -> Value {
    // args: sum_range, (crit_range, crit)+
    if args.len() < 3 || !(args.len() - 1).is_multiple_of(2) {
        return Value::Error(CellError::Value);
    }
    let sum_range = flat(ctx, &args[0]);
    let mut pairs = Vec::new();
    let mut i = 1;
    while i + 1 < args.len() {
        let rng = flat(ctx, &args[i]);
        let crit = Criteria::parse(&single(&args[i + 1]));
        pairs.push((rng, crit));
        i += 2;
    }
    let mut total = 0.0;
    for (idx, sv) in sum_range.iter().enumerate() {
        let ok = pairs
            .iter()
            .all(|(rng, crit)| rng.get(idx).is_some_and(|c| crit.matches(c)));
        if ok && let Value::Number(x) = sv {
            total += x;
        }
    }
    Value::Number(total)
}

fn sumproduct(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let arrays: Vec<Vec<f64>> = args
        .iter()
        .map(|a| {
            flat(ctx, a)
                .iter()
                .map(|v| match v {
                    Value::Number(n) => *n,
                    Value::Bool(b) if *b => 1.0,
                    _ => 0.0,
                })
                .collect()
        })
        .collect();
    let len = arrays.first().map_or(0, std::vec::Vec::len);
    if arrays.iter().any(|a| a.len() != len) {
        return Value::Error(CellError::Value);
    }
    let mut total = 0.0;
    for i in 0..len {
        let mut prod = 1.0;
        for a in &arrays {
            prod *= a[i];
        }
        total += prod;
    }
    Value::Number(total)
}

fn single(v: &Value) -> Value {
    match v {
        Value::Array(a) => a.data.first().cloned().unwrap_or(Value::Empty),
        other => other.clone(),
    }
}

// --- rounding --------------------------------------------------------------

fn round_to(x: f64, digits: f64, mode: RoundMode) -> f64 {
    let factor = 10f64.powf(digits);
    let scaled = x * factor;
    let rounded = match mode {
        RoundMode::Half => {
            // round half away from zero
            if scaled >= 0.0 {
                (scaled + 0.5).floor()
            } else {
                (scaled - 0.5).ceil()
            }
        }
        RoundMode::Up => {
            if scaled >= 0.0 {
                scaled.ceil()
            } else {
                scaled.floor()
            }
        }
        RoundMode::Down => scaled.trunc(),
    };
    rounded / factor
}

enum RoundMode {
    Half,
    Up,
    Down,
}

fn round(_: &mut dyn Context, a: &[Value]) -> Value {
    bin(a, |x, d| round_to(x, d, RoundMode::Half))
}
fn roundup(_: &mut dyn Context, a: &[Value]) -> Value {
    bin(a, |x, d| round_to(x, d, RoundMode::Up))
}
fn rounddown(_: &mut dyn Context, a: &[Value]) -> Value {
    bin(a, |x, d| round_to(x, d, RoundMode::Down))
}

fn bin(a: &[Value], f: impl Fn(f64, f64) -> f64) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(x), Ok(y)) => {
            let r = f(x, y);
            if r.is_finite() {
                Value::Number(r)
            } else {
                Value::Error(CellError::Num)
            }
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn mround(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(x), Ok(m)) => {
            if m == 0.0 {
                return Value::Number(0.0);
            }
            if (x < 0.0) != (m < 0.0) {
                return Value::Error(CellError::Num);
            }
            Value::Number((x / m).round() * m)
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn trunc(_: &mut dyn Context, a: &[Value]) -> Value {
    let digits = if a.len() == 2 {
        match n(a, 1) {
            Ok(d) => d,
            Err(e) => return Value::Error(e),
        }
    } else {
        0.0
    };
    bin(&[a[0].clone(), Value::Number(digits)], |x, d| {
        round_to(x, d, RoundMode::Down)
    })
}

fn int_fn(_: &mut dyn Context, a: &[Value]) -> Value {
    match n(a, 0) {
        Ok(x) => Value::Number(x.floor()),
        Err(e) => Value::Error(e),
    }
}

fn ceiling_math(_: &mut dyn Context, a: &[Value]) -> Value {
    let x = match n(a, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let sig = if a.len() >= 2 {
        match n(a, 1) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        1.0
    };
    let mode = if a.len() >= 3 {
        match n(a, 2) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        0.0
    };
    if sig == 0.0 {
        return Value::Number(0.0);
    }
    let s = sig.abs();
    let r = if x >= 0.0 {
        (x / s).ceil() * s
    } else if mode != 0.0 {
        (x / s).floor() * s
    } else {
        (x / s).ceil() * s
    };
    Value::Number(r)
}

fn floor_math(_: &mut dyn Context, a: &[Value]) -> Value {
    let x = match n(a, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let sig = if a.len() >= 2 {
        match n(a, 1) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        1.0
    };
    let mode = if a.len() >= 3 {
        match n(a, 2) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        0.0
    };
    if sig == 0.0 {
        return Value::Number(0.0);
    }
    let s = sig.abs();
    let r = if x >= 0.0 {
        (x / s).floor() * s
    } else if mode != 0.0 {
        (x / s).ceil() * s
    } else {
        (x / s).floor() * s
    };
    Value::Number(r)
}

fn ceiling(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(x), Ok(s)) => {
            if s == 0.0 {
                return Value::Number(0.0);
            }
            if (x > 0.0) && (s < 0.0) {
                return Value::Error(CellError::Num);
            }
            Value::Number((x / s).ceil() * s)
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn floor(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(x), Ok(s)) => {
            if s == 0.0 {
                return Value::Error(CellError::Div0);
            }
            if (x > 0.0) && (s < 0.0) {
                return Value::Error(CellError::Num);
            }
            Value::Number((x / s).floor() * s)
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn ceiling_precise(_: &mut dyn Context, a: &[Value]) -> Value {
    let x = match n(a, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let s = if a.len() >= 2 {
        match n(a, 1) {
            Ok(v) => v.abs(),
            Err(e) => return Value::Error(e),
        }
    } else {
        1.0
    };
    if s == 0.0 {
        return Value::Number(0.0);
    }
    Value::Number((x / s).ceil() * s)
}

fn floor_precise(_: &mut dyn Context, a: &[Value]) -> Value {
    let x = match n(a, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let s = if a.len() >= 2 {
        match n(a, 1) {
            Ok(v) => v.abs(),
            Err(e) => return Value::Error(e),
        }
    } else {
        1.0
    };
    if s == 0.0 {
        return Value::Number(0.0);
    }
    Value::Number((x / s).floor() * s)
}

fn even(_: &mut dyn Context, a: &[Value]) -> Value {
    match n(a, 0) {
        Ok(x) => {
            let r = if x >= 0.0 {
                (x / 2.0).ceil() * 2.0
            } else {
                (x / 2.0).floor() * 2.0
            };
            Value::Number(r)
        }
        Err(e) => Value::Error(e),
    }
}

fn odd(_: &mut dyn Context, a: &[Value]) -> Value {
    match n(a, 0) {
        Ok(x) => {
            if x == 0.0 {
                return Value::Number(1.0);
            }
            let mut r = if x >= 0.0 { x.ceil() } else { x.floor() };
            if (r as i64) % 2 == 0 {
                r += if x >= 0.0 { 1.0 } else { -1.0 };
            }
            Value::Number(r)
        }
        Err(e) => Value::Error(e),
    }
}

// --- basic -----------------------------------------------------------------

fn sign(_: &mut dyn Context, a: &[Value]) -> Value {
    match n(a, 0) {
        Ok(x) => Value::Number(if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            0.0
        }),
        Err(e) => Value::Error(e),
    }
}

