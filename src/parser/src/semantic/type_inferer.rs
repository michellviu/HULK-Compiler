//! Pass 2.5: Type inference.
//!
//! Runs after semantic checking and before type checking.
//! Infers types for symbols that lack explicit type annotations:
//!
//! - **Let variables**: type = type of initialization expression
//! - **Attributes**: type = type of initialization expression
//! - **Function/method arguments**: lowest type consistent with body usage
//! - **Constructor arguments**: lowest type consistent with attribute initializers
//!
//! The inferer collects *constraints* from how each Unknown-typed symbol is
//! used (operators, function arguments, etc.) and picks the most specific
//! type that satisfies all constraints, or leaves it Unknown if ambiguous.

use crate::ast;
use crate::tokens;

use super::errors::CompilerError;
use super::symbol_table::SymbolTable;
use super::types::HulkType;

/// Runs the type inference pass on the full program.
/// Mutates the symbol table to fill in inferred types for parameters,
/// attributes, and return types.
pub fn infer_types(
    program: &ast::Program,
    symbols: &mut SymbolTable,
) -> Vec<CompilerError> {
    let mut inferer = TypeInferer {
        symbols,
        errors: Vec::new(),
        constraints: Vec::new(),
        current_class: None,
    };

    // Phase 1: Infer function argument types from body usage.
    for func in &program.functions {
        inferer.infer_function_params(func);
    }

    // Phase 2: Infer class constructor params and attribute types.
    for class in &program.classes {
        inferer.infer_class(class);
    }

    inferer.errors
}

// ═══════════════════════════════════════════════════════════════════
// Inferer state
// ═══════════════════════════════════════════════════════════════════

struct TypeInferer<'a> {
    symbols: &'a mut SymbolTable,
    errors: Vec<CompilerError>,
    /// Temporary storage for constraints on parameters being inferred.
    constraints: Vec<(String, HulkType)>,
    current_class: Option<String>,
}

impl<'a> TypeInferer<'a> {
    // ── Function parameter inference ────────────────────────────

    fn infer_function_params(&mut self, func: &ast::FunctionDecl) {
        // Collect which params need inference.
        let unknown_params: Vec<String> = func
            .params
            .iter()
            .filter(|p| p.type_ann.is_none())
            .map(|p| p.name.clone())
            .collect();

        if !unknown_params.is_empty() {
            // Collect constraints from the body.
            self.constraints.clear();
            self.collect_constraints_body(&func.body, &unknown_params);

            // Resolve each unknown param.
            for param_name in &unknown_params {
                let inferred = self.resolve_constraints(param_name);
                if let Some(ty) = inferred {
                    if let Some(func_info) = self.symbols.functions.get_mut(&func.name) {
                        for (pname, ptype) in func_info.params.iter_mut() {
                            if pname == param_name && *ptype == HulkType::Unknown {
                                *ptype = ty.clone();
                            }
                        }
                    }
                }
            }
        }

        // Also infer the return type if not annotated.
        if func.return_type.is_none() {
            let body_type = self.infer_body_type(&func.body);
            if body_type.is_resolved() {
                if let Some(func_info) = self.symbols.functions.get_mut(&func.name) {
                    if !func_info.return_type.is_resolved() {
                        func_info.return_type = body_type;
                    }
                }
            }
        }
    }

    // ── Class inference ─────────────────────────────────────────

