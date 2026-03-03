use crate::ast::Expression;
use crate::ast::Visitable;
use crate::ast::Visitor;
use crate::tokens;
use crate::tokens::Span;

#[derive(Debug,Clone)]
pub struct UnaryOp {
    pub op: tokens::UnaryOp,
    pub expr: Box<Expression>,
    pub span: Span,
}

impl UnaryOp {
    pub fn new(op: tokens::UnaryOp, expr: Expression, span: Span) -> Self {
        UnaryOp { op, expr: Box::new(expr), span }
    }
}

impl Visitable for UnaryOp {

    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_unary_op(self);
    }
}
