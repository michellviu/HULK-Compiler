use crate::ast::{Expression, Param, Attribute, Method, Visitable, Visitor};
use crate::tokens::Span;

/// `class Name[(params)] [inherits Parent[(args)]] { attrs methods }`
#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub parent: Option<String>,
    pub parent_args: Vec<Expression>,
    pub attributes: Vec<Attribute>,
    pub methods: Vec<Method>,
    pub span: Span,
}

impl Visitable for ClassDecl {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_class_decl(self);
    }
}

