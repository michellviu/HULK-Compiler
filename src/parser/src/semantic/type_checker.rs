//! Pass 3: Type checking and inference.
//!
//! Traverses every expression, infers its type, and verifies that:
//! - Operators are applied to operands of the correct type
//! - Conditions are Boolean
//! - Function/method arguments conform to parameter types
//! - Return types match declared types
//! - Assignment values conform to the target's type
//! - Member access and method calls are valid on the type

use crate::ast::{self};
use crate::tokens;

use super::errors::CompilerError;
use super::symbol_table::SymbolTable;
use super::types::HulkType;

/// Runs type checking on the full program.
/// Requires that `collect_declarations` and `check_semantics` have
/// already been run successfully.
pub fn check_types(
    program: &ast::Program,
    symbols: &mut SymbolTable,
) -> Vec<CompilerError> {
    let mut checker = TypeChecker {
        symbols,
        errors: Vec::new(),
        in_method: false,
    };

    // Type-check function bodies.
    for func in &program.functions {
        checker.check_function(func);
    }

    // Type-check class method/attribute bodies.
    for class in &program.classes {
        checker.check_class(class);
    }

    // Type-check entry expression.
    if let Some(ref entry) = program.entry {
        checker.symbols.push_scope();
        checker.infer_expr_body(entry);
        checker.symbols.pop_scope();
    }

    checker.errors
}

// ═══════════════════════════════════════════════════════════════════
// Type checker
// ═══════════════════════════════════════════════════════════════════

struct TypeChecker<'a> {
    symbols: &'a mut SymbolTable,
    errors: Vec<CompilerError>,
    in_method: bool,
}

impl<'a> TypeChecker<'a> {
    // ── Top-level ───────────────────────────────────────────────

    fn check_function(&mut self, func: &ast::FunctionDecl) {
        self.symbols.push_scope();

        for p in &func.params {
            let t = match &p.type_ann {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Unknown,
            };
            self.symbols.define_var(&p.name, t, p.span);
        }

        let body_type = self.infer_body(&func.body);

        // Check return type if annotated.
        if let Some(ref ret_ann) = func.return_type {
            let expected = HulkType::from_name(ret_ann);
            if expected.is_resolved() && body_type.is_resolved() {
                if !self.symbols.conforms_to(&body_type, &expected) {
                    self.errors.push(CompilerError::return_type_mismatch(
                        &func.name,
                        &expected.type_name(),
                        &body_type.type_name(),
                        func.span,
                    ));
                }
            }
        }

        self.symbols.pop_scope();
    }

    fn check_class(&mut self, class: &ast::ClassDecl) {
        self.symbols.current_class = Some(class.name.clone());

        // Type-check attribute initializers.
        for attr in &class.attributes {
            self.symbols.push_scope();
            // Constructor params available in attribute initializers.
            for p in &class.params {
                let t = match &p.type_ann {
                    Some(a) => HulkType::from_name(a),
                    None => HulkType::Unknown,
                };
                self.symbols.define_var(&p.name, t, p.span);
            }
            self.in_method = true;
            let init_type = self.infer_expression(&attr.init);
            self.in_method = false;

            if let Some(ref ann) = attr.type_ann {
                let expected = HulkType::from_name(ann);
                if expected.is_resolved() && init_type.is_resolved() {
                    if !self.symbols.conforms_to(&init_type, &expected) {
                        self.errors.push(CompilerError::type_mismatch(
                            &expected.type_name(),
                            &init_type.type_name(),
                            attr.span,
                        ));
                    }
                }
            }
            self.symbols.pop_scope();
        }

        // Type-check methods.
        for method in &class.methods {
            self.check_method(method);
        }

        self.symbols.current_class = None;
    }

    fn check_method(&mut self, method: &ast::Method) {
        self.in_method = true;
        self.symbols.push_scope();

        let self_type = match &self.symbols.current_class {
            Some(name) => HulkType::Class(name.clone()),
            None => HulkType::Error,
        };
        self.symbols.define_var("self", self_type, method.span);

        for p in &method.params {
            let t = match &p.type_ann {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Unknown,
            };
            self.symbols.define_var(&p.name, t, p.span);
        }

        let body_type = self.infer_body(&method.body);

        if let Some(ref ret_ann) = method.return_type {
            let expected = HulkType::from_name(ret_ann);
            if expected.is_resolved() && body_type.is_resolved() {
                if !self.symbols.conforms_to(&body_type, &expected) {
                    self.errors.push(CompilerError::return_type_mismatch(
                        &method.name,
                        &expected.type_name(),
                        &body_type.type_name(),
                        method.span,
                    ));
                }
            }
        }

        self.symbols.pop_scope();
        self.in_method = false;
    }

