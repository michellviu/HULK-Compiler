use crate::ast::{Expression, Visitable, Visitor};
use crate::tokens::Span;

/// `expr is TypeName`
#[derive(Debug, Clone)]
pub struct IsExpr {
    pub expr: Expression,
    pub type_name: String,
    pub span: Span,
}

impl Visitable for IsExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_is_expr(self);
    }
}
