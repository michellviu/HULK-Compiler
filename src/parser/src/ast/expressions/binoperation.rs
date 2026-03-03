use super::Expression;
use crate::tokens::BinOp;
use crate::tokens::Span;
use super::super::Visitor;
use super::super::Visitable;
#[derive(Debug,Clone)]
pub struct BinaryOp{
   pub left: Box<Expression>,
   pub right: Box<Expression>,
   pub operator: BinOp,
   pub span: Span,
}

impl BinaryOp {
    pub fn new(left: Expression, right: Expression, operator:BinOp, span: Span) -> Self {
        BinaryOp {
            left: Box::new(left),
            right: Box::new(right),
            operator,
            span,
        }
    }
}

impl Visitable for BinaryOp {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_binary_op(self);
    }
    
}