    // ── Body inference ──────────────────────────────────────────

    fn infer_body(&mut self, body: &ast::Body) -> HulkType {
        match body {
            ast::Body::Inline(expr) => self.infer_expression(expr),
            ast::Body::Block(exprs) => {
                let mut t = HulkType::Void;
                for expr in exprs {
                    t = self.infer_expression(expr);
                }
                t // type of a block is the type of the last expression
            }
        }
    }

    fn infer_expr_body(&mut self, body: &ast::ExprBody) -> HulkType {
        match body {
            ast::ExprBody::Single(expr) => self.infer_expression(expr),
            ast::ExprBody::Block(exprs) => {
                let mut t = HulkType::Void;
                for expr in exprs {
                    t = self.infer_expression(expr);
                }
                t
            }
        }
    }

    // ── Expression inference ────────────────────────────────────

    fn infer_expression(&mut self, expr: &ast::Expression) -> HulkType {
        match expr {
            ast::Expression::Atom(atom) => self.infer_atom(atom),
            ast::Expression::BinaryOp(binop) => self.infer_binary_op(binop),
            ast::Expression::UnaryOp(unary) => self.infer_unary_op(unary),
            ast::Expression::Let(let_expr) => self.infer_let(let_expr),
            ast::Expression::If(if_expr) => self.infer_if(if_expr),
            ast::Expression::While(while_expr) => self.infer_while(while_expr),
            ast::Expression::Case(case_expr) => self.infer_case(case_expr),
            ast::Expression::Assign(assign) => self.infer_assign(assign),
            ast::Expression::FunctionCall(call) => self.infer_function_call(call),
            ast::Expression::MemberAccess(access) => self.infer_member_access(access),
            ast::Expression::MethodCall(call) => self.infer_method_call(call),
            ast::Expression::IndexAccess(access) => self.infer_index_access(access),
            ast::Expression::NewInstance(inst) => self.infer_new_instance(inst),
            ast::Expression::NewArray(arr) => self.infer_new_array(arr),
        }
    }

    fn infer_atom(&mut self, atom: &ast::atoms::atom::Atom) -> HulkType {
        match atom {
            ast::atoms::atom::Atom::NumberLiteral(_) => HulkType::Number,
            ast::atoms::atom::Atom::StringLiteral(_) => HulkType::String,
            ast::atoms::atom::Atom::BooleanLiteral(_) => HulkType::Boolean,
            ast::atoms::atom::Atom::Variable(id) => {
                if id.name == "self" {
                    match &self.symbols.current_class {
                        Some(name) => HulkType::Class(name.clone()),
                        None => HulkType::Error,
                    }
                } else {
                    self.symbols
                        .var_type(&id.name)
                        .unwrap_or(HulkType::Error)
                }
            }
            ast::atoms::atom::Atom::Group(group) => {
                self.infer_expression(&group.expression)
            }
        }
    }

