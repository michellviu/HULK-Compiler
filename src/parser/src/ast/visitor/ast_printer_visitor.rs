use crate::{ast, tokens};
use crate::ast::Expression;
use crate::ast::visitor::Visitable;
use crate::ast::visitor::Visitor;

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

    fn print_expr_body(&mut self, body: &ast::ExprBody) {
        match body {
            ast::ExprBody::Single(expr) => expr.accept(self),
            ast::ExprBody::Block(exprs) => {
                println!("{}Block:", self.pad());
                self.indent += 1;
                for expr in exprs {
                    expr.accept(self);
                }
                self.indent -= 1;
            }
        }
    }

    fn print_body(&mut self, body: &ast::Body) {
        match body {
            ast::Body::Inline(expr) => expr.accept(self),
            ast::Body::Block(exprs) => {
                println!("{}Block:", self.pad());
                self.indent += 1;
                for expr in exprs {
                    expr.accept(self);
                }
                self.indent -= 1;
            }
        }
    }

    fn print_params(&self, params: &[ast::Param]) {
        for p in params {
            if let Some(ref t) = p.type_ann {
                print!("{}: {}", p.name, t);
            } else {
                print!("{}", p.name);
            }
            print!(", ");
        }
    }
}

impl Visitor for AstPrinterVisitor {

    // ── Program ─────────────────────────────────────────────────

    fn visit_program(&mut self, program: &ast::Program) {
        println!("{}Program:", self.pad());
        self.indent += 1;
        for class in &program.classes {
            class.accept(self);
        }
        for func in &program.functions {
            func.accept(self);
        }
        if let Some(ref entry) = program.entry {
            println!("{}Entry:", self.pad());
            self.indent += 1;
            self.print_expr_body(entry);
            self.indent -= 1;
        }
        self.indent -= 1;
    }

    // ── Declarations ────────────────────────────────────────────

    fn visit_class_decl(&mut self, class: &ast::ClassDecl) {
        print!("{}ClassDecl: {}", self.pad(), class.name);
        if !class.params.is_empty() {
            print!("(");
            self.print_params(&class.params);
            print!(")");
        }
        if let Some(ref parent) = class.parent {
            print!(" is {}", parent);
        }
        println!();
        self.indent += 1;
        for attr in &class.attributes {
            attr.accept(self);
        }
        for method in &class.methods {
            method.accept(self);
        }
        self.indent -= 1;
    }

    fn visit_function_decl(&mut self, func: &ast::FunctionDecl) {
        print!("{}FunctionDecl: {}(", self.pad(), func.name);
        self.print_params(&func.params);
        print!(")");
        if let Some(ref ret) = func.return_type {
            print!(": {}", ret);
        }
        println!();
        self.indent += 1;
        self.print_body(&func.body);
        self.indent -= 1;
    }

    fn visit_method(&mut self, method: &ast::Method) {
        print!("{}Method: {}(", self.pad(), method.name);
        self.print_params(&method.params);
        print!(")");
        if let Some(ref ret) = method.return_type {
            print!(": {}", ret);
        }
        println!();
        self.indent += 1;
        self.print_body(&method.body);
        self.indent -= 1;
    }

    fn visit_attribute(&mut self, attr: &ast::Attribute) {
        print!("{}Attribute: {}", self.pad(), attr.name);
        if let Some(ref t) = attr.type_ann {
            print!(": {}", t);
        }
        println!(" =");
        self.indent += 1;
        attr.init.accept(self);
        self.indent -= 1;
    }

    // ── Expressions ─────────────────────────────────────────────

