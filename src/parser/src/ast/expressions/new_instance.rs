use crate::ast::{Expression, Visitable, Visitor};
use crate::tokens::Span;

/// `new TypeName(args)`
#[derive(Debug, Clone)]
pub struct NewInstance {
    pub type_name: String,
    pub args: Vec<Expression>,
    pub span: Span,
}

impl Visitable for NewInstance {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_new_instance(self);
    }
}

