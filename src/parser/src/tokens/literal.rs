use std::fmt;
use super::position::Position;
use crate::ast::Visitable;
use crate::ast::Visitor;

#[derive(Debug,Clone)]
pub enum Literal {
    Number(i32, Position),
    Str(String, Position),
    Bool(bool, Position),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Number(n, _) => write!(f, "{}", n),
            Literal::Str(s, _) => write!(f, "{}", s),
            Literal::Bool(b, _) => write!(f, "{}", b),
        }
    }
}

impl Visitable for Literal {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_literal(&self);
    }
    
}