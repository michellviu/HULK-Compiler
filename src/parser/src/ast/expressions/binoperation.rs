use super::Expression;
use crate::tokens::BinOp;
use super::super::Visitor;
use super::super::Visitable;
#[derive(Debug,Clone)]
pub struct BinaryOp{
   pub left: Box<Expression>,
   pub right: Box<Expression>,
   pub operator: BinOp,
}

impl BinaryOp {
    pub fn new(left: Expression, right: Expression, operator:BinOp) -> Self {
        BinaryOp {
            left: Box::new(left),
            right: Box::new(right),
            operator,
        }
    }
}

impl Visitable for BinaryOp {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_binary_op(self);
    }
    
}