use crate::ast::{Expression, Visitable, Visitor};

/// `expr.member`
#[derive(Debug, Clone)]
pub struct MemberAccess {
    pub object: Expression,
    pub member: String,
}

impl Visitable for MemberAccess {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_member_access(self);
    }
}

