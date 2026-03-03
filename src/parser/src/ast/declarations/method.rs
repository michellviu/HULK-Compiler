use crate::ast::{Body, Param, Visitable, Visitor};

/// Method: `name(params) [: ReturnType] body`
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
    pub body: Body,
}

impl Visitable for Method {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_method(self);
    }
}

