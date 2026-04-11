use crate::ast;
use crate::tokens;

pub trait Visitor {
    // Program
    fn visit_program(&mut self, program: &ast::Program);

    // Declarations
    fn visit_class_decl(&mut self, class: &ast::ClassDecl);
    fn visit_function_decl(&mut self, func: &ast::FunctionDecl);
    fn visit_method(&mut self, method: &ast::Method);
    fn visit_attribute(&mut self, attr: &ast::Attribute);

    // Expressions
    fn visit_expression(&mut self, expr: &ast::Expression);
    fn visit_atom(&mut self, atom: &ast::atoms::atom::Atom);
    fn visit_binary_op(&mut self, binop: &ast::expressions::binoperation::BinaryOp);
    fn visit_unary_op(&mut self, unary_op: &ast::expressions::unaryoperation::UnaryOp);
    fn visit_let_expr(&mut self, let_expr: &ast::LetExpr);
    fn visit_if_expr(&mut self, if_expr: &ast::IfExpr);
    fn visit_while_expr(&mut self, while_expr: &ast::WhileExpr);
    fn visit_for_expr(&mut self, for_expr: &ast::ForExpr);
    fn visit_is_expr(&mut self, is_expr: &ast::IsExpr);
    fn visit_as_expr(&mut self, as_expr: &ast::AsExpr);
    fn visit_case_expr(&mut self, case_expr: &ast::CaseExpr);
    fn visit_assign_expr(&mut self, assign: &ast::AssignExpr);
    fn visit_member_access(&mut self, access: &ast::MemberAccess);
    fn visit_method_call(&mut self, call: &ast::MethodCall);
    fn visit_index_access(&mut self, access: &ast::IndexAccess);
    fn visit_function_call(&mut self, call: &ast::FunctionCall);
    fn visit_new_instance(&mut self, inst: &ast::NewInstance);
    fn visit_new_array(&mut self, arr: &ast::NewArray);

    // Tokens
    fn visit_identifier(&mut self, identifier: &tokens::Identifier);
    fn visit_literal(&mut self, literal: &tokens::Literal);
}

pub trait Visitable {
    fn accept<V: Visitor>(&self, visitor: &mut V);
}
