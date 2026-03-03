use super::super::Visitable;
use super::super::Visitor;
use super::*;
use crate::ast::Atom;
use crate::tokens;

#[derive(Debug, Clone)]
pub enum Expression {
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    Atom(Box<Atom>),
    Let(Box<LetExpr>),
    If(Box<IfExpr>),
    While(Box<WhileExpr>),
    Case(Box<CaseExpr>),
    Assign(Box<AssignExpr>),
    MemberAccess(Box<MemberAccess>),
    MethodCall(Box<MethodCall>),
    IndexAccess(Box<IndexAccess>),
    FunctionCall(Box<FunctionCall>),
    NewInstance(Box<NewInstance>),
    NewArray(Box<NewArray>),
}

impl Expression {
    pub fn new_binary_op(left: Expression, right: Expression, operator: tokens::BinOp) -> Self {
        Expression::BinaryOp(BinaryOp::new(left, right, operator))
    }

    pub fn new_unary_op(op: tokens::UnaryOp, expr: Expression) -> Self {
        Expression::UnaryOp(UnaryOp::new(op, expr))
    }

    pub fn new_atom(atom: Atom) -> Self {
        Expression::Atom(Box::new(atom))
    }
}

impl Visitable for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_expression(self);
    }
}
