use super::super::Expression;
use crate::tokens::GroupingOperator;
use super::super::Visitable;
use super::super::Visitor;

#[derive(Debug,Clone)]
pub struct Group{
    pub open_paren: GroupingOperator,
    pub close_paren: GroupingOperator,
    pub expression: Expression,
}

impl Group {
    pub fn new(
        open_paren: GroupingOperator,
        expression: Expression,
        close_paren: GroupingOperator,
    ) -> Self {
        Group {
            open_paren,
            close_paren,
            expression,
        }
    }
}

impl Visitable for Group {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_group(self);
    }
    
}