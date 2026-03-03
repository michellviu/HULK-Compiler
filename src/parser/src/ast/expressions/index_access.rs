use crate::ast::{Expression, Visitable, Visitor};
use crate::tokens::Span;

/// `expr[index]`
#[derive(Debug, Clone)]
pub struct IndexAccess {
    pub object: Expression,
    pub index: Expression,
    pub span: Span,
}

impl Visitable for IndexAccess {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_index_access(self);
    }
}

