use crate::Expression;
use crate::Visitable;
use crate::Visitor;

#[derive(Debug,Clone)]
pub struct UnaryOp {
    pub op: tokens::UnaryOp,
    pub expr: Box<Expression>,
}

impl UnaryOp {
    pub fn new(op: tokens::UnaryOp, expr: Expression) -> Self {
        UnaryOp { op, expr: Box::new(expr) }
    }
}

impl Visitable for UnaryOp {

    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_unary_op(self);
    }
}
