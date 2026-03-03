use crate::ast::{Expression, ExprBody, Visitable, Visitor};
use crate::tokens::Span;

/// `let x: Type = val, y = val2 in body`
#[derive(Debug, Clone)]
pub struct LetExpr {
    pub decls: Vec<LetDecl>,
    pub body: ExprBody,
    pub span: Span,
}

/// A single declaration inside a let: `ID [: Type] = expr`
#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub type_ann: Option<String>,
    pub value: Expression,
    pub span: Span,
}

impl Visitable for LetExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_let_expr(self);
    }
}

