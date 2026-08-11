//! Logical functions. IF/AND/OR/IFS/SWITCH/IFERROR/IFNA are lazy "special forms"
//! handled directly in the evaluator; the remaining (non-lazy) logical functions
//! live here.

use super::Registry;
use crate::formula::coerce::to_bool;
use crate::formula::context::Context;
use crate::formula::value::Value;
use easyexcel_model::error::CellError;
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub fn register(r: &mut Registry) {
    r.add("NOT", 1, 1, false, not);
    r.add("TRUE", 0, 0, false, |_, _| Value::Bool(true));
    r.add("FALSE", 0, 0, false, |_, _| Value::Bool(false));
    r.add("XOR", 1, super::VARIADIC, false, xor);
}

fn not(_: &mut dyn Context, args: &[Value]) -> Value {
    match to_bool(&args[0]) {
        Ok(b) => Value::Bool(!b),
        Err(e) => Value::Error(e),
    }
}

fn xor(ctx: &mut dyn Context, args: &[Value]) -> Value {
    let mut count = 0u64;
    let mut seen = false;
    for arg in args {
        for v in ctx.flatten(arg) {
            match v {
                Value::Empty | Value::Text(_) => continue,
                Value::Error(e) => return Value::Error(e),
                other => match to_bool(&other) {
                    Ok(b) => {
                        seen = true;
                        if b {
                            count += 1;
                        }
                    }
                    Err(e) => return Value::Error(e),
                },
            }
        }
    }
    if !seen {
        return Value::Error(CellError::Value);
    }
    Value::Bool(count % 2 == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::functions::testutil::TestCtx;

    #[test]
    fn not_fn() {
        let mut c = TestCtx::new();
        assert_eq!(not(&mut c, &[Value::Bool(true)]), Value::Bool(false));
        assert_eq!(not(&mut c, &[Value::Number(0.0)]), Value::Bool(true));
        assert_eq!(
            not(&mut c, &[Value::Text("x".into())]),
            Value::Error(CellError::Value)
        );
    }

    #[test]
    fn xor_fn() {
        let mut c = TestCtx::new();
        assert_eq!(
            xor(&mut c, &[Value::Bool(true), Value::Bool(false)]),
            Value::Bool(true)
        );
        assert_eq!(
            xor(&mut c, &[Value::Bool(true), Value::Bool(true)]),
            Value::Bool(false)
        );
        assert_eq!(
            xor(
                &mut c,
                &[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]
            ),
            Value::Bool(true)
        );
    }

    // ── 补充逻辑函数测试 ────────────────────────────────────────────────

    #[test]
    fn not_fn_error() {
        let mut c = TestCtx::new();
        assert_eq!(
            not(&mut c, &[Value::Text("x".into())]),
            Value::Error(CellError::Value)
        );
    }

    #[test]
    fn xor_fn_no_args() {
        let mut c = TestCtx::new();
        // 空参数 → #VALUE!
        let result = xor(&mut c, &[]);
        assert_eq!(result, Value::Error(CellError::Value));
    }

    #[test]
    fn xor_fn_text_ignored() {
        let mut c = TestCtx::new();
        // 文本在范围中被忽略
        let result = xor(&mut c, &[Value::Text("x".into()), Value::Bool(true)]);
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn xor_fn_error_propagation() {
        let mut c = TestCtx::new();
        assert_eq!(
            xor(&mut c, &[Value::Error(CellError::NA)]),
            Value::Error(CellError::NA)
        );
    }

    #[test]
    fn xor_fn_all_false() {
        let mut c = TestCtx::new();
        assert_eq!(
            xor(&mut c, &[Value::Bool(false), Value::Bool(false)]),
            Value::Bool(false)
        );
    }
}
