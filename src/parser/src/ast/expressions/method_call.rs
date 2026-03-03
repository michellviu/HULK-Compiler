use crate::ast::{Expression, Visitable, Visitor};
use crate::tokens::Span;

/// `expr.method(args)`
#[derive(Debug, Clone)]
pub struct MethodCall {
    pub object: Expression,
    pub method: String,
    pub args: Vec<Expression>,
    pub span: Span,
}

impl Visitable for MethodCall {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_method_call(self);
    }
}