    fn infer_binary_op(&mut self, binop: &ast::expressions::binoperation::BinaryOp) -> HulkType {
        let left_type = self.infer_expression(&binop.left);
        let right_type = self.infer_expression(&binop.right);

        // Skip checking if either side already has an error.
        if left_type.is_error() || right_type.is_error() {
            return HulkType::Error;
        }

        match &binop.operator {
            // Arithmetic: Number × Number → Number
            tokens::BinOp::Plus(_)
            | tokens::BinOp::Minus(_)
            | tokens::BinOp::Mul(_)
            | tokens::BinOp::Div(_)
            | tokens::BinOp::Mod(_)
            | tokens::BinOp::Pow(_) => {
                if left_type != HulkType::Number {
                    self.errors.push(CompilerError::binary_op_type_error(
                        &binop.operator.to_string(),
                        &left_type.type_name(),
                        &right_type.type_name(),
                        binop.span,
                    ));
                    return HulkType::Error;
                }
                if right_type != HulkType::Number {
                    self.errors.push(CompilerError::binary_op_type_error(
                        &binop.operator.to_string(),
                        &left_type.type_name(),
                        &right_type.type_name(),
                        binop.span,
                    ));
                    return HulkType::Error;
                }
                HulkType::Number
            }

            // Comparison: Number × Number → Boolean
            tokens::BinOp::Less(_)
            | tokens::BinOp::LessEqual(_)
            | tokens::BinOp::Greater(_)
            | tokens::BinOp::GreaterEqual(_) => {
                if left_type != HulkType::Number || right_type != HulkType::Number {
                    self.errors.push(CompilerError::binary_op_type_error(
                        &binop.operator.to_string(),
                        &left_type.type_name(),
                        &right_type.type_name(),
                        binop.span,
                    ));
                    return HulkType::Error;
                }
                HulkType::Boolean
            }

            // Equality: T × T → Boolean (any matching types)
            tokens::BinOp::EqualEqual(_) | tokens::BinOp::NotEqual(_) => {
                // Allow comparing any types for equality.
                HulkType::Boolean
            }

            // Logical: Boolean × Boolean → Boolean
            tokens::BinOp::And(_) | tokens::BinOp::Or(_) => {
                if left_type != HulkType::Boolean || right_type != HulkType::Boolean {
                    self.errors.push(CompilerError::binary_op_type_error(
                        &binop.operator.to_string(),
                        &left_type.type_name(),
                        &right_type.type_name(),
                        binop.span,
                    ));
                    return HulkType::Error;
                }
                HulkType::Boolean
            }

            // Concatenation: Any × Any → String
            tokens::BinOp::Concat(_) | tokens::BinOp::ConcatSpaced(_) => HulkType::String,

            // Assignment operators  — should not appear as binary ops.
            tokens::BinOp::Equal(_) | tokens::BinOp::Assign(_) => HulkType::Error,
        }
    }

    fn infer_unary_op(&mut self, unary: &ast::expressions::unaryoperation::UnaryOp) -> HulkType {
        let operand_type = self.infer_expression(&unary.expr);
        if operand_type.is_error() {
            return HulkType::Error;
        }

        match &unary.op {
            tokens::UnaryOp::Minus(_) => {
                if operand_type != HulkType::Number {
                    self.errors.push(CompilerError::unary_op_type_error(
                        "-",
                        &operand_type.type_name(),
                        unary.span,
                    ));
                    return HulkType::Error;
                }
                HulkType::Number
            }
            tokens::UnaryOp::Not(_) => {
                if operand_type != HulkType::Boolean {
                    self.errors.push(CompilerError::unary_op_type_error(
                        "!",
                        &operand_type.type_name(),
                        unary.span,
                    ));
                    return HulkType::Error;
                }
                HulkType::Boolean
            }
        }
    }

    fn infer_let(&mut self, let_expr: &ast::LetExpr) -> HulkType {
        self.symbols.push_scope();

        for decl in &let_expr.decls {
            let init_type = self.infer_expression(&decl.value);

            let declared_type = match &decl.type_ann {
                Some(ann) => {
                    let expected = HulkType::from_name(ann);
                    if expected.is_resolved() && init_type.is_resolved() {
                        if !self.symbols.conforms_to(&init_type, &expected) {
                            self.errors.push(CompilerError::type_mismatch(
                                &expected.type_name(),
                                &init_type.type_name(),
                                decl.span,
                            ));
                        }
                    }
                    expected
                }
                None => init_type,
            };

            self.symbols.define_var(&decl.name, declared_type, decl.span);
        }

        let result = self.infer_expr_body(&let_expr.body);
        self.symbols.pop_scope();
        result
    }

