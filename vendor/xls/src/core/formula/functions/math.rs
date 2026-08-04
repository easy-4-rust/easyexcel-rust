//! Math & trigonometry functions.

use std::cell::Cell as StdCell;

use super::{Criteria, Registry, VARIADIC, collect_numbers};
use crate::core::error::CellError;
use crate::core::formula::coerce::to_number;
use crate::core::formula::context::Context;
use crate::core::formula::value::Value;

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
            if !(-1.0..=1.0).contains(&x) {
                Err(CellError::Num)
            } else {
                Ok(x.asin())
            }
        })
    });
    r.add("ACOS", 1, 1, false, |_, a| {
        guarded(a, |x| {
            if !(-1.0..=1.0).contains(&x) {
                Err(CellError::Num)
            } else {
                Ok(x.acos())
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
            .all(|(rng, crit)| rng.get(idx).map(|c| crit.matches(c)).unwrap_or(false));
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
    let len = arrays.first().map(|a| a.len()).unwrap_or(0);
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

fn mod_fn(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(x), Ok(d)) => {
            if d == 0.0 {
                return Value::Error(CellError::Div0);
            }
            // Excel MOD result takes the sign of the divisor.
            let r = x - d * (x / d).floor();
            Value::Number(r)
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn quotient(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(x), Ok(d)) => {
            if d == 0.0 {
                return Value::Error(CellError::Div0);
            }
            Value::Number((x / d).trunc())
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn power(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(x), Ok(y)) => {
            let r = x.powf(y);
            if r.is_finite() {
                Value::Number(r)
            } else {
                Value::Error(CellError::Num)
            }
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn log(_: &mut dyn Context, a: &[Value]) -> Value {
    let x = match n(a, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let base = if a.len() == 2 {
        match n(a, 1) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        }
    } else {
        10.0
    };
    if x <= 0.0 || base <= 0.0 || base == 1.0 {
        return Value::Error(CellError::Num);
    }
    Value::Number(x.log(base))
}

fn atan2(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(x), Ok(y)) => {
            if x == 0.0 && y == 0.0 {
                return Value::Error(CellError::Div0);
            }
            Value::Number(y.atan2(x))
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

// --- combinatorics ---------------------------------------------------------

fn fact(_: &mut dyn Context, a: &[Value]) -> Value {
    match n(a, 0) {
        Ok(x) => {
            if !(0.0..171.0).contains(&x) {
                return Value::Error(CellError::Num);
            }
            let k = x.trunc() as u64;
            let mut r = 1.0;
            for i in 2..=k {
                r *= i as f64;
            }
            Value::Number(r)
        }
        Err(e) => Value::Error(e),
    }
}

fn factdouble(_: &mut dyn Context, a: &[Value]) -> Value {
    match n(a, 0) {
        Ok(x) => {
            if x < -1.0 {
                return Value::Error(CellError::Num);
            }
            let k = x.trunc() as i64;
            let mut r = 1.0;
            let mut i = k;
            while i > 1 {
                r *= i as f64;
                i -= 2;
            }
            Value::Number(r)
        }
        Err(e) => Value::Error(e),
    }
}

fn binom(nn: f64, k: f64) -> Option<f64> {
    if nn < 0.0 || k < 0.0 || k > nn {
        return None;
    }
    let (nn, k) = (nn.trunc() as u64, k.trunc() as u64);
    let k = k.min(nn - k);
    let mut r = 1.0;
    for i in 0..k {
        r = r * (nn - i) as f64 / (i + 1) as f64;
    }
    Some(r)
}

fn combin(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(nn), Ok(k)) => match binom(nn, k) {
            Some(r) => Value::Number(r.round()),
            None => Value::Error(CellError::Num),
        },
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn combina(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(nn), Ok(k)) => {
            if nn < 0.0 || k < 0.0 {
                return Value::Error(CellError::Num);
            }
            match binom(nn + k - 1.0, k) {
                Some(r) => Value::Number(r.round()),
                None => Value::Error(CellError::Num),
            }
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn permut(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(nn), Ok(k)) => {
            if nn < 0.0 || k < 0.0 || k > nn {
                return Value::Error(CellError::Num);
            }
            let (nn, k) = (nn.trunc() as u64, k.trunc() as u64);
            let mut r = 1.0;
            for i in 0..k {
                r *= (nn - i) as f64;
            }
            Value::Number(r)
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn permutationa(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(nn), Ok(k)) => {
            if nn < 0.0 || k < 0.0 {
                return Value::Error(CellError::Num);
            }
            Value::Number(nn.trunc().powf(k.trunc()))
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn gcd2(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd2(b, a % b) }
}

fn gcd(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            let mut g = 0u64;
            for x in ns {
                if x < 0.0 {
                    return Value::Error(CellError::Num);
                }
                g = gcd2(g, x.trunc() as u64);
            }
            Value::Number(g as f64)
        }
        Err(e) => Value::Error(e),
    }
}

fn lcm(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            let mut l = 1u64;
            for x in ns {
                if x < 0.0 {
                    return Value::Error(CellError::Num);
                }
                let v = x.trunc() as u64;
                if v == 0 {
                    return Value::Number(0.0);
                }
                l = l / gcd2(l, v) * v;
            }
            Value::Number(l as f64)
        }
        Err(e) => Value::Error(e),
    }
}

// --- random ----------------------------------------------------------------

thread_local! {
    static RNG_STATE: StdCell<u64> = StdCell::new(seed());
}

fn seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B9);
    nanos | 1
}

fn next_u64() -> u64 {
    RNG_STATE.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        x
    })
}

fn next_rand() -> f64 {
    // 53-bit mantissa fraction in [0,1)
    (next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

fn randbetween(_: &mut dyn Context, a: &[Value]) -> Value {
    match (n(a, 0), n(a, 1)) {
        (Ok(lo), Ok(hi)) => {
            let lo = lo.ceil() as i64;
            let hi = hi.floor() as i64;
            if lo > hi {
                return Value::Error(CellError::Num);
            }
            let span = (hi - lo + 1) as u64;
            let v = lo + (next_u64() % span) as i64;
            Value::Number(v as f64)
        }
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

// --- base / roman ----------------------------------------------------------

fn base(_: &mut dyn Context, a: &[Value]) -> Value {
    let num = match n(a, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let radix = match n(a, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let min_len = if a.len() == 3 {
        match n(a, 2) {
            Ok(v) => v as usize,
            Err(e) => return Value::Error(e),
        }
    } else {
        0
    };
    if num < 0.0 || !(2.0..=36.0).contains(&radix) {
        return Value::Error(CellError::Num);
    }
    let mut v = num.trunc() as u64;
    let radix = radix as u64;
    if v == 0 {
        return Value::Text(format!("{:0>1$}", "0", min_len.max(1)));
    }
    let digits = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut out = Vec::new();
    while v > 0 {
        out.push(digits[(v % radix) as usize]);
        v /= radix;
    }
    out.reverse();
    let mut s = String::from_utf8(out).unwrap();
    while s.len() < min_len {
        s.insert(0, '0');
    }
    Value::Text(s)
}

fn decimal(_: &mut dyn Context, a: &[Value]) -> Value {
    let text = match &a[0] {
        Value::Text(s) => s.clone(),
        Value::Number(n) => crate::core::value::format_number_general(*n),
        _ => return Value::Error(CellError::Value),
    };
    let radix = match n(a, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if !(2.0..=36.0).contains(&radix) {
        return Value::Error(CellError::Num);
    }
    let radix = radix as u32;
    match u64::from_str_radix(text.trim().to_uppercase().as_str(), radix) {
        Ok(v) => Value::Number(v as f64),
        Err(_) => Value::Error(CellError::Num),
    }
}

fn arabic(_: &mut dyn Context, a: &[Value]) -> Value {
    let s = match &a[0] {
        Value::Text(s) => s.trim().to_uppercase(),
        _ => return Value::Error(CellError::Value),
    };
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r.to_string()),
        None => (false, s),
    };
    let val = |c: char| match c {
        'I' => 1,
        'V' => 5,
        'X' => 10,
        'L' => 50,
        'C' => 100,
        'D' => 500,
        'M' => 1000,
        _ => 0,
    };
    let chars: Vec<char> = s.chars().collect();
    let mut total = 0i64;
    for i in 0..chars.len() {
        let cur = val(chars[i]);
        if cur == 0 {
            return Value::Error(CellError::Value);
        }
        let next = chars.get(i + 1).map(|c| val(*c)).unwrap_or(0);
        if cur < next {
            total -= cur;
        } else {
            total += cur;
        }
    }
    Value::Number(if neg { -total } else { total } as f64)
}

fn roman(_: &mut dyn Context, a: &[Value]) -> Value {
    let num = match n(a, 0) {
        Ok(v) => v.trunc() as i64,
        Err(e) => return Value::Error(e),
    };
    if !(0..=3999).contains(&num) {
        return Value::Error(CellError::Value);
    }
    let table = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut v = num;
    let mut out = String::new();
    for (val, sym) in table {
        while v >= val {
            out.push_str(sym);
            v -= val;
        }
    }
    Value::Text(out)
}

// --- MULTINOMIAL -----------------------------------------------------------

fn multinomial(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match collect_numbers(ctx, args, false) {
        Ok(ns) => {
            if ns.iter().any(|&x| x < 0.0) {
                return Value::Error(CellError::Num);
            }
            // factorial(sum) / product(factorial(each))
            let sum: f64 = ns.iter().sum();
            let mut result = factorial_f64(sum);
            for &x in &ns {
                let f = factorial_f64(x);
                if f == 0.0 {
                    return Value::Error(CellError::Num);
                }
                result /= f;
            }
            if result.is_finite() {
                Value::Number(result.round())
            } else {
                Value::Error(CellError::Num)
            }
        }
        Err(e) => Value::Error(e),
    }
}

/// Compute k! as f64 (truncates k to integer). Returns infinity for k >= 171.
fn factorial_f64(k: f64) -> f64 {
    let k = k.trunc() as u64;
    if k == 0 {
        return 1.0;
    }
    let mut r = 1.0f64;
    for i in 2..=k {
        r *= i as f64;
    }
    r
}

// --- paired-array functions ------------------------------------------------

/// Flatten a `Value` to a `Vec<f64>`, treating non-numeric as `#N/A`.
fn flatten_nums(ctx: &mut dyn Context, v: &Value) -> Result<Vec<f64>, CellError> {
    let flat = ctx.flatten(v);
    let mut out = Vec::with_capacity(flat.len());
    for val in flat {
        match val {
            Value::Number(n) => out.push(n),
            Value::Bool(b) => out.push(if b { 1.0 } else { 0.0 }),
            Value::Empty => out.push(0.0),
            Value::Error(e) => return Err(e),
            _ => return Err(CellError::Value),
        }
    }
    Ok(out)
}

fn paired_arrays(ctx: &mut dyn Context, args: &[Value]) -> Result<(Vec<f64>, Vec<f64>), CellError> {
    let xs = flatten_nums(ctx, &args[0])?;
    let ys = flatten_nums(ctx, &args[1])?;
    if xs.len() != ys.len() {
        return Err(CellError::NA);
    }
    Ok((xs, ys))
}

fn sumx2my2(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match paired_arrays(ctx, args) {
        Ok((xs, ys)) => {
            let s: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| x * x - y * y).sum();
            Value::Number(s)
        }
        Err(e) => Value::Error(e),
    }
}

fn sumx2py2(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match paired_arrays(ctx, args) {
        Ok((xs, ys)) => {
            let s: f64 = xs.iter().zip(ys.iter()).map(|(x, y)| x * x + y * y).sum();
            Value::Number(s)
        }
        Err(e) => Value::Error(e),
    }
}

fn sumxmy2(ctx: &mut dyn Context, args: &[Value]) -> Value {
    match paired_arrays(ctx, args) {
        Ok((xs, ys)) => {
            let s: f64 = xs
                .iter()
                .zip(ys.iter())
                .map(|(x, y)| (x - y) * (x - y))
                .sum();
            Value::Number(s)
        }
        Err(e) => Value::Error(e),
    }
}

// --- SERIESSUM -------------------------------------------------------------

fn seriessum(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let x = match n(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let nm = match n(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let m = match n(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let coeffs = match flatten_nums(ctx, &args[3]) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let mut total = 0.0;
    for (i, &c) in coeffs.iter().enumerate() {
        let exp = nm + i as f64 * m;
        total += c * x.powf(exp);
    }
    if total.is_finite() {
        Value::Number(total)
    } else {
        Value::Error(CellError::Num)
    }
}

// --- SUBTOTAL --------------------------------------------------------------

/// Perform a subtotal aggregation (function_num 1-11; 101-111 treated same).
/// PARITY: Does not skip nested SUBTOTAL results — tracking nested calls is not
/// feasible without engine support.
fn subtotal(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let fn_num = match n(args, 0) {
        Ok(v) => v as u32,
        Err(e) => return Value::Error(e),
    };
    // 101-111 → 1-11 (treat same; ignoring manually-hidden rows is not supported)
    let fn_num = if (101..=111).contains(&fn_num) {
        fn_num - 100
    } else {
        fn_num
    };
    let refs = &args[1..];
    let nums: Vec<f64> = {
        let mut out = Vec::new();
        for v in refs {
            for cell in ctx.flatten(v) {
                match cell {
                    Value::Number(n) => out.push(n),
                    Value::Error(e) => return Value::Error(e),
                    _ => {}
                }
            }
        }
        out
    };
    // For COUNTA we also need non-empty count
    let all_flat: Vec<Value> = refs.iter().flat_map(|v| ctx.flatten(v)).collect();

    match fn_num {
        1 => {
            // AVERAGE
            if nums.is_empty() {
                Value::Error(CellError::Div0)
            } else {
                Value::Number(nums.iter().sum::<f64>() / nums.len() as f64)
            }
        }
        2 => Value::Number(nums.len() as f64), // COUNT (numeric cells)
        3 => {
            // COUNTA (non-empty cells)
            let count = all_flat
                .iter()
                .filter(|v| !matches!(v, Value::Empty))
                .count();
            Value::Number(count as f64)
        }
        4 => {
            // MAX
            nums.iter()
                .copied()
                .reduce(f64::max)
                .map(Value::Number)
                .unwrap_or(Value::Number(0.0))
        }
        5 => {
            // MIN
            nums.iter()
                .copied()
                .reduce(f64::min)
                .map(Value::Number)
                .unwrap_or(Value::Number(0.0))
        }
        6 => {
            // PRODUCT
            Value::Number(if nums.is_empty() {
                0.0
            } else {
                nums.iter().product()
            })
        }
        7 => {
            // STDEV (sample)
            sample_stdev(&nums)
        }
        8 => {
            // STDEVP (population)
            pop_stdev(&nums)
        }
        9 => Value::Number(nums.iter().sum()), // SUM
        10 => {
            // VAR (sample)
            sample_var(&nums)
        }
        11 => {
            // VARP (population)
            pop_var(&nums)
        }
        _ => Value::Error(CellError::Value),
    }
}

fn sample_var(nums: &[f64]) -> Value {
    if nums.len() < 2 {
        return Value::Error(CellError::Div0);
    }
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
    Value::Number(var)
}

fn pop_var(nums: &[f64]) -> Value {
    if nums.is_empty() {
        return Value::Error(CellError::Div0);
    }
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let var = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
    Value::Number(var)
}

fn sample_stdev(nums: &[f64]) -> Value {
    match sample_var(nums) {
        Value::Number(v) => Value::Number(v.sqrt()),
        other => other,
    }
}

fn pop_stdev(nums: &[f64]) -> Value {
    match pop_var(nums) {
        Value::Number(v) => Value::Number(v.sqrt()),
        other => other,
    }
}

// --- AGGREGATE -------------------------------------------------------------

/// AGGREGATE(function_num, options, ref1/array, [k])
///
/// function_num 1-11: same as SUBTOTAL (1=AVERAGE … 11=VARP)
/// function_num 12=MEDIAN, 13=MODE.SNGL
/// function_num 14=LARGE, 15=SMALL (require k arg)
/// function_num 16=PERCENTILE.INC, 17=QUARTILE.INC (require k arg)
/// function_num 18=PERCENTILE.EXC, 19=QUARTILE.EXC (require k arg)
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
            .map(Value::Number)
            .unwrap_or(Value::Number(0.0)),
        5 => nums
            .iter()
            .copied()
            .reduce(f64::min)
            .map(Value::Number)
            .unwrap_or(Value::Number(0.0)),
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
        (s[len / 2 - 1] + s[len / 2]) / 2.0
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
    agg_percentile_inc(nums, q as f64 / 4.0)
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
    agg_percentile_exc(nums, q as f64 / 4.0)
}

// --- matrix helpers --------------------------------------------------------

/// Extract a 2D f64 matrix from a Value. Returns (rows, cols, data) or an error.
fn extract_matrix(ctx: &mut dyn Context, v: &Value) -> Result<(usize, usize, Vec<f64>), CellError> {
    let arr = match v {
        Value::Array(a) => a.clone(),
        Value::Ref(r) => ctx.ref_to_array(*r),
        other => {
            // scalar → 1×1 matrix
            match to_number(other) {
                Ok(x) => {
                    return Ok((1, 1, vec![x]));
                }
                Err(e) => return Err(e),
            }
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
            Value::Array(crate::core::formula::value::Array::new(n, n, result))
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
    Value::Array(crate::core::formula::value::Array::new(ar, bc, result))
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
    Value::Array(crate::core::formula::value::Array::new(size, size, data))
}

// --- GAMMA -----------------------------------------------------------------

/// Γ(x) via exp(ln_gamma) with proper sign.
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
mod tests {
    use super::*;
    use crate::core::formula::functions::testutil::{TestCtx, rng};

    #[test]
    fn sum_and_product() {
        let mut c = TestCtx::new();
        assert_eq!(
            sum(
                &mut c,
                &[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]
            ),
            Value::Number(6.0)
        );
        assert_eq!(
            product(&mut c, &[Value::Number(2.0), Value::Number(3.0)]),
            Value::Number(6.0)
        );
    }

    #[test]
    fn sum_ignores_text_in_ranges() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(10.0)),
            (1, 0, Value::Text("x".into())),
            (2, 0, Value::Number(5.0)),
        ]);
        assert_eq!(sum(&mut c, &[rng(0, 0, 2, 0)]), Value::Number(15.0));
    }

    #[test]
    fn sum_coerces_numeric_text_in_ranges() {
        // PARITY: Excel ignores text in ranges; we coerce numeric-looking text
        // ("numbers stored as text") so they still contribute. Non-numeric text
        // ("x") is still skipped.
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Text("6,000.00".into())),
            (1, 0, Value::Text("x".into())),
            (2, 0, Value::Text("500.00".into())),
            (3, 0, Value::Number(5.0)),
        ]);
        assert_eq!(sum(&mut c, &[rng(0, 0, 3, 0)]), Value::Number(6505.0));
    }

    #[test]
    fn rounding() {
        let mut c = TestCtx::new();
        assert_eq!(
            round(&mut c, &[Value::Number(2.5), Value::Number(0.0)]),
            Value::Number(3.0)
        );
        assert_eq!(
            round(&mut c, &[Value::Number(-2.5), Value::Number(0.0)]),
            Value::Number(-3.0)
        );
        assert_eq!(
            round(&mut c, &[Value::Number(9.87654), Value::Number(2.0)]),
            Value::Number(9.88)
        );
        assert_eq!(
            rounddown(&mut c, &[Value::Number(3.99), Value::Number(0.0)]),
            Value::Number(3.0)
        );
    }

    #[test]
    fn mod_and_int() {
        let mut c = TestCtx::new();
        assert_eq!(
            mod_fn(&mut c, &[Value::Number(-3.0), Value::Number(2.0)]),
            Value::Number(1.0)
        );
        assert_eq!(int_fn(&mut c, &[Value::Number(-2.5)]), Value::Number(-3.0));
        assert_eq!(
            mod_fn(&mut c, &[Value::Number(5.0), Value::Number(0.0)]),
            Value::Error(CellError::Div0)
        );
    }

    #[test]
    fn sumif_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(5.0)),
            (2, 0, Value::Number(10.0)),
        ]);
        let r = sumif(&mut c, &[rng(0, 0, 2, 0), Value::Text(">3".into())]);
        assert_eq!(r, Value::Number(15.0));
    }

    #[test]
    fn fact_and_combin() {
        let mut c = TestCtx::new();
        assert_eq!(fact(&mut c, &[Value::Number(5.0)]), Value::Number(120.0));
        assert_eq!(
            combin(&mut c, &[Value::Number(5.0), Value::Number(2.0)]),
            Value::Number(10.0)
        );
        assert_eq!(
            fact(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    #[test]
    fn roman_arabic() {
        let mut c = TestCtx::new();
        assert_eq!(
            roman(&mut c, &[Value::Number(1994.0)]),
            Value::Text("MCMXCIV".into())
        );
        assert_eq!(
            arabic(&mut c, &[Value::Text("MCMXCIV".into())]),
            Value::Number(1994.0)
        );
    }

    #[test]
    fn rand_in_range() {
        let mut c = TestCtx::new();
        for _ in 0..100 {
            if let Value::Number(x) = Value::Number(next_rand()) {
                assert!((0.0..1.0).contains(&x));
            }
            let v = randbetween(&mut c, &[Value::Number(1.0), Value::Number(6.0)]);
            if let Value::Number(x) = v {
                assert!((1.0..=6.0).contains(&x));
            }
        }
    }

    // --- MULTINOMIAL ---

    #[test]
    fn multinomial_basic() {
        let mut c = TestCtx::new();
        // MULTINOMIAL(2,3,4) = 9! / (2! * 3! * 4!) = 362880 / (2*6*24) = 362880/288 = 1260
        assert_eq!(
            multinomial(
                &mut c,
                &[Value::Number(2.0), Value::Number(3.0), Value::Number(4.0)]
            ),
            Value::Number(1260.0)
        );
        // MULTINOMIAL(1) = 1
        assert_eq!(
            multinomial(&mut c, &[Value::Number(1.0)]),
            Value::Number(1.0)
        );
        // Negative arg → #NUM!
        assert_eq!(
            multinomial(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }

    // --- SUMX2MY2 / SUMX2PY2 / SUMXMY2 ---

    #[test]
    fn paired_array_fns() {
        use crate::core::formula::value::Array;
        let mut c = TestCtx::new();

        let xs = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
        ));
        let ys = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(0.0), Value::Number(0.0), Value::Number(0.0)],
        ));

        // SUMX2MY2({1,2,3},{0,0,0}) = 1+4+9 = 14
        assert_eq!(
            sumx2my2(&mut c, &[xs.clone(), ys.clone()]),
            Value::Number(14.0)
        );

        // SUMX2PY2({1,2,3},{0,0,0}) = 1+4+9 = 14
        assert_eq!(
            sumx2py2(&mut c, &[xs.clone(), ys.clone()]),
            Value::Number(14.0)
        );

        // SUMXMY2({1,2},{0,0}) = 1+4 = 5
        let xs2 = Value::Array(Array::new(
            1,
            2,
            vec![Value::Number(1.0), Value::Number(2.0)],
        ));
        let ys2 = Value::Array(Array::new(
            1,
            2,
            vec![Value::Number(0.0), Value::Number(0.0)],
        ));
        assert_eq!(
            sumxmy2(&mut c, &[xs2.clone(), ys2.clone()]),
            Value::Number(5.0)
        );

        // Mismatched sizes → #N/A
        let ys3 = Value::Array(Array::new(1, 1, vec![Value::Number(0.0)]));
        assert_eq!(
            sumx2my2(&mut c, &[xs.clone(), ys3]),
            Value::Error(CellError::NA)
        );
    }

    // --- SERIESSUM ---

    #[test]
    fn seriessum_basic() {
        use crate::core::formula::value::Array;
        let mut c = TestCtx::new();

        // SERIESSUM(2, 0, 1, {1,1,1}) = 1*2^0 + 1*2^1 + 1*2^2 = 1+2+4 = 7
        let coeffs = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)],
        ));
        assert_eq!(
            seriessum(
                &mut c,
                &[
                    Value::Number(2.0),
                    Value::Number(0.0),
                    Value::Number(1.0),
                    coeffs
                ]
            ),
            Value::Number(7.0)
        );

        // SERIESSUM(1, 0, 1, {1,1,1}) = 1+1+1 = 3
        let coeffs2 = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)],
        ));
        assert_eq!(
            seriessum(
                &mut c,
                &[
                    Value::Number(1.0),
                    Value::Number(0.0),
                    Value::Number(1.0),
                    coeffs2
                ]
            ),
            Value::Number(3.0)
        );

        // SERIESSUM(3, 1, 2, {1,2}) = 1*3^1 + 2*3^3 = 3 + 54 = 57
        let coeffs3 = Value::Array(Array::new(
            1,
            2,
            vec![Value::Number(1.0), Value::Number(2.0)],
        ));
        assert_eq!(
            seriessum(
                &mut c,
                &[
                    Value::Number(3.0),
                    Value::Number(1.0),
                    Value::Number(2.0),
                    coeffs3
                ]
            ),
            Value::Number(57.0)
        );
    }

    // --- SUBTOTAL ---

    #[test]
    fn subtotal_basic() {
        let mut c = TestCtx::with_cells(&[
            (0, 0, Value::Number(1.0)),
            (1, 0, Value::Number(2.0)),
            (2, 0, Value::Number(3.0)),
            (3, 0, Value::Number(4.0)),
            (4, 0, Value::Number(5.0)),
        ]);
        let rng_arg = rng(0, 0, 4, 0);

        // fn_num=9 → SUM
        assert_eq!(
            subtotal(&mut c, &[Value::Number(9.0), rng_arg.clone()]),
            Value::Number(15.0)
        );

        // fn_num=1 → AVERAGE
        assert_eq!(
            subtotal(&mut c, &[Value::Number(1.0), rng_arg.clone()]),
            Value::Number(3.0)
        );

        // fn_num=4 → MAX
        assert_eq!(
            subtotal(&mut c, &[Value::Number(4.0), rng_arg.clone()]),
            Value::Number(5.0)
        );

        // fn_num=5 → MIN
        assert_eq!(
            subtotal(&mut c, &[Value::Number(5.0), rng_arg.clone()]),
            Value::Number(1.0)
        );

        // fn_num=109 → treated same as 9 (SUM)
        assert_eq!(
            subtotal(&mut c, &[Value::Number(109.0), rng_arg.clone()]),
            Value::Number(15.0)
        );
    }

    // --- AGGREGATE ---

    #[test]
    fn aggregate_basic() {
        use crate::core::formula::value::Array;
        let mut c = TestCtx::new();

        let data = Value::Array(Array::new(
            1,
            5,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Number(5.0),
            ],
        ));

        // fn_num=9 (SUM), options=0
        assert_eq!(
            aggregate(
                &mut c,
                &[Value::Number(9.0), Value::Number(0.0), data.clone()]
            ),
            Value::Number(15.0)
        );

        // fn_num=12 (MEDIAN), options=0
        assert_eq!(
            aggregate(
                &mut c,
                &[Value::Number(12.0), Value::Number(0.0), data.clone()]
            ),
            Value::Number(3.0)
        );

        // fn_num=4 (MAX), options=6 (ignore errors)
        let data_with_err = Value::Array(Array::new(
            1,
            4,
            vec![
                Value::Number(1.0),
                Value::Error(CellError::Div0),
                Value::Number(5.0),
                Value::Number(3.0),
            ],
        ));
        assert_eq!(
            aggregate(
                &mut c,
                &[
                    Value::Number(4.0),
                    Value::Number(6.0),
                    data_with_err.clone()
                ]
            ),
            Value::Number(5.0)
        );

        // fn_num=14 (LARGE), options=0, k=2
        assert_eq!(
            aggregate(
                &mut c,
                &[
                    Value::Number(14.0),
                    Value::Number(0.0),
                    data.clone(),
                    Value::Number(2.0)
                ]
            ),
            Value::Number(4.0)
        );
    }

    // --- Matrix functions ---

    #[test]
    fn mdeterm_basic() {
        use crate::core::formula::value::Array;
        let mut c = TestCtx::new();

        // [[1,2],[3,4]] → det = 1*4 - 2*3 = -2
        let mat = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
            ],
        ));
        assert_eq!(mdeterm(&mut c, &[mat]), Value::Number(-2.0));

        // [[5]] → det = 5
        let mat2 = Value::Array(Array::new(1, 1, vec![Value::Number(5.0)]));
        assert_eq!(mdeterm(&mut c, &[mat2]), Value::Number(5.0));

        // Non-square → #VALUE!
        let mat3 = Value::Array(Array::new(
            1,
            3,
            vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)],
        ));
        assert_eq!(mdeterm(&mut c, &[mat3]), Value::Error(CellError::Value));
    }

    #[test]
    fn minverse_basic() {
        use crate::core::formula::value::Array;
        let mut c = TestCtx::new();

        // [[1,0],[0,1]] inverse = [[1,0],[0,1]]
        let identity = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(1.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        ));
        let result = minverse(&mut c, &[identity]);
        if let Value::Array(a) = result {
            let data: Vec<f64> = a
                .data
                .iter()
                .map(|v| {
                    if let Value::Number(n) = v {
                        *n
                    } else {
                        f64::NAN
                    }
                })
                .collect();
            assert!((data[0] - 1.0).abs() < 1e-10);
            assert!((data[1] - 0.0).abs() < 1e-10);
            assert!((data[2] - 0.0).abs() < 1e-10);
            assert!((data[3] - 1.0).abs() < 1e-10);
        } else {
            panic!("expected Array result");
        }
    }

    #[test]
    fn mmult_basic() {
        use crate::core::formula::value::Array;
        let mut c = TestCtx::new();

        // [[1,0],[0,1]] × [[3,4],[5,6]] = [[3,4],[5,6]]
        let identity = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(1.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        ));
        let other = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Number(5.0),
                Value::Number(6.0),
            ],
        ));
        let result = mmult(&mut c, &[identity, other]);
        if let Value::Array(a) = result {
            let data: Vec<f64> = a
                .data
                .iter()
                .map(|v| {
                    if let Value::Number(n) = v {
                        *n
                    } else {
                        f64::NAN
                    }
                })
                .collect();
            assert_eq!(data, vec![3.0, 4.0, 5.0, 6.0]);
        } else {
            panic!("expected Array result");
        }

        // Mismatched inner dims → #VALUE!
        let a2 = Value::Array(Array::new(
            2,
            3,
            vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
                Value::Number(4.0),
                Value::Number(5.0),
                Value::Number(6.0),
            ],
        ));
        let b2 = Value::Array(Array::new(
            2,
            2,
            vec![
                Value::Number(1.0),
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Number(1.0),
            ],
        ));
        assert_eq!(mmult(&mut c, &[a2, b2]), Value::Error(CellError::Value));
    }

    #[test]
    fn munit_basic() {
        let mut c = TestCtx::new();

        let result = munit(&mut c, &[Value::Number(3.0)]);
        if let Value::Array(a) = result {
            assert_eq!(a.rows, 3);
            assert_eq!(a.cols, 3);
            // diagonal should be 1, off-diagonal 0
            for r in 0..3 {
                for c_idx in 0..3 {
                    let expected = if r == c_idx { 1.0 } else { 0.0 };
                    assert_eq!(a.get(r, c_idx), Some(&Value::Number(expected)));
                }
            }
        } else {
            panic!("expected Array result");
        }
    }

    // --- GAMMA ---

    #[test]
    fn gamma_basic() {
        let mut c = TestCtx::new();

        // Γ(1) = 1
        if let Value::Number(g) = gamma(&mut c, &[Value::Number(1.0)]) {
            assert!((g - 1.0).abs() < 1e-9, "Γ(1) ≈ 1, got {g}");
        } else {
            panic!("expected Number for Γ(1)");
        }

        // Γ(5) = 4! = 24
        if let Value::Number(g) = gamma(&mut c, &[Value::Number(5.0)]) {
            assert!((g - 24.0).abs() < 1e-6, "Γ(5) ≈ 24, got {g}");
        } else {
            panic!("expected Number");
        }

        // Γ(0.5) = √π ≈ 1.7724538509
        if let Value::Number(g) = gamma(&mut c, &[Value::Number(0.5)]) {
            assert!(
                (g - std::f64::consts::PI.sqrt()).abs() < 1e-6,
                "Γ(0.5) ≈ √π, got {g}"
            );
        } else {
            panic!("expected Number");
        }

        // Non-positive integer → #NUM!
        assert_eq!(
            gamma(&mut c, &[Value::Number(0.0)]),
            Value::Error(CellError::Num)
        );
        assert_eq!(
            gamma(&mut c, &[Value::Number(-1.0)]),
            Value::Error(CellError::Num)
        );
    }
}
