use super::super::*;
use crate::tokens::*;

#[derive(Debug, Clone)]
pub enum Atom {
    NumberLiteral(Literal),
    BooleanLiteral(Literal),
    StringLiteral(Literal),
    Variable(Identifier),
    Group(Box<group::Group>),
}

impl Atom {
    pub fn new_identifier(start: usize, end: usize, id: &str) -> Self {
        Atom::Variable(Identifier {
            name: id.to_string(),
            position: Position::new(start, end),
        })
    }

    pub fn new_grouped_expression(group: Group) -> Self {
        Atom::Group(Box::new(group))
    }
}

impl Visitable for Atom {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_atom(self);
    }
}
