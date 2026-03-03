use crate::ast::{Expression, Visitable, Visitor};

/// `new Type?[size] { id -> init_expr }?`
#[derive(Debug, Clone)]
pub struct NewArray {
    pub type_name: Option<String>,
    pub size: Expression,
    pub init: Option<(String, Expression)>,
}

impl Visitable for NewArray {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_new_array(self);
    }
}

