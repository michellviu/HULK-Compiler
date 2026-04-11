//! Code generation for top-level functions and class methods.

use inkwell::types::{BasicMetadataTypeEnum, BasicType};

use parser::ast;
use parser::semantic::types::HulkType;

use super::context::CodegenContext;

impl<'ctx> CodegenContext<'ctx> {
    /// Generates LLVM IR for all top-level function declarations.
    pub fn gen_functions(&mut self, functions: &[ast::FunctionDecl]) {
        // Forward-declare all functions first.
        for func in functions {
            self.declare_function(func);
        }
        // Then generate bodies.
        for func in functions {
            self.gen_function_body(func);
        }
    }

    /// Forward-declares a HULK function in the LLVM module.
    fn declare_function(&mut self, func: &ast::FunctionDecl) {
        let func_info = self.symbols.get_function(&func.name).cloned();

        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = if let Some(info) = &func_info {
            info.params
                .iter()
                .map(|(_, ht)| self.hulk_type_to_meta(ht))
                .collect()
        } else {
            func
                .params
                .iter()
                .map(|p| {
                    let ht = match &p.type_ann {
                        Some(ann) => HulkType::from_name(ann),
                        None => HulkType::Number,
                    };
                    self.hulk_type_to_meta(&ht)
                })
                .collect()
        };

        let ret_type = func_info
            .as_ref()
            .map(|i| i.return_type.clone())
            .unwrap_or_else(|| match &func.return_type {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Void,
            });

        let fn_type = if Self::is_void_type(&ret_type) {
            self.void_type().fn_type(&param_types, false)
        } else {
            self.hulk_type_to_llvm(&ret_type)
                .fn_type(&param_types, false)
        };

        let llvm_func = self.module.add_function(&func.name, fn_type, None);
        self.functions.insert(func.name.clone(), llvm_func);
    }

    /// Generates the body of a HULK function.
    fn gen_function_body(&mut self, func: &ast::FunctionDecl) {
        let llvm_func = *self.functions.get(&func.name).unwrap();
        let func_info = self.symbols.get_function(&func.name).cloned();
        let entry_bb = self.context.append_basic_block(llvm_func, "entry");
        self.builder.position_at_end(entry_bb);

        self.push_scope();

        // Bind parameters to allocas.
        for (i, param) in func.params.iter().enumerate() {
            let ht = func_info
                .as_ref()
                .and_then(|info| info.params.get(i).map(|(_, t)| t.clone()))
                .unwrap_or_else(|| match &param.type_ann {
                    Some(ann) => HulkType::from_name(ann),
                    None => HulkType::Number,
                });
            let llvm_ty = self.hulk_type_to_llvm(&ht);
            let alloca = self.create_entry_block_alloca(llvm_func, &param.name, llvm_ty);
            self.builder
                .build_store(alloca, llvm_func.get_nth_param(i as u32).unwrap())
                .unwrap();
            self.set_variable(&param.name, alloca, llvm_ty);
        }

        // Generate body.
        let body_val = self.gen_body(&func.body);

        // Build return.
        let ret_type = func_info
            .as_ref()
            .map(|i| i.return_type.clone())
            .unwrap_or_else(|| match &func.return_type {
                Some(ann) => HulkType::from_name(ann),
                None => HulkType::Void,
            });

        // Only build return if current block has no terminator yet.
        let current_bb = self.builder.get_insert_block().unwrap();
        if current_bb.get_terminator().is_none() {
            if Self::is_void_type(&ret_type) {
                self.builder.build_return(None).unwrap();
            } else if let Some(val) = body_val {
                self.builder.build_return(Some(&val)).unwrap();
            } else {
                // Body produced void but function expects a return — return default.
                let default = self.default_value(&ret_type);
                self.builder.build_return(Some(&default)).unwrap();
            }
        }

        self.pop_scope();
    }

    /// Generates LLVM IR for a function/method `Body`.
    pub fn gen_body(&mut self, body: &ast::Body) -> super::context::HulkValue<'ctx> {
        match body {
            ast::Body::Inline(expr) => self.gen_expression(expr),
            ast::Body::Block(exprs) => {
                let mut last = None;
                for expr in exprs {
                    last = self.gen_expression(expr);
                }
                last
            }
        }
    }

    /// Generates LLVM IR for an `ExprBody`.
    pub fn gen_expr_body(&mut self, body: &ast::ExprBody) -> super::context::HulkValue<'ctx> {
        match body {
            ast::ExprBody::Single(expr) => self.gen_expression(expr),
            ast::ExprBody::Block(exprs) => {
                let mut last = None;
                for expr in exprs {
                    last = self.gen_expression(expr);
                }
                last
            }
        }
    }

    /// Returns a sensible default LLVM value for a given HULK type.
    pub fn default_value(&self, ht: &HulkType) -> inkwell::values::BasicValueEnum<'ctx> {
        match ht {
            HulkType::Number => self.f64_type().const_float(0.0).into(),
            HulkType::Boolean => self.bool_type().const_int(0, false).into(),
            HulkType::String => {
                let empty = self
                    .builder
                    .build_global_string_ptr("", "empty_str")
                    .unwrap();
                empty.as_pointer_value().into()
            }
            _ => self.ptr_type().const_null().into(),
        }
    }
}
