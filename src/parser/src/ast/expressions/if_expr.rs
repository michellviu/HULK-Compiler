use crate::ast::{Expression, ExprBody, Visitable, Visitor};
use crate::tokens::Span;

/// `if (cond) body elif (cond2) body2 else body3`
#[derive(Debug, Clone)]
pub struct IfExpr {
    pub condition: Expression,
    pub then_body: ExprBody,
    pub elif_branches: Vec<CondBranch>,
    pub else_body: Option<ExprBody>,
    pub span: Span,
}

/// A single elif branch: `elif (condition) body`
#[derive(Debug, Clone)]
pub struct CondBranch {
    pub condition: Expression,
    pub body: ExprBody,
    pub span: Span,
}

impl Visitable for IfExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_if_expr(self);
    }
}

