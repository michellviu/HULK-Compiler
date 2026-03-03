use crate::ast::{Expression, Visitable, Visitor};

/// `expr[index]`
#[derive(Debug, Clone)]
pub struct IndexAccess {
    pub object: Expression,
    pub index: Expression,
}

impl Visitable for IndexAccess {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_index_access(self);
    }
}

