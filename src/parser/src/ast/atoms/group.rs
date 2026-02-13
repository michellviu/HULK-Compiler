use super::super::Expression;
use crate::tokens::GroupingOperator;

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