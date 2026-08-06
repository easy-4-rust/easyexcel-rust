fn convert(_: &mut dyn Context, args: &[Value]) -> Value {
    let val = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let from = match get_text(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let to = match get_text(args, 2) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };

    // Temperature
    if is_temp(&from) || is_temp(&to) {
        if !is_temp(&from) || !is_temp(&to) {
            return Value::Error(CellError::NA);
        }
        let k = match temp_to_kelvin(val, &from) {
            Some(v) => v,
            None => return Value::Error(CellError::NA),
        };
        return match kelvin_to_temp(k, &to) {
            Some(v) => Value::Number(v),
            None => Value::Error(CellError::NA),
        };
    }

    let from_factor = match resolve_unit(&from) {
        Some(f) => f,
        None => return Value::Error(CellError::NA),
    };
    let to_factor = match resolve_unit(&to) {
        Some(f) => f,
        None => return Value::Error(CellError::NA),
    };

    // Both must be in same dimension (same base unit category):
    // We detect this by checking if from/to can both map to SI and are compatible.
    // Simple heuristic: if factor ratio is completely unreasonable, return #N/A.
    // For correctness, we rely on the user providing same-category units.
    let result = val * from_factor / to_factor;
    if result.is_finite() {
        Value::Number(result)
    } else {
        Value::Error(CellError::NA)
    }
}

// ---------------------------------------------------------------------------
// ERF / ERFC
// Approximation using the complementary error function series.
// PARITY: uses Horner's method rational approximation; max relative error ~1.5e-7.
// ---------------------------------------------------------------------------

/// Error function using rational approximation (Abramowitz & Stegun 7.1.26).
/// Max absolute error ~1.5e-7.
#[must_use]
pub fn erf_approx(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    if x.abs() > 6.0 {
        return x.signum();
    }
    if x < 0.0 {
        return -erf_approx(-x);
    }
    // A&S 7.1.26: erfc(x) ≈ poly(t) * exp(-x^2), t = 1/(1+0.3275911*x)
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    1.0 - poly * (-x * x).exp()
}

#[must_use]
pub fn erfc_approx(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    1.0 - erf_approx(x)
}

fn erf_fn(_: &mut dyn Context, args: &[Value]) -> Value {
    let lower = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    if args.len() == 2 {
        let upper = match get(args, 1) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        Value::Number(erf_approx(upper) - erf_approx(lower))
    } else {
        Value::Number(erf_approx(lower))
    }
}

fn erfc_fn(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    Value::Number(erfc_approx(x))
}

fn erf_precise(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    Value::Number(erf_approx(x))
}

fn erfc_precise(_: &mut dyn Context, args: &[Value]) -> Value {
    let x = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    Value::Number(erfc_approx(x))
}

