//! Pass 2: Semantic analysis.
//!
//! Traverses all function/method bodies and the entry expression, verifying:
//! - Variables are declared before use
//! - Functions/methods exist and have correct arity
//! - Types referenced in annotations exist
//! - Assignment targets are valid (variable, member access, index access)
//! - `self` is only used inside methods
//! - Warns about unused variables

use crate::ast;

use super::errors::CompilerError;
use super::symbol_table::SymbolTable;
use super::types::HulkType;

/// Runs semantic analysis on the full program.
/// Requires that `collect_declarations` has already populated `symbols`.
pub fn check_semantics(
    program: &ast::Program,
    symbols: &mut SymbolTable,
) -> Vec<CompilerError> {
    let mut checker = SemanticChecker {
        symbols,
        errors: Vec::new(),
        in_method: false,
        current_method: None,
    };

    // Analyse function bodies.
    for func in &program.functions {
        checker.check_function(func);
    }

    // Analyse class method bodies.
    for class in &program.classes {
        checker.check_class(class);
    }

    // Analyse entry expression.
    if let Some(ref entry) = program.entry {
        checker.symbols.push_scope();
        checker.check_expr_body(entry);
        let vars = checker.symbols.pop_scope();
        checker.warn_unused(&vars);
    }

    checker.errors
}

// ═══════════════════════════════════════════════════════════════════
// Checker state
// ═══════════════════════════════════════════════════════════════════

struct SemanticChecker<'a> {
    symbols: &'a mut SymbolTable,
    errors: Vec<CompilerError>,
    /// Whether we are currently inside a class method (for `self` checks).
    in_method: bool,
    current_method: Option<String>,
}

impl<'a> SemanticChecker<'a> {
    // ── Top-level analysis entry points ─────────────────────────

    fn check_function(&mut self, func: &ast::FunctionDecl) {
        self.symbols.push_scope();

        // Define parameters.
        for p in &func.params {
            let t = match &p.type_ann {
                Some(ann) => {
                    self.check_type_exists(ann, p.span);
                    HulkType::from_name(ann)
                }
                None => HulkType::Unknown,
            };
            self.symbols.define_var(&p.name, t, p.span);
        }

        // Check return type annotation.
        if let Some(ref ret) = func.return_type {
            self.check_type_exists(ret, func.span);
        }

        // Analyse body.
        self.check_body(&func.body);

        let vars = self.symbols.pop_scope();
        self.warn_unused(&vars);
    }

    fn check_class(&mut self, class: &ast::ClassDecl) {
        self.symbols.current_class = Some(class.name.clone());

        // Check parent type annotation.
        if let Some(ref parent) = class.parent {
            self.check_type_exists(parent, class.span);
        }

        // Check constructor param type annotations.
        for p in &class.params {
            if let Some(ref ann) = p.type_ann {
                self.check_type_exists(ann, p.span);
            }
        }

        if !class.parent_args.is_empty() {
            if let Some(parent_name) = &class.parent {
                if let Some(parent_info) = self.symbols.get_class(parent_name) {
                    let expected = parent_info.params.len();
                    let got = class.parent_args.len();
                    if expected != got {
                        self.errors.push(CompilerError::wrong_arity(
                            parent_name,
                            expected,
                            got,
                            class.span,
                        ));
                    }
                }
            }

            self.symbols.push_scope();
            self.define_effective_constructor_params(&class.name, class.span);
            for arg in &class.parent_args {
                self.check_expression(arg);
            }
            self.symbols.pop_scope();
        }

        // Check attributes.
        for attr in &class.attributes {
            if let Some(ref ann) = attr.type_ann {
                self.check_type_exists(ann, attr.span);
            }
            // Attribute initializer runs in constructor scope with effective constructor params.
            self.symbols.push_scope();
            self.define_effective_constructor_params(&class.name, attr.span);
            self.in_method = true;
            self.check_expression(&attr.init);
            self.in_method = false;
            self.symbols.pop_scope();
        }

        // Check method bodies.
        for method in &class.methods {
            self.check_override_signature(class, method);
            self.check_method(method);
        }

        self.symbols.current_class = None;
    }

