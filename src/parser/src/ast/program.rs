use crate::ast::{ClassDecl, FunctionDecl, ExprBody, Visitable, Visitor};
use crate::tokens::Span;

/// Root AST node: `classes* functions* [expr ; | { expr; ... }]`
#[derive(Debug, Clone)]
pub struct Program {
    pub classes: Vec<ClassDecl>,
    pub functions: Vec<FunctionDecl>,
    pub entry: Option<ExprBody>,
    pub span: Span,
}

impl Visitable for Program {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_program(self);
    }
}

