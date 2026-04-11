use crate::ast::{ExprBody, Expression, Visitable, Visitor};
use crate::tokens::Span;

/// `for (x in iterable) body`
#[derive(Debug, Clone)]
pub struct ForExpr {
    pub var: String,
    pub iterable: Expression,
    pub body: ExprBody,
    pub span: Span,
}

impl Visitable for ForExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_for_expr(self);
    }
}