    fn infer_if(&mut self, if_expr: &ast::IfExpr) -> HulkType {
        let cond_type = self.infer_expression(&if_expr.condition);
        if cond_type.is_resolved() && cond_type != HulkType::Boolean {
            self.errors
                .push(CompilerError::condition_not_boolean(if_expr.span));
        }

        let then_type = self.infer_expr_body(&if_expr.then_body);

        let mut result_type = then_type;

        for branch in &if_expr.elif_branches {
            let bc = self.infer_expression(&branch.condition);
            if bc.is_resolved() && bc != HulkType::Boolean {
                self.errors
                    .push(CompilerError::condition_not_boolean(branch.span));
            }
            let bt = self.infer_expr_body(&branch.body);
            result_type = self.symbols.lca(&result_type, &bt);
        }

        if let Some(ref else_body) = if_expr.else_body {
            let et = self.infer_expr_body(else_body);
            result_type = self.symbols.lca(&result_type, &et);
        }

        result_type
    }

    fn infer_while(&mut self, while_expr: &ast::WhileExpr) -> HulkType {
        let cond_type = self.infer_expression(&while_expr.condition);
        if cond_type.is_resolved() && cond_type != HulkType::Boolean {
            self.errors
                .push(CompilerError::condition_not_boolean(while_expr.span));
        }

        let body_type = self.infer_expr_body(&while_expr.body);

        match &while_expr.else_body {
            Some(else_body) => {
                let et = self.infer_expr_body(else_body);
                self.symbols.lca(&body_type, &et)
            }
            None => HulkType::Void,
        }
    }

    fn infer_case(&mut self, case_expr: &ast::CaseExpr) -> HulkType {
        self.infer_expression(&case_expr.expr);

        let mut result = HulkType::Error;

        for branch in &case_expr.branches {
            self.symbols.push_scope();
            let t = HulkType::from_name(&branch.type_name);
            self.symbols.define_var(&branch.name, t, branch.span);
            let bt = self.infer_expr_body(&branch.body);
            if result.is_error() {
                result = bt;
            } else {
                result = self.symbols.lca(&result, &bt);
            }
            self.symbols.pop_scope();
        }

        result
    }

    fn infer_assign(&mut self, assign: &ast::AssignExpr) -> HulkType {
        let target_type = self.infer_expression(&assign.target);
        let value_type = self.infer_expression(&assign.value);

        if target_type.is_resolved() && value_type.is_resolved() {
            if !self.symbols.conforms_to(&value_type, &target_type) {
                self.errors.push(CompilerError::type_does_not_conform(
                    &value_type.type_name(),
                    &target_type.type_name(),
                    assign.span,
                ));
            }
        }

        value_type
    }

    fn infer_function_call(&mut self, call: &ast::FunctionCall) -> HulkType {
        // Check arguments.
        let arg_types: Vec<HulkType> = call.args.iter().map(|a| self.infer_expression(a)).collect();

        match self.symbols.get_function(&call.name).cloned() {
            Some(func_info) => {
                // Check argument types conform to parameter types.
                for (i, ((_pname, ptype), atype)) in
                    func_info.params.iter().zip(arg_types.iter()).enumerate()
                {
                    if ptype.is_resolved() && atype.is_resolved() {
                        if !self.symbols.conforms_to(atype, ptype) {
                            self.errors.push(CompilerError::type_does_not_conform(
                                &atype.type_name(),
                                &ptype.type_name(),
                                call.args.get(i).map(|a| a.span()).unwrap_or(call.span),
                            ));
                        }
                    }
                }
                func_info.return_type.clone()
            }
            None => HulkType::Error, // already reported by semantic checker
        }
    }

    fn infer_member_access(&mut self, access: &ast::MemberAccess) -> HulkType {
        let obj_type = self.infer_expression(&access.object);

        let class_name = self.resolve_class_name(&obj_type);
        match class_name {
            Some(name) => {
                match self.symbols.resolve_attribute(&name, &access.member) {
                    Some((_owner, attr)) => attr.hulk_type.clone(),
                    None => {
                        self.errors.push(CompilerError::undefined_member(
                            &name,
                            &access.member,
                            access.span,
                        ));
                        HulkType::Error
                    }
                }
            }
            None => {
                if !obj_type.is_error() {
                    self.errors.push(CompilerError::undefined_member(
                        &obj_type.type_name(),
                        &access.member,
                        access.span,
                    ));
                }
                HulkType::Error
            }
        }
    }

