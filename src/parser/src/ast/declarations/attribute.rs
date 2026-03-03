use crate::ast::{Expression, Visitable, Visitor};
use crate::tokens::Span;

/// Class attribute: `name [: Type] = init_expr`
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub type_ann: Option<String>,
    pub init: Expression,
    pub span: Span,
}

impl Visitable for Attribute {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_attribute(self);
    }
}