    fn infer_class(&mut self, class: &ast::ClassDecl) {
        self.current_class = Some(class.name.clone());

        // 1) Infer constructor param types from attribute initializers.
        let unknown_ctor_params: Vec<String> = class
            .params
            .iter()
            .filter(|p| p.type_ann.is_none())
            .map(|p| p.name.clone())
            .collect();

        if !unknown_ctor_params.is_empty() {
            self.constraints.clear();
            for attr in &class.attributes {
                self.collect_constraints_expr(&attr.init, &unknown_ctor_params);
            }
            // Also collect from parent args.
            for arg in &class.parent_args {
                self.collect_constraints_expr(arg, &unknown_ctor_params);
            }

            for param_name in &unknown_ctor_params {
                let inferred = self.resolve_constraints(param_name);
                if let Some(ty) = inferred {
                    if let Some(class_info) = self.symbols.classes.get_mut(&class.name) {
                        for (pname, ptype) in class_info.params.iter_mut() {
                            if pname == param_name && *ptype == HulkType::Unknown {
                                *ptype = ty.clone();
                            }
                        }
                    }
                }
            }
        }

        // 2) Infer attribute types from initializer expressions.
        for attr in &class.attributes {
            if attr.type_ann.is_none() {
                let inferred = self.infer_expr_type(&attr.init);
                if inferred.is_resolved() {
                    if let Some(class_info) = self.symbols.classes.get_mut(&class.name) {
                        if let Some(attr_info) = class_info
                            .attributes
                            .iter_mut()
                            .find(|a| a.name == attr.name)
                        {
                            if attr_info.hulk_type == HulkType::Unknown {
                                attr_info.hulk_type = inferred;
                            }
                        }
                    }
                }
            }
        }

        // 3) Infer method parameter types from body usage.
        for method in &class.methods {
            self.infer_method_params(&class.name, method);
        }

        self.current_class = None;
    }

    fn infer_method_params(&mut self, class_name: &str, method: &ast::Method) {
        let unknown_params: Vec<String> = method
            .params
            .iter()
            .filter(|p| p.type_ann.is_none())
            .map(|p| p.name.clone())
            .collect();

        if unknown_params.is_empty() {
            return;
        }

        self.constraints.clear();
        self.collect_constraints_body(&method.body, &unknown_params);

        for param_name in &unknown_params {
            let inferred = self.resolve_constraints(param_name);
            if let Some(ty) = inferred {
                if let Some(class_info) = self.symbols.classes.get_mut(class_name) {
                    if let Some(method_info) = class_info.methods.get_mut(&method.name) {
                        for (pname, ptype) in method_info.params.iter_mut() {
                            if pname == param_name && *ptype == HulkType::Unknown {
                                *ptype = ty.clone();
                            }
                        }
                    }
                }
            }
        }
    }

    // ── Constraint collection ───────────────────────────────────
    //
    // Walk the AST and when we see a tracked variable used in a
    // context that constrains its type, record (var_name, type).

    fn collect_constraints_body(&mut self, body: &ast::Body, targets: &[String]) {
        match body {
            ast::Body::Inline(expr) => self.collect_constraints_expr(expr, targets),
            ast::Body::Block(exprs) => {
                for e in exprs {
                    self.collect_constraints_expr(e, targets);
                }
            }
        }
    }

    fn collect_constraints_expr_body(&mut self, body: &ast::ExprBody, targets: &[String]) {
        match body {
            ast::ExprBody::Single(expr) => self.collect_constraints_expr(expr, targets),
            ast::ExprBody::Block(exprs) => {
                for e in exprs {
                    self.collect_constraints_expr(e, targets);
                }
            }
        }
    }

