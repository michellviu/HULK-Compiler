use crate::ast::{Expression, ExprBody, Visitable, Visitor};
use crate::tokens::Span;

/// `while (condition) body`
#[derive(Debug, Clone)]
pub struct WhileExpr {
    pub condition: Expression,
    pub body: ExprBody,
    pub span: Span,
}

impl Visitable for WhileExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_while_expr(self);
    }
}