    fn infer_method_call(&mut self, call: &ast::MethodCall) -> HulkType {
        let obj_type = self.infer_expression(&call.object);
        let arg_types: Vec<HulkType> = call.args.iter().map(|a| self.infer_expression(a)).collect();

        let class_name = self.resolve_class_name(&obj_type);
        match class_name {
            Some(name) => {
                match self.symbols.resolve_method(&name, &call.method) {
                    Some((_owner, method_info)) => {
                        let method_info = method_info.clone();
                        // Check arity.
                        if method_info.params.len() != arg_types.len() {
                            self.errors.push(CompilerError::wrong_arity(
                                &call.method,
                                method_info.params.len(),
                                arg_types.len(),
                                call.span,
                            ));
                        } else {
                            // Check argument types.
                            for (i, ((_, ptype), atype)) in
                                method_info.params.iter().zip(arg_types.iter()).enumerate()
                            {
                                if ptype.is_resolved() && atype.is_resolved() {
                                    if !self.symbols.conforms_to(atype, ptype) {
                                        self.errors.push(CompilerError::type_does_not_conform(
                                            &atype.type_name(),
                                            &ptype.type_name(),
                                            call.args.get(i).map(|a: &ast::Expression| a.span()).unwrap_or(call.span),
                                        ));
                                    }
                                }
                            }
                        }
                        method_info.return_type.clone()
                    }
                    None => {
                        self.errors.push(CompilerError::undefined_method(
                            &name,
                            &call.method,
                            call.span,
                        ));
                        HulkType::Error
                    }
                }
            }
            None => {
                if !obj_type.is_error() {
                    self.errors.push(CompilerError::undefined_method(
                        &obj_type.type_name(),
                        &call.method,
                        call.span,
                    ));
                }
                HulkType::Error
            }
        }
    }

    fn infer_index_access(&mut self, access: &ast::IndexAccess) -> HulkType {
        let obj_type = self.infer_expression(&access.object);
        let idx_type = self.infer_expression(&access.index);

        if idx_type.is_resolved() && idx_type != HulkType::Number {
            self.errors.push(CompilerError::index_not_number(access.span));
        }

        match &obj_type {
            HulkType::Array(inner) => *inner.clone(),
            HulkType::Error => HulkType::Error,
            _ => {
                self.errors
                    .push(CompilerError::not_indexable(&obj_type.type_name(), access.span));
                HulkType::Error
            }
        }
    }

    fn infer_new_instance(&mut self, inst: &ast::NewInstance) -> HulkType {
        let arg_types: Vec<HulkType> = inst.args.iter().map(|a| self.infer_expression(a)).collect();

        if let Some(class) = self.symbols.get_class(&inst.type_name).cloned() {
            // Check constructor argument types.
            for (i, ((_, ptype), atype)) in class.params.iter().zip(arg_types.iter()).enumerate() {
                if ptype.is_resolved() && atype.is_resolved() {
                    if !self.symbols.conforms_to(atype, ptype) {
                        self.errors.push(CompilerError::type_does_not_conform(
                            &atype.type_name(),
                            &ptype.type_name(),
                            inst.args.get(i).map(|a| a.span()).unwrap_or(inst.span),
                        ));
                    }
                }
            }
        }

        HulkType::Class(inst.type_name.clone())
    }

    fn infer_new_array(&mut self, arr: &ast::NewArray) -> HulkType {
        let size_type = self.infer_expression(&arr.size);
        if size_type.is_resolved() && size_type != HulkType::Number {
            self.errors.push(CompilerError::type_mismatch(
                "Number",
                &size_type.type_name(),
                arr.span,
            ));
        }

        let element_type = match &arr.type_name {
            Some(name) => HulkType::from_name(name),
            None => HulkType::Unknown,
        };

        if let Some((ref _var, ref init)) = arr.init {
            self.infer_expression(init);
        }

        HulkType::Array(Box::new(element_type))
    }

    // ── Helpers ─────────────────────────────────────────────────

    /// Resolves a HulkType to a class name (for member/method lookup).
    fn resolve_class_name(&self, t: &HulkType) -> Option<String> {
        match t {
            HulkType::Class(name) => Some(name.clone()),
            HulkType::Number => Some("Number".into()),
            HulkType::String => Some("String".into()),
            HulkType::Boolean => Some("Boolean".into()),
            HulkType::Object => Some("Object".into()),
            HulkType::SelfType => self.symbols.current_class.clone(),
            _ => None,
        }
    }
}
