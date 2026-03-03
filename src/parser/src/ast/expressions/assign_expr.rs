use crate::ast::{Expression, Visitable, Visitor};

/// `location := value`
#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub target: Expression,
    pub value: Expression,
}

impl Visitable for AssignExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_assign_expr(self);
    }
}