// ---------------------------------------------------------------------------
// Complex number support
// Excel represents complex numbers as text: "3+4i", "3-4i", "3", "4i", etc.
// suffix is 'i' by default, 'j' if specified.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    fn to_text(self, suffix: char) -> String {
        let re = self.re;
        let im = self.im;
        if im == 0.0 {
            return easyexcel_model::value::format_number_general(re);
        }
        if re == 0.0 {
            if im == 1.0 {
                return format!("{suffix}");
            } else if im == -1.0 {
                return format!("-{suffix}");
            }
            return format!(
                "{}{suffix}",
                easyexcel_model::value::format_number_general(im)
            );
        }
        let re_str = easyexcel_model::value::format_number_general(re);
        if im == 1.0 {
            format!("{re_str}+{suffix}")
        } else if im == -1.0 {
            format!("{re_str}-{suffix}")
        } else if im < 0.0 {
            format!(
                "{re_str}{}{suffix}",
                easyexcel_model::value::format_number_general(im)
            )
        } else {
            format!(
                "{re_str}+{}{suffix}",
                easyexcel_model::value::format_number_general(im)
            )
        }
    }

    fn abs(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
    fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    fn add(&self, other: &Complex) -> Complex {
        Complex::new(self.re + other.re, self.im + other.im)
    }
    fn sub(&self, other: &Complex) -> Complex {
        Complex::new(self.re - other.re, self.im - other.im)
    }
    fn mul(&self, other: &Complex) -> Complex {
        Complex::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
    fn div(&self, other: &Complex) -> Option<Complex> {
        let denom = other.re * other.re + other.im * other.im;
        if denom == 0.0 {
            return None;
        }
        Some(Complex::new(
            (self.re * other.re + self.im * other.im) / denom,
            (self.im * other.re - self.re * other.im) / denom,
        ))
    }
    fn exp(&self) -> Complex {
        let e_re = self.re.exp();
        Complex::new(e_re * self.im.cos(), e_re * self.im.sin())
    }
    fn ln(&self) -> Option<Complex> {
        let r = self.abs();
        if r == 0.0 {
            return None;
        }
        Some(Complex::new(r.ln(), self.arg()))
    }
    fn sqrt(&self) -> Complex {
        let r = self.abs().sqrt();
        let theta = self.arg() / 2.0;
        Complex::new(r * theta.cos(), r * theta.sin())
    }
    fn pow_complex(&self, n: &Complex) -> Option<Complex> {
        let ln = self.ln()?;
        Some((ln.mul(n)).exp())
    }
    fn conjugate(&self) -> Complex {
        Complex::new(self.re, -self.im)
    }
}

/// Parse a complex number from Excel text format.
fn parse_complex(s: &str) -> Result<(Complex, char), CellError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(CellError::Value);
    }

    // Determine suffix
    let suffix = if s.ends_with('j') { 'j' } else { 'i' };

    // If no imaginary part (no 'i' or 'j' suffix)
    if !s.ends_with('i') && !s.ends_with('j') {
        let re = s.parse::<f64>().map_err(|_| CellError::Value)?;
        return Ok((Complex::new(re, 0.0), 'i'));
    }

    // Strip the suffix
    let s = &s[..s.len() - 1];

    // Pure imaginary: "i", "-i", "+i"
    if s.is_empty() || s == "+" {
        return Ok((Complex::new(0.0, 1.0), suffix));
    }
    if s == "-" {
        return Ok((Complex::new(0.0, -1.0), suffix));
    }

    // Find the split point between real and imaginary parts.
    // Look for a + or - that's not at the start and not part of an exponent.
    let bytes = s.as_bytes();
    let mut split = None;
    let mut i = 1usize; // skip potential leading sign
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '+' || ch == '-' {
            // Make sure it's not an exponent sign
            let prev = bytes[i - 1] as char;
            if prev != 'e' && prev != 'E' {
                split = Some(i);
                break;
            }
        }
        i += 1;
    }

    if let Some(pos) = split {
        let re_str = &s[..pos];
        let im_str = &s[pos..];
        let re = if re_str.is_empty() {
            0.0
        } else {
            re_str.parse::<f64>().map_err(|_| CellError::Value)?
        };
        let im = if im_str.is_empty() || im_str == "+" {
            1.0
        } else if im_str == "-" {
            -1.0
        } else {
            im_str.parse::<f64>().map_err(|_| CellError::Value)?
        };
        Ok((Complex::new(re, im), suffix))
    } else {
        // Pure imaginary number (no real part)
        let im = if s.is_empty() || s == "+" {
            1.0
        } else if s == "-" {
            -1.0
        } else {
            s.parse::<f64>().map_err(|_| CellError::Value)?
        };
        Ok((Complex::new(0.0, im), suffix))
    }
}

fn parse_complex_arg(args: &[Value], i: usize) -> Result<(Complex, char), CellError> {
    match &args[i] {
        Value::Text(s) => parse_complex(s),
        Value::Number(n) => Ok((Complex::new(*n, 0.0), 'i')),
        Value::Error(e) => Err(*e),
        _ => Err(CellError::Value),
    }
}

fn complex_val(c: Complex, suffix: char) -> Value {
    Value::Text(c.to_text(suffix))
}

