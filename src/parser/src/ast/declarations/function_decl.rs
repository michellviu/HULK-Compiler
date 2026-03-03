use crate::ast::{Body, Param, Visitable, Visitor};
use crate::tokens::Span;

/// `function name(params) [: ReturnType] body`
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
    pub body: Body,
    pub span: Span,
}

impl Visitable for FunctionDecl {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_function_decl(self);
    }
}

