/// 对应 Java：无直接对应对象；Rust 架构扩展。 A `LAMBDA(param1, …, body)` closure: parameter names and the body expression.
#[derive(Debug, PartialEq)]
pub struct Lambda {
    pub params: Vec<String>,
    pub body: Expr,
}