    fn check_method(&mut self, method: &ast::Method) {
        self.in_method = true;
        self.current_method = Some(method.name.clone());
        self.symbols.push_scope();

        // Define `self`.
        let self_type = match &self.symbols.current_class {
            Some(name) => HulkType::Class(name.clone()),
            None => HulkType::Error,
        };
        self.symbols.define_var("self", self_type, method.span);

        // Define parameters.
        for p in &method.params {
            let t = match &p.type_ann {
                Some(ann) => {
                    self.check_type_exists(ann, p.span);
                    HulkType::from_name(ann)
                }
                None => HulkType::Unknown,
            };
            self.symbols.define_var(&p.name, t, p.span);
        }

        if let Some(ref ret) = method.return_type {
            self.check_type_exists(ret, method.span);
        }

        self.check_body(&method.body);

        let vars = self.symbols.pop_scope();
        self.warn_unused(&vars);
        self.in_method = false;
        self.current_method = None;
    }

    // ── Body / expression body helpers ──────────────────────────

    fn check_body(&mut self, body: &ast::Body) {
        match body {
            ast::Body::Inline(expr) => self.check_expression(expr),
            ast::Body::Block(exprs) => {
                for expr in exprs {
                    self.check_expression(expr);
                }
            }
        }
    }

    fn check_expr_body(&mut self, body: &ast::ExprBody) {
        match body {
            ast::ExprBody::Single(expr) => self.check_expression(expr),
            ast::ExprBody::Block(exprs) => {
                for expr in exprs {
                    self.check_expression(expr);
                }
            }
        }
    }

    // ── Expression analysis ─────────────────────────────────────

    fn check_expression(&mut self, expr: &ast::Expression) {
        match expr {
            ast::Expression::Atom(atom) => self.check_atom(atom),
            ast::Expression::BinaryOp(binop) => {
                self.check_expression(&binop.left);
                self.check_expression(&binop.right);
            }
            ast::Expression::UnaryOp(unary) => {
                self.check_expression(&unary.expr);
            }
            ast::Expression::Let(let_expr) => self.check_let(let_expr),
            ast::Expression::If(if_expr) => self.check_if(if_expr),
            ast::Expression::While(while_expr) => self.check_while(while_expr),
            ast::Expression::For(for_expr) => self.check_for(for_expr),
            ast::Expression::IsType(is_expr) => self.check_is(is_expr),
            ast::Expression::AsType(as_expr) => self.check_as(as_expr),
            ast::Expression::Case(case_expr) => self.check_case(case_expr),
            ast::Expression::Assign(assign) => self.check_assign(assign),
            ast::Expression::FunctionCall(call) => self.check_function_call(call),
            ast::Expression::MemberAccess(access) => {
                self.check_expression(&access.object);
                // Member existence is checked during type checking.
            }
            ast::Expression::MethodCall(call) => {
                self.check_expression(&call.object);
                for arg in &call.args {
                    self.check_expression(arg);
                }
                // Method existence and arity checked during type checking.
            }
            ast::Expression::IndexAccess(access) => {
                self.check_expression(&access.object);
                self.check_expression(&access.index);
            }
            ast::Expression::NewInstance(inst) => self.check_new_instance(inst),
            ast::Expression::NewArray(arr) => self.check_new_array(arr),
        }
    }

    fn check_atom(&mut self, atom: &ast::atoms::atom::Atom) {
        match atom {
            ast::atoms::atom::Atom::Variable(id) => {
                if id.name == "self" {
                    if !self.in_method {
                        self.errors
                            .push(CompilerError::self_outside_class(id.position));
                    }
                } else if self.symbols.lookup_var(&id.name).is_none() {
                    self.errors
                        .push(CompilerError::undefined_variable(&id.name, id.position));
                } else {
                    self.symbols.mark_var_used(&id.name);
                }
            }
            ast::atoms::atom::Atom::Group(group) => {
                self.check_expression(&group.expression);
            }
            _ => {} // literals are always valid
        }
    }