    fn collect_constraints_expr(&mut self, expr: &ast::Expression, targets: &[String]) {
        match expr {
            ast::Expression::BinaryOp(binop) => {
                self.collect_binary_op_constraints(binop, targets);
            }
            ast::Expression::UnaryOp(unary) => {
                self.collect_unary_op_constraints(unary, targets);
            }
            ast::Expression::FunctionCall(call) => {
                self.collect_function_call_constraints(call, targets);
            }
            ast::Expression::MethodCall(call) => {
                // Recurse into object and args.
                self.collect_constraints_expr(&call.object, targets);
                for arg in &call.args {
                    self.collect_constraints_expr(arg, targets);
                }
            }
            ast::Expression::Let(let_expr) => {
                for decl in &let_expr.decls {
                    self.collect_constraints_expr(&decl.value, targets);
                }
                self.collect_constraints_expr_body(&let_expr.body, targets);
            }
            ast::Expression::If(if_expr) => {
                self.collect_constraints_expr(&if_expr.condition, targets);
                self.collect_constraints_expr_body(&if_expr.then_body, targets);
                for branch in &if_expr.elif_branches {
                    self.collect_constraints_expr(&branch.condition, targets);
                    self.collect_constraints_expr_body(&branch.body, targets);
                }
                if let Some(ref else_body) = if_expr.else_body {
                    self.collect_constraints_expr_body(else_body, targets);
                }
            }
            ast::Expression::While(while_expr) => {
                self.collect_constraints_expr(&while_expr.condition, targets);
                self.collect_constraints_expr_body(&while_expr.body, targets);
            }
            ast::Expression::For(for_expr) => {
                self.collect_constraints_expr(&for_expr.iterable, targets);
                self.collect_constraints_expr_body(&for_expr.body, targets);
            }
            ast::Expression::Assign(assign) => {
                self.collect_constraints_expr(&assign.target, targets);
                self.collect_constraints_expr(&assign.value, targets);
            }
            ast::Expression::Atom(atom) => {
                // Atoms by themselves don't generate constraints.
                if let ast::atoms::atom::Atom::Group(g) = atom.as_ref() {
                    self.collect_constraints_expr(&g.expression, targets);
                }
            }
            ast::Expression::IsType(is_expr) => {
                self.collect_constraints_expr(&is_expr.expr, targets);
            }
            ast::Expression::AsType(as_expr) => {
                self.collect_constraints_expr(&as_expr.expr, targets);
            }
            ast::Expression::Case(case_expr) => {
                self.collect_constraints_expr(&case_expr.expr, targets);
                for branch in &case_expr.branches {
                    self.collect_constraints_expr_body(&branch.body, targets);
                }
            }
            ast::Expression::MemberAccess(access) => {
                self.collect_constraints_expr(&access.object, targets);
            }
            ast::Expression::IndexAccess(access) => {
                self.collect_constraints_expr(&access.object, targets);
                // Index must be Number — if the index is a target var, constrain it.
                if let Some(name) = self.extract_var_name(&access.index) {
                    if targets.contains(&name) {
                        self.constraints.push((name, HulkType::Number));
                    }
                }
                self.collect_constraints_expr(&access.index, targets);
            }
            ast::Expression::NewInstance(inst) => {
                // Check if args match constructor param types.
                let ctor_params: Vec<(String, HulkType)> = self
                    .symbols
                    .get_class(&inst.type_name)
                    .map(|c| c.params.clone())
                    .unwrap_or_default();
                for (i, arg) in inst.args.iter().enumerate() {
                    if let Some(name) = self.extract_var_name(arg) {
                        if targets.contains(&name) {
                            if let Some((_, ptype)) = ctor_params.get(i) {
                                if ptype.is_resolved() {
                                    self.constraints.push((name, ptype.clone()));
                                }
                            }
                        }
                    }
                    self.collect_constraints_expr(arg, targets);
                }
            }
            ast::Expression::NewArray(arr) => {
                self.collect_constraints_expr(&arr.size, targets);
                if let Some((_, ref init)) = arr.init {
                    self.collect_constraints_expr(init, targets);
                }
            }
        }
    }