// COMPLEX: COMPLEX(real, imaginary [, suffix])
fn complex(_: &mut dyn Context, args: &[Value]) -> Value {
    let re = match get(args, 0) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let im = match get(args, 1) {
        Ok(v) => v,
        Err(e) => return Value::Error(e),
    };
    let suffix = if args.len() >= 3 {
        match get_text(args, 2) {
            Ok(s) => {
                if s == "j" {
                    'j'
                } else if s == "i" || s.is_empty() {
                    'i'
                } else {
                    return Value::Error(CellError::Value);
                }
            }
            Err(e) => return Value::Error(e),
        }
    } else {
        'i'
    };
    complex_val(Complex::new(re, im), suffix)
}

fn imabs(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, _)) => Value::Number(c.abs()),
        Err(e) => Value::Error(e),
    }
}

fn imreal(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, _)) => Value::Number(c.re),
        Err(e) => Value::Error(e),
    }
}

fn imaginary(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, _)) => Value::Number(c.im),
        Err(e) => Value::Error(e),
    }
}

fn imconjugate(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => complex_val(c.conjugate(), s),
        Err(e) => Value::Error(e),
    }
}

fn imsum(_: &mut dyn Context, args: &[Value]) -> Value {
    let mut result = Complex::new(0.0, 0.0);
    let mut suffix = 'i';
    for arg in args {
        match parse_complex_arg(std::slice::from_ref(arg), 0) {
            Ok((c, s)) => {
                result = result.add(&c);
                suffix = s;
            }
            Err(e) => return Value::Error(e),
        }
    }
    complex_val(result, suffix)
}

fn imsub(_: &mut dyn Context, args: &[Value]) -> Value {
    match (parse_complex_arg(args, 0), parse_complex_arg(args, 1)) {
        (Ok((a, s)), Ok((b, _))) => complex_val(a.sub(&b), s),
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn improduct(_: &mut dyn Context, args: &[Value]) -> Value {
    let mut result = Complex::new(1.0, 0.0);
    let mut suffix = 'i';
    for arg in args {
        match parse_complex_arg(std::slice::from_ref(arg), 0) {
            Ok((c, s)) => {
                result = result.mul(&c);
                suffix = s;
            }
            Err(e) => return Value::Error(e),
        }
    }
    complex_val(result, suffix)
}

fn imdiv(_: &mut dyn Context, args: &[Value]) -> Value {
    match (parse_complex_arg(args, 0), parse_complex_arg(args, 1)) {
        (Ok((a, s)), Ok((b, _))) => match a.div(&b) {
            Some(c) => complex_val(c, s),
            None => Value::Error(CellError::Div0),
        },
        (Err(e), _) | (_, Err(e)) => Value::Error(e),
    }
}

fn imexp(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => complex_val(c.exp(), s),
        Err(e) => Value::Error(e),
    }
}

fn imln(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => match c.ln() {
            Some(r) => complex_val(r, s),
            None => Value::Error(CellError::Num),
        },
        Err(e) => Value::Error(e),
    }
}

fn imsqrt(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => complex_val(c.sqrt(), s),
        Err(e) => Value::Error(e),
    }
}

fn imargument(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, _)) => {
            if c.re == 0.0 && c.im == 0.0 {
                return Value::Error(CellError::Div0);
            }
            Value::Number(c.arg())
        }
        Err(e) => Value::Error(e),
    }
}

fn impower(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // Power can be a number or complex
            let p = if let Ok((cp, _)) = parse_complex_arg(args, 1) {
                cp
            } else {
                match get(args, 1) {
                    Ok(n) => Complex::new(n, 0.0),
                    Err(e) => return Value::Error(e),
                }
            };
            match c.pow_complex(&p) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Num),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imsin(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // sin(a+bi) = sin(a)cosh(b) + i*cos(a)sinh(b)
            let r = Complex::new(c.re.sin() * c.im.cosh(), c.re.cos() * c.im.sinh());
            complex_val(r, s)
        }
        Err(e) => Value::Error(e),
    }
}