    fn visit_expression(&mut self, expr: &ast::Expression) {
        match expr {
            Expression::BinaryOp(binop) => binop.accept(self),
            Expression::UnaryOp(unary_op) => unary_op.accept(self),
            Expression::Atom(atom) => atom.accept(self),
            Expression::Let(let_expr) => let_expr.accept(self),
            Expression::If(if_expr) => if_expr.accept(self),
            Expression::While(while_expr) => while_expr.accept(self),
            Expression::Case(case_expr) => case_expr.accept(self),
            Expression::Assign(assign) => assign.accept(self),
            Expression::MemberAccess(access) => access.accept(self),
            Expression::MethodCall(call) => call.accept(self),
            Expression::IndexAccess(access) => access.accept(self),
            Expression::FunctionCall(call) => call.accept(self),
            Expression::NewInstance(inst) => inst.accept(self),
            Expression::NewArray(arr) => arr.accept(self),
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
        println!("{}BinaryOp: {}", self.pad(), binop.operator);
        self.indent += 1;
        binop.left.accept(self);
        binop.right.accept(self);
        self.indent -= 1;
    }

    fn visit_unary_op(&mut self, unary_op: &ast::expressions::unaryoperation::UnaryOp) {
        println!("{}UnaryOp: {}", self.pad(), unary_op.op);
        self.indent += 1;
        unary_op.expr.accept(self);
        self.indent -= 1;
    }

    fn visit_let_expr(&mut self, let_expr: &ast::LetExpr) {
        println!("{}LetExpr:", self.pad());
        self.indent += 1;
        println!("{}Declarations:", self.pad());
        self.indent += 1;
        for decl in &let_expr.decls {
            print!("{}Decl: {}", self.pad(), decl.name);
            if let Some(ref t) = decl.type_ann {
                print!(": {}", t);
            }
            println!(" =");
            self.indent += 1;
            decl.value.accept(self);
            self.indent -= 1;
        }
        self.indent -= 1;
        println!("{}In:", self.pad());
        self.indent += 1;
        self.print_expr_body(&let_expr.body);
        self.indent -= 1;
        self.indent -= 1;
    }

    fn visit_if_expr(&mut self, if_expr: &ast::IfExpr) {
        println!("{}IfExpr:", self.pad());
        self.indent += 1;
        println!("{}Condition:", self.pad());
        self.indent += 1;
        if_expr.condition.accept(self);
        self.indent -= 1;
        println!("{}Then:", self.pad());
        self.indent += 1;
        self.print_expr_body(&if_expr.then_body);
        self.indent -= 1;
        for branch in &if_expr.elif_branches {
            println!("{}Elif:", self.pad());
            self.indent += 1;
            println!("{}Condition:", self.pad());
            self.indent += 1;
            branch.condition.accept(self);
            self.indent -= 1;
            println!("{}Then:", self.pad());
            self.indent += 1;
            self.print_expr_body(&branch.body);
            self.indent -= 1;
            self.indent -= 1;
        }
        if let Some(ref else_body) = if_expr.else_body {
            println!("{}Else:", self.pad());
            self.indent += 1;
            self.print_expr_body(else_body);
            self.indent -= 1;
        }
        self.indent -= 1;
    }

    fn visit_while_expr(&mut self, while_expr: &ast::WhileExpr) {
        println!("{}WhileExpr:", self.pad());
        self.indent += 1;
        println!("{}Condition:", self.pad());
        self.indent += 1;
        while_expr.condition.accept(self);
        self.indent -= 1;
        println!("{}Body:", self.pad());
        self.indent += 1;
        self.print_expr_body(&while_expr.body);
        self.indent -= 1;
        if let Some(ref else_body) = while_expr.else_body {
            println!("{}Else:", self.pad());
            self.indent += 1;
            self.print_expr_body(else_body);
            self.indent -= 1;
        }
        self.indent -= 1;
    }

    fn visit_case_expr(&mut self, case_expr: &ast::CaseExpr) {
        println!("{}CaseExpr:", self.pad());
        self.indent += 1;
        println!("{}Expr:", self.pad());
        self.indent += 1;
        case_expr.expr.accept(self);
        self.indent -= 1;
        for branch in &case_expr.branches {
            println!("{}{}: {} ->", self.pad(), branch.name, branch.type_name);
            self.indent += 1;
            self.print_expr_body(&branch.body);
            self.indent -= 1;
        }
        self.indent -= 1;
    }

    fn visit_assign_expr(&mut self, assign: &ast::AssignExpr) {
        println!("{}AssignExpr: :=", self.pad());
        self.indent += 1;
        assign.target.accept(self);
        assign.value.accept(self);
        self.indent -= 1;
    }

    fn visit_member_access(&mut self, access: &ast::MemberAccess) {
        println!("{}MemberAccess: .{}", self.pad(), access.member);
        self.indent += 1;
        access.object.accept(self);
        self.indent -= 1;
    }

    fn visit_method_call(&mut self, call: &ast::MethodCall) {
        println!("{}MethodCall: .{}()", self.pad(), call.method);
        self.indent += 1;
        println!("{}Object:", self.pad());
        self.indent += 1;
        call.object.accept(self);
        self.indent -= 1;
        if !call.args.is_empty() {
            println!("{}Args:", self.pad());
            self.indent += 1;
            for arg in &call.args {
                arg.accept(self);
            }
            self.indent -= 1;
        }
        self.indent -= 1;
    }

    fn visit_index_access(&mut self, access: &ast::IndexAccess) {
        println!("{}IndexAccess:", self.pad());
        self.indent += 1;
        println!("{}Object:", self.pad());
        self.indent += 1;
        access.object.accept(self);
        self.indent -= 1;
        println!("{}Index:", self.pad());
        self.indent += 1;
        access.index.accept(self);
        self.indent -= 1;
        self.indent -= 1;
    }

    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        println!("{}FunctionCall: {}()", self.pad(), call.name);
        self.indent += 1;
        if !call.args.is_empty() {
            println!("{}Args:", self.pad());
            self.indent += 1;
            for arg in &call.args {
                arg.accept(self);
            }
            self.indent -= 1;
        }
        self.indent -= 1;
    }

    fn visit_new_instance(&mut self, inst: &ast::NewInstance) {
        println!("{}NewInstance: {}", self.pad(), inst.type_name);
        self.indent += 1;
        if !inst.args.is_empty() {
            println!("{}Args:", self.pad());
            self.indent += 1;
            for arg in &inst.args {
                arg.accept(self);
            }
            self.indent -= 1;
        }
        self.indent -= 1;
    }

    fn visit_new_array(&mut self, arr: &ast::NewArray) {
        print!("{}NewArray", self.pad());
        if let Some(ref t) = arr.type_name {
            print!(": {}", t);
        }
        println!();
        self.indent += 1;
        println!("{}Size:", self.pad());
        self.indent += 1;
        arr.size.accept(self);
        self.indent -= 1;
        if let Some((ref var, ref init)) = arr.init {
            println!("{}Init: {} ->", self.pad(), var);
            self.indent += 1;
            init.accept(self);
            self.indent -= 1;
        }
        self.indent -= 1;
    }

    // ── Tokens ──────────────────────────────────────────────────

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
}
