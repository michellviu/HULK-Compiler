use crate::ast::Expression;

/// Represents an expression body that can be either a single expression
/// or a block of expressions: `{ expr1; expr2; ... }`
#[derive(Debug, Clone)]
pub enum ExprBody {
    Single(Box<Expression>),
    Block(Vec<Expression>),
}

