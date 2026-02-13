use crate::ast;
use crate::tokens;

pub trait Visitor {
    fn visit_expression(&mut self, expr: &ast::Expression);
    fn visit_atom(&mut self, atom: &ast::atoms::atom::Atom);
    fn visit_binary_op(&mut self, binop: &ast::expressions::binoperation::BinaryOp);
    fn visit_unary_op(&mut self, unary_op: &ast::expressions::unaryoperation::UnaryOp);
    fn visit_identifier(&mut self, identifier: &tokens::Identifier);
    fn visit_literal(&mut self, literal: &tokens::Literal);
    

}

pub trait Visitable {
    fn accept<V: Visitor>(&self, visitor: &mut V);
}
