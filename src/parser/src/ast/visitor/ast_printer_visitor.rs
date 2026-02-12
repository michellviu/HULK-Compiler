use crate::ast::atoms::atom::Atom;
use crate::ast::Expression;
use crate::visitor::Visitable;
use crate::visitor::Visitor;

pub struct AstPrinterVisitor {
    pub indent: usize,
}

impl AstPrinterVisitor {
    pub fn new() -> Self {
        AstPrinterVisitor { indent: 0 }
    }
    fn pad(&self) -> String {
        "  ".repeat(self.indent)
    }
}

impl Visitor for AstPrinterVisitor {

    fn visit_expression(&mut self, expr: &ast::Expression) {
        match expr {
            Expression::BinaryOp(binop) => binop.accept(self),
            Expression::Atom(atom) => atom.accept(self),
            Expression::UnaryOp(unary_op) => unary_op.accept(self),
        }
    }

    fn visit_atom(&mut self, atom: &ast::atoms::atom::Atom) {
        use crate::ast::atoms::atom::Atom::*;
        match atom {
            NumberLiteral(lit) => lit.accept(self),
            BooleanLiteral(lit) => lit.accept(self),
            StringLiteral(lit) => lit.accept(self),
            Group(group) => {
                println!("{}Group:", self.pad());
                self.indent += 1;
                group.expression.accept(self);
                self.indent -= 1;
            }
            Variable(id) => {
                println!("{}Variable: {}", self.pad(), id.name);
            }
        }
    }

    fn visit_binary_op(&mut self, binop: &ast::expressions::binoperation::BinaryOp) {
        use crate::tokens::BinOp;
        match &binop.operator {
            BinOp::Assign(_) => {
                println!("{}DestructiveAssign:", self.pad());
                self.indent += 1;
                binop.left.accept(self);
                binop.right.accept(self);
                self.indent -= 1;
            }
            _ => {
                println!("{}BinaryOp: {}", self.pad(), binop.operator);
                self.indent += 1;
                binop.left.accept(self);
                binop.right.accept(self);
                self.indent -= 1;
            }
        }
    }

    fn visit_literal(&mut self, literal: &tokens::Literal) {
        let type_str = match literal {
            tokens::Literal::Number(_, _) => "Number",
            tokens::Literal::Str(_, _) => "String",
            tokens::Literal::Bool(_, _) => "Bool",
        };
        println!("{}{}Literal: {}", self.pad(), type_str, literal);
    }

    fn visit_identifier(&mut self, identifier: &tokens::Identifier) {
        println!("{}Identifier: {}", self.pad(), identifier);
    }

    fn visit_unary_op(&mut self, unary_op: &ast::expressions::unaryoperation::UnaryOp) {
        println!("{}UnaryOp: {}", self.pad(), unary_op.op);
        self.indent += 1;
        unary_op.expr.accept(self);
        self.indent -= 1;
    }
}
