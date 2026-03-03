use crate::ast::Expression;

/// Function/method body:
/// - `-> expr ;`  (inline)
/// - `{ expr; expr; ... }` (block)
#[derive(Debug, Clone)]
pub enum Body {
    Inline(Expression),
    Block(Vec<Expression>),
}

