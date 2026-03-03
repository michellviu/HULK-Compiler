use super::super::Visitable;
use super::super::Visitor;
use super::*;
use crate::ast::Atom;
use crate::tokens;
use crate::tokens::Span;

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
    pub fn new_binary_op(left: Expression, right: Expression, operator: tokens::BinOp, span: Span) -> Self {
        Expression::BinaryOp(BinaryOp::new(left, right, operator, span))
    }

    pub fn new_unary_op(op: tokens::UnaryOp, expr: Expression, span: Span) -> Self {
        Expression::UnaryOp(UnaryOp::new(op, expr, span))
    }

    pub fn new_atom(atom: Atom) -> Self {
        Expression::Atom(Box::new(atom))
    }

    /// Returns the span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Expression::BinaryOp(e) => e.span,
            Expression::UnaryOp(e) => e.span,
            Expression::Atom(a) => a.span(),
            Expression::Let(e) => e.span,
            Expression::If(e) => e.span,
            Expression::While(e) => e.span,
            Expression::Case(e) => e.span,
            Expression::Assign(e) => e.span,
            Expression::MemberAccess(e) => e.span,
            Expression::MethodCall(e) => e.span,
            Expression::IndexAccess(e) => e.span,
            Expression::FunctionCall(e) => e.span,
            Expression::NewInstance(e) => e.span,
            Expression::NewArray(e) => e.span,
        }
    }
}

impl Visitable for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_expression(self);
    }
}
