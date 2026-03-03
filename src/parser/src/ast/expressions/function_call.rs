use crate::ast::{Expression, Visitable, Visitor};

/// `func_name(args)`
#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expression>,
}

impl Visitable for FunctionCall {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_function_call(self);
    }
}

