use crate::ast::{Expression, ExprBody, Visitable, Visitor};

/// `case expr of { branches }`
#[derive(Debug, Clone)]
pub struct CaseExpr {
    pub expr: Expression,
    pub branches: Vec<CaseBranch>,
}

/// A single branch: `ID : Type -> body`
#[derive(Debug, Clone)]
pub struct CaseBranch {
    pub name: String,
    pub type_name: String,
    pub body: ExprBody,
}

impl Visitable for CaseExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_case_expr(self);
    }
}

