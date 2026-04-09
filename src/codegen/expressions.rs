//! Code generation for HULK expressions.
//!
//! Every `gen_*` method returns `HulkValue<'ctx>` — `Some(val)` for
//! expressions that produce a value, `None` for void expressions.

use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum};
use inkwell::FloatPredicate;

use parser::ast;
use parser::tokens;
use parser::semantic::types::HulkType;

use super::context::{CodegenContext, HulkValue};

impl<'ctx> CodegenContext<'ctx> {
    // ── Main dispatch ────────────────────────────────────────────

    pub fn gen_expression(&mut self, expr: &ast::Expression) -> HulkValue<'ctx> {
        match expr {
            ast::Expression::Atom(atom) => self.gen_atom(atom),
            ast::Expression::BinaryOp(binop) => self.gen_binary_op(binop),
            ast::Expression::UnaryOp(unary) => self.gen_unary_op(unary),
            ast::Expression::Let(let_expr) => self.gen_let(let_expr),
            ast::Expression::If(if_expr) => self.gen_if(if_expr),
            ast::Expression::While(while_expr) => self.gen_while(while_expr),
            ast::Expression::Case(_case_expr) => {
                // Case/pattern matching — simplified: evaluate first branch.
                // Full implementation would need runtime type tags.
                None
            }
            ast::Expression::Assign(assign) => self.gen_assign(assign),
            ast::Expression::FunctionCall(call) => self.gen_function_call(call),
            ast::Expression::MemberAccess(access) => self.gen_member_access(access),
            ast::Expression::MethodCall(call) => self.gen_method_call(call),
            ast::Expression::IndexAccess(_access) => {
                // TODO: array indexing
                None
            }
            ast::Expression::NewInstance(inst) => self.gen_new_instance(inst),
            ast::Expression::NewArray(_arr) => {
                // TODO: array allocation
                None
            }
        }
    }

    // ── Atoms ────────────────────────────────────────────────────

    fn gen_atom(&mut self, atom: &ast::atoms::atom::Atom) -> HulkValue<'ctx> {
        match atom {
            ast::atoms::atom::Atom::NumberLiteral(lit) => {
                if let tokens::Literal::Number(val, _) = lit {
                    Some(self.f64_type().const_float(*val).into())
                } else {
                    None
                }
            }
            ast::atoms::atom::Atom::BooleanLiteral(lit) => {
                if let tokens::Literal::Bool(val, _) = lit {
                    let i = if *val { 1 } else { 0 };
                    Some(self.bool_type().const_int(i, false).into())
                } else {
                    None
                }
            }
            ast::atoms::atom::Atom::StringLiteral(lit) => {
                if let tokens::Literal::Str(s, _) = lit {
                    Some(self.get_or_create_string(s).into())
                } else {
                    None
                }
            }
            ast::atoms::atom::Atom::Variable(id) => {
                if let Some((ptr, llvm_ty)) = self.get_variable(&id.name) {
                    let val = self.builder.build_load(llvm_ty, ptr, &id.name).unwrap();
                    Some(val)
                } else {
                    None
                }
            }
            ast::atoms::atom::Atom::Group(group) => {
                self.gen_expression(&group.expression)
            }
        }
    }

    // ── Binary operations ────────────────────────────────────────

    fn gen_binary_op(
        &mut self,
        binop: &ast::expressions::binoperation::BinaryOp,
    ) -> HulkValue<'ctx> {
        // String concatenation is special.
        match &binop.operator {
            tokens::BinOp::Concat(_) => return self.gen_concat(&binop.left, &binop.right, false),
            tokens::BinOp::ConcatSpaced(_) => {
                return self.gen_concat(&binop.left, &binop.right, true)
            }
            _ => {}
        }

        let lhs = self.gen_expression(&binop.left)?;
        let rhs = self.gen_expression(&binop.right)?;

        match &binop.operator {
            // ── Arithmetic ───────────────────────────────────────
            tokens::BinOp::Plus(_) => {
                let v = self
                    .builder
                    .build_float_add(lhs.into_float_value(), rhs.into_float_value(), "add")
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::Minus(_) => {
                let v = self
                    .builder
                    .build_float_sub(lhs.into_float_value(), rhs.into_float_value(), "sub")
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::Mul(_) => {
                let v = self
                    .builder
                    .build_float_mul(lhs.into_float_value(), rhs.into_float_value(), "mul")
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::Div(_) => {
                let v = self
                    .builder
                    .build_float_div(lhs.into_float_value(), rhs.into_float_value(), "div")
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::Mod(_) => {
                let v = self
                    .builder
                    .build_float_rem(lhs.into_float_value(), rhs.into_float_value(), "mod")
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::Pow(_) => {
                // Call runtime hulk_pow(base, exp).
                let pow_fn = *self.functions.get("__hulk_pow").unwrap();
                let result = self
                    .builder
                    .build_call(
                        pow_fn,
                        &[lhs.into(), rhs.into()],
                        "pow",
                    )
                    .unwrap()
                    .try_as_basic_value()
                    .left();
                result
            }

            // ── Comparison ───────────────────────────────────────
            tokens::BinOp::Less(_) => {
                let v = self
                    .builder
                    .build_float_compare(
                        FloatPredicate::OLT,
                        lhs.into_float_value(),
                        rhs.into_float_value(),
                        "lt",
                    )
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::LessEqual(_) => {
                let v = self
                    .builder
                    .build_float_compare(
                        FloatPredicate::OLE,
                        lhs.into_float_value(),
                        rhs.into_float_value(),
                        "le",
                    )
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::Greater(_) => {
                let v = self
                    .builder
                    .build_float_compare(
                        FloatPredicate::OGT,
                        lhs.into_float_value(),
                        rhs.into_float_value(),
                        "gt",
                    )
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::GreaterEqual(_) => {
                let v = self
                    .builder
                    .build_float_compare(
                        FloatPredicate::OGE,
                        lhs.into_float_value(),
                        rhs.into_float_value(),
                        "ge",
                    )
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::EqualEqual(_) => {
                // Generic equality — works on f64.
                let v = self
                    .builder
                    .build_float_compare(
                        FloatPredicate::OEQ,
                        lhs.into_float_value(),
                        rhs.into_float_value(),
                        "eq",
                    )
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::NotEqual(_) => {
                let v = self
                    .builder
                    .build_float_compare(
                        FloatPredicate::ONE,
                        lhs.into_float_value(),
                        rhs.into_float_value(),
                        "ne",
                    )
                    .unwrap();
                Some(v.into())
            }

            // ── Logical ──────────────────────────────────────────
            tokens::BinOp::And(_) => {
                let v = self
                    .builder
                    .build_and(lhs.into_int_value(), rhs.into_int_value(), "and")
                    .unwrap();
                Some(v.into())
            }
            tokens::BinOp::Or(_) => {
                let v = self
                    .builder
                    .build_or(lhs.into_int_value(), rhs.into_int_value(), "or")
                    .unwrap();
                Some(v.into())
            }

            // Concat handled above.
            tokens::BinOp::Concat(_) | tokens::BinOp::ConcatSpaced(_) => unreachable!(),
            // Assignment ops should not appear as binary ops in codegen.
            tokens::BinOp::Equal(_) | tokens::BinOp::Assign(_) => None,
        }
    }

    // ── Unary operations ─────────────────────────────────────────

    fn gen_unary_op(
        &mut self,
        unary: &ast::expressions::unaryoperation::UnaryOp,
    ) -> HulkValue<'ctx> {
        let operand = self.gen_expression(&unary.expr)?;

        match &unary.op {
            tokens::UnaryOp::Minus(_) => {
                let v = self
                    .builder
                    .build_float_neg(operand.into_float_value(), "neg")
                    .unwrap();
                Some(v.into())
            }
            tokens::UnaryOp::Not(_) => {
                let v = self
                    .builder
                    .build_not(operand.into_int_value(), "not")
                    .unwrap();
                Some(v.into())
            }
        }
    }

    // ── String concatenation ─────────────────────────────────────

    fn gen_concat(
        &mut self,
        left: &ast::Expression,
        right: &ast::Expression,
        spaced: bool,
    ) -> HulkValue<'ctx> {
        let left_str = self.expr_to_string(left);
        let right_str = self.expr_to_string(right);

        let concat_fn_name = if spaced {
            "__hulk_concat_spaced"
        } else {
            "__hulk_concat"
        };
        let concat_fn = *self.functions.get(concat_fn_name).unwrap();

        let result = self
            .builder
            .build_call(
                concat_fn,
                &[left_str.into(), right_str.into()],
                "concat",
            )
            .unwrap()
            .try_as_basic_value()
            .left();
        result
    }

    /// Converts an expression result to a string pointer (i8*),
    /// calling the appropriate conversion runtime function.
    fn expr_to_string(&mut self, expr: &ast::Expression) -> BasicValueEnum<'ctx> {
        // Determine the semantic type to know which conversion to use.
        let val = self.gen_expression(expr);

        match val {
            Some(v) => {
                if v.is_float_value() {
                    // Number → string.
                    let conv_fn = *self.functions.get("__hulk_number_to_string").unwrap();
                    self.builder
                        .build_call(conv_fn, &[v.into()], "num_str")
                        .unwrap()
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                } else if v.is_int_value() && v.into_int_value().get_type().get_bit_width() == 1 {
                    // Boolean → string.
                    let i32_val = self
                        .builder
                        .build_int_z_extend(
                            v.into_int_value(),
                            self.context.i32_type(),
                            "bool_ext",
                        )
                        .unwrap();
                    let conv_fn = *self.functions.get("__hulk_bool_to_string").unwrap();
                    self.builder
                        .build_call(conv_fn, &[i32_val.into()], "bool_str")
                        .unwrap()
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                } else {
                    // Already a pointer (string or object) — assume string.
                    v
                }
            }
            None => {
                // Void — return empty string.
                self.get_or_create_string("").into()
            }
        }
    }

    // ── Let expressions ──────────────────────────────────────────

    fn gen_let(&mut self, let_expr: &ast::LetExpr) -> HulkValue<'ctx> {
        self.push_scope();

        for decl in &let_expr.decls {
            let init_val = self.gen_expression(&decl.value);

            let ht = match &decl.type_ann {
                Some(ann) => HulkType::from_name(ann),
                None => {
                    // Infer from AST expression first, then fall back to LLVM value type.
                    self.infer_hulk_type_from_expr(&decl.value, &init_val)
                }
            };

            let llvm_ty = self.hulk_type_to_llvm(&ht);
            let func = self.current_function();
            let alloca = self.create_entry_block_alloca(func, &decl.name, llvm_ty);

            if let Some(val) = init_val {
                self.builder.build_store(alloca, val).unwrap();
            }

            // Make the variable visible in the symbol table for load inference.
            self.symbols.push_scope();
            self.symbols.define_var(
                &decl.name,
                ht,
                parser::tokens::Position::new(0, 0),
            );
            self.set_variable(&decl.name, alloca, llvm_ty);
        }

        let result = self.gen_expr_body(&let_expr.body);

        // Pop the scopes we pushed for each decl.
        for _ in &let_expr.decls {
            self.symbols.pop_scope();
        }
        self.pop_scope();

        result
    }

    // ── If expressions ───────────────────────────────────────────

    fn gen_if(&mut self, if_expr: &ast::IfExpr) -> HulkValue<'ctx> {
        let func = self.current_function();

        let cond_val = self
            .gen_expression(&if_expr.condition)
            .unwrap()
            .into_int_value();

        // How many total branches (then + elif + else)?
        let has_else = if_expr.else_body.is_some();
        let _branch_count = 1 + if_expr.elif_branches.len() + if has_else { 1 } else { 0 };

        let merge_bb = self.context.append_basic_block(func, "if.merge");

        // We'll accumulate (value, source_block) for the phi node.
        let mut incoming: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
            Vec::new();
        let mut all_void = true;

        // Then branch.
        let then_bb = self.context.append_basic_block(func, "if.then");
        let else_bb = if !if_expr.elif_branches.is_empty() || has_else {
            self.context.append_basic_block(func, "if.else")
        } else {
            merge_bb
        };

        self.builder
            .build_conditional_branch(cond_val, then_bb, else_bb)
            .unwrap();

        // Generate then-body.
        self.builder.position_at_end(then_bb);
        let then_val = self.gen_expr_body(&if_expr.then_body);
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            if let Some(v) = then_val {
                incoming.push((v, self.builder.get_insert_block().unwrap()));
                all_void = false;
            }
            self.builder.build_unconditional_branch(merge_bb).unwrap();
        }

        // Generate elif branches.
        let mut current_else_bb = else_bb;
        for (i, branch) in if_expr.elif_branches.iter().enumerate() {
            self.builder.position_at_end(current_else_bb);
            let cond = self
                .gen_expression(&branch.condition)
                .unwrap()
                .into_int_value();

            let elif_then_bb = self
                .context
                .append_basic_block(func, &format!("elif.then.{}", i));
            let next_else_bb = if i + 1 < if_expr.elif_branches.len() || has_else {
                self.context
                    .append_basic_block(func, &format!("elif.else.{}", i))
            } else {
                merge_bb
            };

            self.builder
                .build_conditional_branch(cond, elif_then_bb, next_else_bb)
                .unwrap();

            self.builder.position_at_end(elif_then_bb);
            let branch_val = self.gen_expr_body(&branch.body);
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                if let Some(v) = branch_val {
                    incoming.push((v, self.builder.get_insert_block().unwrap()));
                    all_void = false;
                }
                self.builder.build_unconditional_branch(merge_bb).unwrap();
            }

            current_else_bb = next_else_bb;
        }

        // Generate else branch.
        if let Some(ref else_body) = if_expr.else_body {
            self.builder.position_at_end(current_else_bb);
            let else_val = self.gen_expr_body(else_body);
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                if let Some(v) = else_val {
                    incoming.push((v, self.builder.get_insert_block().unwrap()));
                    all_void = false;
                }
                self.builder.build_unconditional_branch(merge_bb).unwrap();
            }
        }

        self.builder.position_at_end(merge_bb);

        if all_void || incoming.is_empty() {
            return None;
        }

        // Build PHI node.
        let first_type = incoming[0].0.get_type();
        let phi = self.builder.build_phi(first_type, "if.result").unwrap();

        let refs: Vec<(&dyn BasicValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
            incoming.iter().map(|(v, bb)| (v as &dyn BasicValue, *bb)).collect();
        phi.add_incoming(&refs);

        Some(phi.as_basic_value())
    }

    // ── While expression ─────────────────────────────────────────

    fn gen_while(&mut self, while_expr: &ast::WhileExpr) -> HulkValue<'ctx> {
        let func = self.current_function();

        let cond_bb = self.context.append_basic_block(func, "while.cond");
        let body_bb = self.context.append_basic_block(func, "while.body");
        let merge_bb = self.context.append_basic_block(func, "while.merge");

        self.builder.build_unconditional_branch(cond_bb).unwrap();

        // Condition.
        self.builder.position_at_end(cond_bb);
        let cond_val = self
            .gen_expression(&while_expr.condition)
            .unwrap()
            .into_int_value();

        let else_bb = if while_expr.else_body.is_some() {
            let bb = self.context.append_basic_block(func, "while.else");
            self.builder
                .build_conditional_branch(cond_val, body_bb, bb)
                .unwrap();
            Some(bb)
        } else {
            self.builder
                .build_conditional_branch(cond_val, body_bb, merge_bb)
                .unwrap();
            None
        };

        // Body.
        self.builder.position_at_end(body_bb);
        self.gen_expr_body(&while_expr.body);
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(cond_bb).unwrap();
        }

        // Else.
        if let (Some(else_body), Some(ebb)) = (&while_expr.else_body, else_bb) {
            self.builder.position_at_end(ebb);
            self.gen_expr_body(else_body);
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                self.builder.build_unconditional_branch(merge_bb).unwrap();
            }
        }

        self.builder.position_at_end(merge_bb);
        None
    }

    // ── Assignment ───────────────────────────────────────────────

    fn gen_assign(&mut self, assign: &ast::AssignExpr) -> HulkValue<'ctx> {
        let value = self.gen_expression(&assign.value);

        match &assign.target {
            ast::Expression::Atom(atom) => {
                if let ast::atoms::atom::Atom::Variable(id) = atom.as_ref() {
                    if let Some((ptr, _ty)) = self.get_variable(&id.name) {
                        if let Some(val) = value {
                            self.builder.build_store(ptr, val).unwrap();
                            return Some(val);
                        }
                    }
                }
            }
            ast::Expression::MemberAccess(access) => {
                // obj.field := value
                if let Some(val) = value {
                    let obj = self.gen_expression(&access.object)?;
                    let obj_ptr = obj.into_pointer_value();

                    // Find the field index.
                    if let Some(class_name) = &self.current_class.clone() {
                        if let Some(&idx) = self.class_field_indices.get(&(class_name.clone(), access.member.clone())) {
                            let struct_ty = self.class_structs.get(class_name).unwrap().clone();
                            let field_ptr = self
                                .builder
                                .build_struct_gep(struct_ty, obj_ptr, idx, &access.member)
                                .unwrap();
                            self.builder.build_store(field_ptr, val).unwrap();
                            return Some(val);
                        }
                    }
                }
            }
            _ => {}
        }

        value
    }

    // ── Function calls ───────────────────────────────────────────

    fn gen_function_call(&mut self, call: &ast::FunctionCall) -> HulkValue<'ctx> {
        // Special handling for `print`.
        if call.name == "print" {
            return self.gen_print_call(call);
        }

        let llvm_func = match self.functions.get(&call.name) {
            Some(f) => *f,
            None => return None,
        };

        let args: Vec<BasicMetadataValueEnum<'ctx>> = call
            .args
            .iter()
            .filter_map(|arg| self.gen_expression(arg).map(|v| v.into()))
            .collect();

        let result = self
            .builder
            .build_call(llvm_func, &args, &call.name)
            .unwrap();

        result.try_as_basic_value().left()
    }

    /// Generates a `print(expr)` call, dispatching to the correct
    /// runtime function based on the argument type.
    fn gen_print_call(&mut self, call: &ast::FunctionCall) -> HulkValue<'ctx> {
        if call.args.is_empty() {
            return None;
        }
        let arg_val = self.gen_expression(&call.args[0]);

        match arg_val {
            Some(v) => {
                if v.is_float_value() {
                    let f = *self.functions.get("__hulk_print_number").unwrap();
                    self.builder
                        .build_call(f, &[v.into()], "")
                        .unwrap();
                } else if v.is_int_value() && v.into_int_value().get_type().get_bit_width() == 1 {
                    // Boolean — extend to i32 for C ABI.
                    let i32_val = self
                        .builder
                        .build_int_z_extend(
                            v.into_int_value(),
                            self.context.i32_type(),
                            "bool_ext",
                        )
                        .unwrap();
                    let f = *self.functions.get("__hulk_print_bool").unwrap();
                    self.builder
                        .build_call(f, &[i32_val.into()], "")
                        .unwrap();
                } else {
                    // Pointer — assume string.
                    let f = *self.functions.get("__hulk_print_string").unwrap();
                    self.builder
                        .build_call(f, &[v.into()], "")
                        .unwrap();
                }
            }
            None => {}
        }

        None // print returns Void
    }

    // ── Member access ────────────────────────────────────────────

    fn gen_member_access(&mut self, access: &ast::MemberAccess) -> HulkValue<'ctx> {
        let obj = self.gen_expression(&access.object)?;
        let obj_ptr = obj.into_pointer_value();

        // Determine object's class.
        // Try current class first (for self.x), then look up the expression type.
        let class_name = self.infer_class_name(&access.object);

        if let Some(ref cname) = class_name {
            if let Some(&idx) = self.class_field_indices.get(&(cname.clone(), access.member.clone())) {
                let struct_ty = self.class_structs.get(cname)?.clone();
                // Determine attribute type.
                let attr_type = self
                    .symbols
                    .resolve_attribute(cname, &access.member)
                    .map(|(_, a)| a.hulk_type.clone())
                    .unwrap_or(HulkType::Number);
                let llvm_ty = self.hulk_type_to_llvm(&attr_type);

                let field_ptr = self
                    .builder
                    .build_struct_gep(struct_ty, obj_ptr, idx, &access.member)
                    .unwrap();
                let val = self
                    .builder
                    .build_load(llvm_ty, field_ptr, &access.member)
                    .unwrap();
                return Some(val);
            }
        }

        None
    }

    // ── Method calls ─────────────────────────────────────────────

    fn gen_method_call(&mut self, call: &ast::MethodCall) -> HulkValue<'ctx> {
        let obj = self.gen_expression(&call.object)?;
        let obj_ptr = obj.into_pointer_value();

        let class_name = self.infer_class_name(&call.object);

        if let Some(ref cname) = class_name {
            // Resolve method to a specific class (for static dispatch).
            if let Some((owner_class, _method_info)) =
                self.symbols.resolve_method(cname, &call.method)
            {
                let method_llvm_name = format!("__{}__{}", owner_class, call.method);
                if let Some(&llvm_func) = self.functions.get(&method_llvm_name) {
                    let mut args: Vec<BasicMetadataValueEnum<'ctx>> =
                        vec![obj_ptr.into()];
                    for arg in &call.args {
                        if let Some(v) = self.gen_expression(arg) {
                            args.push(v.into());
                        }
                    }

                    let result = self
                        .builder
                        .build_call(llvm_func, &args, &call.method)
                        .unwrap();

                    return result.try_as_basic_value().left();
                }
            }
        }

        None
    }

    // ── New instance ─────────────────────────────────────────────

    fn gen_new_instance(&mut self, inst: &ast::NewInstance) -> HulkValue<'ctx> {
        let ctor_name = format!("__{}_new", inst.type_name);

        let ctor_func = match self.functions.get(&ctor_name) {
            Some(f) => *f,
            None => return None,
        };

        let args: Vec<BasicMetadataValueEnum<'ctx>> = inst
            .args
            .iter()
            .filter_map(|arg| self.gen_expression(arg).map(|v| v.into()))
            .collect();

        let result = self
            .builder
            .build_call(ctor_func, &args, "new_obj")
            .unwrap();

        result.try_as_basic_value().left()
    }

    // ── Helpers ──────────────────────────────────────────────────

    /// Infer the HULK type from an AST expression and/or its generated LLVM value.
    /// This is used in `let` declarations without type annotations.
    fn infer_hulk_type_from_expr(
        &self,
        expr: &ast::Expression,
        llvm_val: &HulkValue<'ctx>,
    ) -> HulkType {
        // 1. Check AST-level patterns that produce known types.
        match expr {
            ast::Expression::NewInstance(inst) => {
                return HulkType::Class(inst.type_name.clone());
            }
            ast::Expression::MethodCall(call) => {
                // Determine class name of object, then look up method return type.
                if let Some(class_name) = self.infer_class_name(&call.object) {
                    if let Some((_owner, method_info)) =
                        self.symbols.resolve_method(&class_name, &call.method)
                    {
                        return method_info.return_type.clone();
                    }
                }
            }
            ast::Expression::FunctionCall(call) => {
                // Look up function return type from symbol table.
                if let Some(func_info) = self.symbols.get_function(&call.name) {
                    return func_info.return_type.clone();
                }
            }
            _ => {}
        }

        // 2. Fall back to LLVM value type inspection.
        if let Some(v) = llvm_val {
            if v.is_float_value() {
                HulkType::Number
            } else if v.is_int_value()
                && v.into_int_value().get_type().get_bit_width() == 1
            {
                HulkType::Boolean
            } else {
                HulkType::String // default for pointers (strings)
            }
        } else {
            HulkType::Number
        }
    }

    /// Tries to determine the class name of an expression's result.
    fn infer_class_name(&self, expr: &ast::Expression) -> Option<String> {
        match expr {
            ast::Expression::Atom(atom) => {
                if let ast::atoms::atom::Atom::Variable(id) = atom.as_ref() {
                    if id.name == "self" {
                        return self.current_class.clone();
                    }
                    if let Some(ht) = self.symbols.var_type(&id.name) {
                        if let HulkType::Class(name) = ht {
                            return Some(name);
                        }
                    }
                }
                None
            }
            ast::Expression::NewInstance(inst) => Some(inst.type_name.clone()),
            ast::Expression::MethodCall(call) => {
                // Try to determine return type of method.
                if let Some(class_name) = self.infer_class_name(&call.object) {
                    if let Some((_owner, method_info)) =
                        self.symbols.resolve_method(&class_name, &call.method)
                    {
                        if let HulkType::Class(name) = &method_info.return_type {
                            return Some(name.clone());
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}
