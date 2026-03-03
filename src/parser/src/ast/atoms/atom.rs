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

    /// Returns the source span of this atom.
    pub fn span(&self) -> Span {
        match self {
            Atom::NumberLiteral(lit) | Atom::BooleanLiteral(lit) | Atom::StringLiteral(lit) => {
                match lit {
                    Literal::Number(_, p) | Literal::Str(_, p) | Literal::Bool(_, p) => *p,
                }
            }
            Atom::Variable(id) => id.position,
            Atom::Group(g) => {
                let start = match &g.open_paren {
                    GroupingOperator::OpenParen(p) => p.start,
                    _ => 0,
                };
                let end = match &g.close_paren {
                    GroupingOperator::CloseParen(p) => p.end,
                    _ => 0,
                };
                Span::new(start, end)
            }
        }
    }
}

impl Visitable for Atom {
    fn accept<V: Visitor>(&self, visitor: &mut V) {
        visitor.visit_atom(self);
    }
}
