use crate::ast::{Expression, Param, Attribute, Method, Visitable, Visitor};

/// `class Name[(params)] [is Parent[(args)]] { attrs methods }`
#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub parent: Option<String>,
    pub parent_args: Vec<Expression>,
    pub attributes: Vec<Attribute>,
    pub methods: Vec<Method>,
}

impl Visitable for ClassDecl {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_class_decl(self);
    }
}

