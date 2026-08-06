//! Java 注解有符号整型参数的结构化解析。

use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Expr, ExprLit, ExprUnary, Lit, LitInt, UnOp, meta::ParseNestedMeta};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 已校验的 Java `int` 值及其 Rust 表达式。
#[derive(Clone)]
pub(crate) struct SignedInteger {
    value: i32,
    expression: Expr,
}

impl SignedInteger {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用指定值和跨度构造可安全生成的一元整数表达式。
    pub(crate) fn new(value: i32, span: Span) -> Self {
        let magnitude = LitInt::new(&i64::from(value).unsigned_abs().to_string(), span);
        let expression = if value < 0 {
            syn::parse_quote_spanned!(span=> -#magnitude)
        } else {
            syn::parse_quote_spanned!(span=> #magnitude)
        };
        Self { value, expression }
    }

    /// 返回解析后的 `i32` 值。
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) const fn value(&self) -> i32 {
        self.value
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回用于代码生成的 Rust 表达式。
    pub(crate) fn tokens(&self) -> proc_macro2::TokenStream {
        let expression = &self.expression;
        quote!(#expression)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回输入表达式的跨度。
    pub(crate) fn span(&self) -> Span {
        match &self.expression {
            Expr::Lit(value) => value.lit.span(),
            Expr::Unary(value) => value.op.span(),
            _ => Span::call_site(),
        }
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析并校验 Java `int`，支持一元负号且不在过程宏中 panic。
pub(crate) fn parse_signed_i32(meta: &ParseNestedMeta<'_>) -> syn::Result<SignedInteger> {
    let expression: Expr = meta.value()?.parse()?;
    let value = expression_value(&expression)?;
    Ok(SignedInteger { value, expression })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析并校验无符号整数字面量。
pub(crate) fn parse_unsigned_integer<T>(meta: &ParseNestedMeta<'_>) -> syn::Result<LitInt>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value: LitInt = meta.value()?.parse()?;
    value
        .base10_parse::<T>()
        .map_err(|error| syn::Error::new_spanned(&value, error))?;
    Ok(value)
}

fn expression_value(expression: &Expr) -> syn::Result<i32> {
    let signed = match expression {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value
            .base10_parse::<i64>()
            .map_err(|error| syn::Error::new_spanned(value, error))?,
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => match expr.as_ref() {
            Expr::Lit(ExprLit {
                lit: Lit::Int(value),
                ..
            }) => value
                .base10_parse::<i64>()
                .map_err(|error| syn::Error::new_spanned(value, error))?
                .checked_neg()
                .ok_or_else(|| syn::Error::new_spanned(value, "integer overflow"))?,
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "annotation value must be an integer",
                ));
            }
        },
        other => {
            return Err(syn::Error::new_spanned(
                other,
                "annotation value must be an integer",
            ));
        }
    };
    i32::try_from(signed).map_err(|error| syn::Error::new_spanned(expression, error))
}
