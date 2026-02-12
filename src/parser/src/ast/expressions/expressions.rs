use super::super::Visitable;
use super::super::Visitor;
use super::*;
use crate::Atom;
use crate::BinOp;
use crate::tokens;

#[derive(Debug,Clone)]
pub enum Expression {
    BinaryOp(BinaryOp),
    Atom(Box<Atom>),
    UnaryOp(UnaryOp),
}

impl Expression {
   
    pub fn new_binary_op(left: Expression, right: Expression, operator: BinOp) -> Self {
        Expression::BinaryOp(BinaryOp::new(left, right, operator))
    }

    pub fn new_unary_op(op: tokens::UnaryOp, expr: Expression) -> Self
    {
        Expression::UnaryOp(UnaryOp::new(op, expr))
    }

    pub fn new_atom(atom: Atom) -> Self {
        Expression::Atom(Box::new(atom))
    }

}

impl Visitable for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        match self {
            Expression::BinaryOp(binop) => visitor.visit_binary_op(binop),
            Expression::Atom(atom) => atom.accept(visitor),
            Expression::UnaryOp(unoperator) => unoperator.accept(visitor),
        }
    }
}