    fn check_let(&mut self, let_expr: &ast::LetExpr) {
        // Each `let` declaration goes into a new scope.
        self.symbols.push_scope();

        for decl in &let_expr.decls {
            // Check type annotation.
            if let Some(ref ann) = decl.type_ann {
                self.check_type_exists(ann, decl.span);
            }
            // Check initializer (in the CURRENT scope, before defining the var).
            self.check_expression(&decl.value);

            let t = match &decl.type_ann {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Unknown,
            };
            if !self.symbols.define_var(&decl.name, t, decl.span) {
                self.errors.push(CompilerError::duplicate_definition(
                    "Variable",
                    &decl.name,
                    decl.span,
                ));
            }
        }

        // Check body (can use all declared variables).
        self.check_expr_body(&let_expr.body);

        let vars = self.symbols.pop_scope();
        self.warn_unused(&vars);
    }

    fn check_if(&mut self, if_expr: &ast::IfExpr) {
        self.check_expression(&if_expr.condition);
        self.check_expr_body(&if_expr.then_body);

        for branch in &if_expr.elif_branches {
            self.check_expression(&branch.condition);
            self.check_expr_body(&branch.body);
        }

        if let Some(ref else_body) = if_expr.else_body {
            self.check_expr_body(else_body);
        }
    }

    fn check_while(&mut self, while_expr: &ast::WhileExpr) {
        self.check_expression(&while_expr.condition);
        self.check_expr_body(&while_expr.body);
    }

    fn check_for(&mut self, for_expr: &ast::ForExpr) {
        self.check_expression(&for_expr.iterable);

        self.symbols.push_scope();
        self.symbols
            .define_var(&for_expr.var, HulkType::Unknown, for_expr.span);
        self.check_expr_body(&for_expr.body);
        let vars = self.symbols.pop_scope();
        self.warn_unused(&vars);
    }

    fn check_is(&mut self, is_expr: &ast::IsExpr) {
        self.check_expression(&is_expr.expr);
        self.check_type_exists(&is_expr.type_name, is_expr.span);
    }

    fn check_as(&mut self, as_expr: &ast::AsExpr) {
        self.check_expression(&as_expr.expr);
        self.check_type_exists(&as_expr.type_name, as_expr.span);
    }

    fn check_case(&mut self, case_expr: &ast::CaseExpr) {
        self.check_expression(&case_expr.expr);
        for branch in &case_expr.branches {
            // Check that the branch type exists.
            self.check_type_exists(&branch.type_name, branch.span);

            // Each branch introduces a new scope with the bound variable.
            self.symbols.push_scope();
            let t = HulkType::from_name(&branch.type_name);
            self.symbols.define_var(&branch.name, t, branch.span);
            self.check_expr_body(&branch.body);
            let vars = self.symbols.pop_scope();
            self.warn_unused(&vars);
        }
    }

    fn check_assign(&mut self, assign: &ast::AssignExpr) {
        // Validate target is assignable.
        match &assign.target {
            ast::Expression::Atom(atom) => match atom.as_ref() {
                ast::atoms::atom::Atom::Variable(id) => {
                    if self.symbols.lookup_var(&id.name).is_none() {
                        self.errors
                            .push(CompilerError::undefined_variable(&id.name, id.position));
                    } else {
                        self.symbols.mark_var_used(&id.name);
                    }
                }
                _ => {
                    self.errors
                        .push(CompilerError::invalid_assign_target(assign.span));
                }
            },
            ast::Expression::MemberAccess(_) | ast::Expression::IndexAccess(_) => {
                // Valid targets — check sub-expressions.
                self.check_expression(&assign.target);
            }
            _ => {
                self.errors
                    .push(CompilerError::invalid_assign_target(assign.span));
            }
        }

        self.check_expression(&assign.value);
    }

    fn check_function_call(&mut self, call: &ast::FunctionCall) {
        if call.name == "base" {
            self.check_base_call(call);
            return;
        }

        // Check function exists.
        match self.symbols.get_function(&call.name) {
            Some(func_info) => {
                let expected = func_info.params.len();
                let got = call.args.len();
                if expected != got {
                    self.errors.push(CompilerError::wrong_arity(
                        &call.name,
                        expected,
                        got,
                        call.span,
                    ));
                }
            }
            None => {
                self.errors
                    .push(CompilerError::undefined_function(&call.name, call.span));
            }
        }

        // Check arguments.
        for arg in &call.args {
            self.check_expression(arg);
        }
    }

