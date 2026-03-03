use crate::ast::{Expression, ExprBody, Visitable, Visitor};

/// `while (condition) body [else else_body]`
#[derive(Debug, Clone)]
pub struct WhileExpr {
    pub condition: Expression,
    pub body: ExprBody,
    pub else_body: Option<ExprBody>,
}

impl Visitable for WhileExpr {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_while_expr(self);
    }
}

