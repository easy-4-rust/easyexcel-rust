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
        .map_or(0x9E37_79B9, |d| d.as_nanos() as u64);
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
        Value::Number(n) => easyexcel_model::value::format_number_general(*n),
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
        let next = chars.get(i + 1).map_or(0, |c| val(*c));
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

/// Perform a subtotal aggregation (`function_num` 1-11; 101-111 treated same).
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
                .map_or(Value::Number(0.0), Value::Number)
        }
        5 => {
            // MIN
            nums.iter()
                .copied()
                .reduce(f64::min)
                .map_or(Value::Number(0.0), Value::Number)
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