    fn check_base_call(&mut self, call: &ast::FunctionCall) {
        if !self.in_method {
            self.errors.push(CompilerError::base_outside_method(call.span));
            for arg in &call.args {
                self.check_expression(arg);
            }
            return;
        }

        let class_name = match self.symbols.current_class.clone() {
            Some(name) => name,
            None => {
                self.errors.push(CompilerError::base_outside_method(call.span));
                return;
            }
        };
        let method_name = self.current_method.clone().unwrap_or_default();

        match self
            .symbols
            .resolve_parent_method(&class_name, &method_name)
        {
            Some((_owner, method_info)) => {
                let expected = method_info.params.len();
                let got = call.args.len();
                if expected != got {
                    self.errors.push(CompilerError::wrong_arity(
                        "base",
                        expected,
                        got,
                        call.span,
                    ));
                }
            }
            None => {
                self.errors.push(CompilerError::undefined_base_method(
                    &class_name,
                    &method_name,
                    call.span,
                ));
            }
        }

        for arg in &call.args {
            self.check_expression(arg);
        }
    }

    fn check_new_instance(&mut self, inst: &ast::NewInstance) {
        // Check type exists.
        if !self.symbols.type_exists(&inst.type_name) {
            self.errors
                .push(CompilerError::undefined_type(&inst.type_name, inst.span));
        } else if let Some(class) = self.symbols.get_class(&inst.type_name) {
            let expected = class.params.len();
            let got = inst.args.len();
            if expected != got {
                self.errors.push(CompilerError::wrong_arity(
                    &inst.type_name,
                    expected,
                    got,
                    inst.span,
                ));
            }
        }

        for arg in &inst.args {
            self.check_expression(arg);
        }
    }

    fn check_new_array(&mut self, arr: &ast::NewArray) {
        if let Some(ref type_name) = arr.type_name {
            self.check_type_exists(type_name, arr.span);
        }
        self.check_expression(&arr.size);
        if let Some((ref _var, ref init)) = arr.init {
            self.check_expression(init);
        }
    }

    // ── Helpers ─────────────────────────────────────────────────

    fn check_type_exists(&mut self, name: &str, span: crate::tokens::Span) {
        if !self.symbols.type_exists(name) {
            self.errors.push(CompilerError::undefined_type(name, span));
        }
    }

    fn define_effective_constructor_params(&mut self, class_name: &str, span: crate::tokens::Span) {
        if let Some(class_info) = self.symbols.get_class(class_name).cloned() {
            for (name, ty) in class_info.params {
                self.symbols.define_var(&name, ty, span);
            }
        }
    }

    fn check_override_signature(&mut self, class: &ast::ClassDecl, method: &ast::Method) {
        let parent_method = self
            .symbols
            .resolve_parent_method(&class.name, &method.name)
            .map(|(_, m)| m);

        let current_method = self
            .symbols
            .get_class(&class.name)
            .and_then(|info| info.get_method(&method.name))
            .cloned();

        if let (Some(parent), Some(current)) = (parent_method, current_method) {
            let same_len = parent.params.len() == current.params.len();
            let same_types = same_len
                && parent
                    .params
                    .iter()
                    .zip(current.params.iter())
                    .all(|((_, pt), (_, ct))| pt == ct);
            let same_ret = parent.return_type == current.return_type;

            if !(same_types && same_ret) {
                self.errors.push(CompilerError::invalid_override_signature(
                    &class.name,
                    &method.name,
                    method.span,
                ));
            }
        }
    }

    fn warn_unused(&mut self, vars: &[super::symbol_table::VarInfo]) {
        for var in vars {
            if !var.used && var.name != "self" {
                self.errors
                    .push(CompilerError::unused_variable(&var.name, var.span));
            }
        }
    }
}
