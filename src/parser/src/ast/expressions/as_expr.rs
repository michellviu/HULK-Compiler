use crate::ast::{Expression, Visitable, Visitor};
use crate::tokens::Span;

/// `expr as TypeName`
#[derive(Debug, Clone)]
pub struct AsExpr {
    pub expr: Expression,
    pub type_name: String,
    pub span: Span,
}

impl Visitable for AsExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_as_expr(self);
    }
}