fn imcos(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // cos(a+bi) = cos(a)cosh(b) - i*sin(a)sinh(b)
            let r = Complex::new(c.re.cos() * c.im.cosh(), -c.re.sin() * c.im.sinh());
            complex_val(r, s)
        }
        Err(e) => Value::Error(e),
    }
}

fn imtan(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // tan(a+bi) = sin(a+bi)/cos(a+bi)
            let sin_c = Complex::new(c.re.sin() * c.im.cosh(), c.re.cos() * c.im.sinh());
            let cos_c = Complex::new(c.re.cos() * c.im.cosh(), -c.re.sin() * c.im.sinh());
            match sin_c.div(&cos_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imcot(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // cot(a+bi) = cos(a+bi)/sin(a+bi)
            let sin_c = Complex::new(c.re.sin() * c.im.cosh(), c.re.cos() * c.im.sinh());
            let cos_c = Complex::new(c.re.cos() * c.im.cosh(), -c.re.sin() * c.im.sinh());
            match cos_c.div(&sin_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imsinh(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // sinh(a+bi) = sinh(a)cos(b) + i*cosh(a)sin(b)
            let r = Complex::new(c.re.sinh() * c.im.cos(), c.re.cosh() * c.im.sin());
            complex_val(r, s)
        }
        Err(e) => Value::Error(e),
    }
}

fn imcosh(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            // cosh(a+bi) = cosh(a)cos(b) + i*sinh(a)sin(b)
            let r = Complex::new(c.re.cosh() * c.im.cos(), c.re.sinh() * c.im.sin());
            complex_val(r, s)
        }
        Err(e) => Value::Error(e),
    }
}

fn imsec(_: &mut dyn Context, args: &[Value]) -> Value {
    // sec = 1/cos
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            let cos_c = Complex::new(c.re.cos() * c.im.cosh(), -c.re.sin() * c.im.sinh());
            match Complex::new(1.0, 0.0).div(&cos_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imcsc(_: &mut dyn Context, args: &[Value]) -> Value {
    // csc = 1/sin
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            let sin_c = Complex::new(c.re.sin() * c.im.cosh(), c.re.cos() * c.im.sinh());
            match Complex::new(1.0, 0.0).div(&sin_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imsech(_: &mut dyn Context, args: &[Value]) -> Value {
    // sech = 1/cosh
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            let cosh_c = Complex::new(c.re.cosh() * c.im.cos(), c.re.sinh() * c.im.sin());
            match Complex::new(1.0, 0.0).div(&cosh_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imcsch(_: &mut dyn Context, args: &[Value]) -> Value {
    // csch = 1/sinh
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => {
            let sinh_c = Complex::new(c.re.sinh() * c.im.cos(), c.re.cosh() * c.im.sin());
            match Complex::new(1.0, 0.0).div(&sinh_c) {
                Some(r) => complex_val(r, s),
                None => Value::Error(CellError::Div0),
            }
        }
        Err(e) => Value::Error(e),
    }
}

fn imlog2(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => match c.ln() {
            Some(ln_c) => {
                let ln2 = Complex::new(std::f64::consts::LN_2, 0.0);
                match ln_c.div(&ln2) {
                    Some(r) => complex_val(r, s),
                    None => Value::Error(CellError::Num),
                }
            }
            None => Value::Error(CellError::Num),
        },
        Err(e) => Value::Error(e),
    }
}

fn imlog10(_: &mut dyn Context, args: &[Value]) -> Value {
    match parse_complex_arg(args, 0) {
        Ok((c, s)) => match c.ln() {
            Some(ln_c) => {
                let ln10 = Complex::new(std::f64::consts::LN_10, 0.0);
                match ln_c.div(&ln10) {
                    Some(r) => complex_val(r, s),
                    None => Value::Error(CellError::Num),
                }
            }
            None => Value::Error(CellError::Num),
        },
        Err(e) => Value::Error(e),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "../engineering_tests/tests.rs"]
mod tests;