    fn collect_binary_op_constraints(
        &mut self,
        binop: &ast::expressions::binoperation::BinaryOp,
        targets: &[String],
    ) {
        let left_name = self.extract_var_name(&binop.left);
        let right_name = self.extract_var_name(&binop.right);

        match &binop.operator {
            // Arithmetic operators require Number operands.
            tokens::BinOp::Plus(_)
            | tokens::BinOp::Minus(_)
            | tokens::BinOp::Mul(_)
            | tokens::BinOp::Div(_)
            | tokens::BinOp::Mod(_)
            | tokens::BinOp::Pow(_) => {
                if let Some(ref name) = left_name {
                    if targets.contains(name) {
                        self.constraints.push((name.clone(), HulkType::Number));
                    }
                }
                if let Some(ref name) = right_name {
                    if targets.contains(name) {
                        self.constraints.push((name.clone(), HulkType::Number));
                    }
                }
            }
            // Comparison operators require Number.
            tokens::BinOp::Less(_)
            | tokens::BinOp::LessEqual(_)
            | tokens::BinOp::Greater(_)
            | tokens::BinOp::GreaterEqual(_) => {
                if let Some(ref name) = left_name {
                    if targets.contains(name) {
                        self.constraints.push((name.clone(), HulkType::Number));
                    }
                }
                if let Some(ref name) = right_name {
                    if targets.contains(name) {
                        self.constraints.push((name.clone(), HulkType::Number));
                    }
                }
            }
            // Logical operators require Boolean.
            tokens::BinOp::And(_) | tokens::BinOp::Or(_) => {
                if let Some(ref name) = left_name {
                    if targets.contains(name) {
                        self.constraints.push((name.clone(), HulkType::Boolean));
                    }
                }
                if let Some(ref name) = right_name {
                    if targets.contains(name) {
                        self.constraints.push((name.clone(), HulkType::Boolean));
                    }
                }
            }
            // Equality — no type constraint (any types can be compared).
            tokens::BinOp::EqualEqual(_) | tokens::BinOp::NotEqual(_) => {}
            // Concatenation — no constraint (any type can be concat'd).
            tokens::BinOp::Concat(_) | tokens::BinOp::ConcatSpaced(_) => {}
            _ => {}
        }

        // Recurse into sub-expressions.
        self.collect_constraints_expr(&binop.left, targets);
        self.collect_constraints_expr(&binop.right, targets);
    }

    fn collect_unary_op_constraints(
        &mut self,
        unary: &ast::expressions::unaryoperation::UnaryOp,
        targets: &[String],
    ) {
        if let Some(ref name) = self.extract_var_name(&unary.expr) {
            if targets.contains(name) {
                match &unary.op {
                    tokens::UnaryOp::Minus(_) => {
                        self.constraints.push((name.clone(), HulkType::Number));
                    }
                    tokens::UnaryOp::Not(_) => {
                        self.constraints.push((name.clone(), HulkType::Boolean));
                    }
                }
            }
        }
        self.collect_constraints_expr(&unary.expr, targets);
    }

    fn collect_function_call_constraints(
        &mut self,
        call: &ast::FunctionCall,
        targets: &[String],
    ) {
        // If an argument is a target variable and the function's param type
        // is known, constrain the variable to that type.
        let param_types: Vec<(String, HulkType)> = self
            .symbols
            .get_function(&call.name)
            .map(|f| f.params.clone())
            .unwrap_or_default();

        for (i, arg) in call.args.iter().enumerate() {
            if let Some(name) = self.extract_var_name(arg) {
                if targets.contains(&name) {
                    if let Some((_, ptype)) = param_types.get(i) {
                        if ptype.is_resolved() {
                            self.constraints.push((name, ptype.clone()));
                        }
                    }
                }
            }
            self.collect_constraints_expr(arg, targets);
        }

        // Also handle recursive calls: if the function being called is the
        // same function we're analyzing, the arg positions give self-constraints.
        // These are handled by the operator constraints already in the body.
    }

    // ── Constraint resolution ───────────────────────────────────

