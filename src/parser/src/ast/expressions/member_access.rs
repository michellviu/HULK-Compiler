use crate::ast::{Expression, Visitable, Visitor};
use crate::tokens::Span;

/// `expr.member`
#[derive(Debug, Clone)]
pub struct MemberAccess {
    pub object: Expression,
    pub member: String,
    pub span: Span,
}

impl Visitable for MemberAccess {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_member_access(self);
    }
}