    /// Resolves all collected constraints for a given variable name.
    /// Returns `Some(type)` if all constraints agree, `None` if conflicting.
    fn resolve_constraints(&self, name: &str) -> Option<HulkType> {
        let relevant: Vec<&HulkType> = self
            .constraints
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, t)| t)
            .collect();

        if relevant.is_empty() {
            return None;
        }

        // All constraints must agree on the same type, or we find the
        // most specific type using LCA.
        let first = relevant[0].clone();
        let mut result = first;

        for ty in &relevant[1..] {
            if **ty == result {
                continue;
            }
            // Try LCA — if both are class types in the same hierarchy.
            let lca = self.symbols.lca(&result, ty);
            if lca == HulkType::Object {
                // Conflicting constraints from different hierarchies — fail.
                // But if both constraints are the same primitive, it's fine.
                // Object is too general — we can't infer a specific type.
                return None;
            }
            result = lca;
        }

        Some(result)
    }

    // ── Expression type inference (lightweight) ─────────────────
    //
    // A simplified version of the type checker's inference, used to
    // infer types for attribute initializers before the full type
    // checker runs.

    fn infer_expr_type(&self, expr: &ast::Expression) -> HulkType {
        match expr {
            ast::Expression::Atom(atom) => self.infer_atom_type(atom),
            ast::Expression::BinaryOp(binop) => self.infer_binop_type(binop),
            ast::Expression::UnaryOp(unary) => self.infer_unary_type(unary),
            ast::Expression::FunctionCall(call) => {
                self.symbols
                    .get_function(&call.name)
                    .map(|f| f.return_type.clone())
                    .unwrap_or(HulkType::Unknown)
            }
            ast::Expression::NewInstance(inst) => HulkType::Class(inst.type_name.clone()),
            ast::Expression::Let(let_expr) => self.infer_expr_body_type(&let_expr.body),
            ast::Expression::If(if_expr) => {
                let then_ty = self.infer_expr_body_type(&if_expr.then_body);
                let mut result = then_ty;
                for branch in &if_expr.elif_branches {
                    let bt = self.infer_expr_body_type(&branch.body);
                    result = self.symbols.lca(&result, &bt);
                }
                if let Some(ref else_body) = if_expr.else_body {
                    let et = self.infer_expr_body_type(else_body);
                    result = self.symbols.lca(&result, &et);
                }
                result
            }
            ast::Expression::While(w) => self.infer_expr_body_type(&w.body),
            ast::Expression::For(f) => self.infer_expr_body_type(&f.body),
            ast::Expression::IsType(_) => HulkType::Boolean,
            ast::Expression::AsType(as_expr) => HulkType::from_name(&as_expr.type_name),
            ast::Expression::Case(case_expr) => {
                let mut result = HulkType::Error;
                for branch in &case_expr.branches {
                    let bt = self.infer_expr_body_type(&branch.body);
                    if result.is_error() {
                        result = bt;
                    } else {
                        result = self.symbols.lca(&result, &bt);
                    }
                }
                result
            }
            ast::Expression::Assign(assign) => self.infer_expr_type(&assign.value),
            ast::Expression::MemberAccess(access) => {
                let obj_type = self.infer_expr_type(&access.object);
                if let Some(class_name) = self.type_to_class_name(&obj_type) {
                    self.symbols
                        .resolve_attribute(&class_name, &access.member)
                        .map(|(_, a)| a.hulk_type.clone())
                        .unwrap_or(HulkType::Unknown)
                } else {
                    HulkType::Unknown
                }
            }
            ast::Expression::MethodCall(call) => {
                let obj_type = self.infer_expr_type(&call.object);
                if let Some(class_name) = self.type_to_class_name(&obj_type) {
                    self.symbols
                        .resolve_method(&class_name, &call.method)
                        .map(|(_, m)| m.return_type.clone())
                        .unwrap_or(HulkType::Unknown)
                } else {
                    HulkType::Unknown
                }
            }
            ast::Expression::IndexAccess(_) => HulkType::Unknown,
            ast::Expression::NewArray(arr) => {
                let elem = match &arr.type_name {
                    Some(name) => HulkType::from_name(name),
                    None => HulkType::Unknown,
                };
                HulkType::Array(Box::new(elem))
            }
        }
    }

    fn infer_atom_type(&self, atom: &ast::atoms::atom::Atom) -> HulkType {
        match atom {
            ast::atoms::atom::Atom::NumberLiteral(_) => HulkType::Number,
            ast::atoms::atom::Atom::StringLiteral(_) => HulkType::String,
            ast::atoms::atom::Atom::BooleanLiteral(_) => HulkType::Boolean,
            ast::atoms::atom::Atom::Variable(id) => {
                if id.name == "self" {
                    self.current_class
                        .as_ref()
                        .map(|n| HulkType::Class(n.clone()))
                        .unwrap_or(HulkType::Unknown)
                } else {
                    // Check constructor params in current class.
                    if let Some(ref class_name) = self.current_class {
                        if let Some(class_info) = self.symbols.get_class(class_name) {
                            for (pname, ptype) in &class_info.params {
                                if pname == &id.name && ptype.is_resolved() {
                                    return ptype.clone();
                                }
                            }
                        }
                    }
                    HulkType::Unknown
                }
            }
            ast::atoms::atom::Atom::Group(g) => self.infer_expr_type(&g.expression),
        }
    }

    fn infer_binop_type(&self, binop: &ast::expressions::binoperation::BinaryOp) -> HulkType {
        match &binop.operator {
            tokens::BinOp::Plus(_)
            | tokens::BinOp::Minus(_)
            | tokens::BinOp::Mul(_)
            | tokens::BinOp::Div(_)
            | tokens::BinOp::Mod(_)
            | tokens::BinOp::Pow(_) => HulkType::Number,
            tokens::BinOp::Less(_)
            | tokens::BinOp::LessEqual(_)
            | tokens::BinOp::Greater(_)
            | tokens::BinOp::GreaterEqual(_)
            | tokens::BinOp::EqualEqual(_)
            | tokens::BinOp::NotEqual(_)
            | tokens::BinOp::And(_)
            | tokens::BinOp::Or(_) => HulkType::Boolean,
            tokens::BinOp::Concat(_) | tokens::BinOp::ConcatSpaced(_) => HulkType::String,
            _ => HulkType::Unknown,
        }
    }

    fn infer_unary_type(&self, unary: &ast::expressions::unaryoperation::UnaryOp) -> HulkType {
        match &unary.op {
            tokens::UnaryOp::Minus(_) => HulkType::Number,
            tokens::UnaryOp::Not(_) => HulkType::Boolean,
        }
    }

    fn infer_body_type(&self, body: &ast::Body) -> HulkType {
        match body {
            ast::Body::Inline(expr) => self.infer_expr_type(expr),
            ast::Body::Block(exprs) => {
                exprs.last().map(|e| self.infer_expr_type(e)).unwrap_or(HulkType::Void)
            }
        }
    }

    fn infer_expr_body_type(&self, body: &ast::ExprBody) -> HulkType {
        match body {
            ast::ExprBody::Single(expr) => self.infer_expr_type(expr),
            ast::ExprBody::Block(exprs) => {
                exprs.last().map(|e| self.infer_expr_type(e)).unwrap_or(HulkType::Void)
            }
        }
    }

    // ── Helpers ─────────────────────────────────────────────────

    /// Extracts the variable name if the expression is a simple variable reference.
    fn extract_var_name(&self, expr: &ast::Expression) -> Option<String> {
        match expr {
            ast::Expression::Atom(atom) => match atom.as_ref() {
                ast::atoms::atom::Atom::Variable(id) => Some(id.name.clone()),
                ast::atoms::atom::Atom::Group(g) => self.extract_var_name(&g.expression),
                _ => None,
            },
            _ => None,
        }
    }

    fn type_to_class_name(&self, ty: &HulkType) -> Option<String> {
        match ty {
            HulkType::Class(name) => Some(name.clone()),
            HulkType::Number => Some("Number".into()),
            HulkType::String => Some("String".into()),
            HulkType::Boolean => Some("Boolean".into()),
            HulkType::Object => Some("Object".into()),
            _ => None,
        }
    }
}
