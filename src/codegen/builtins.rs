//! Declarations for built-in / runtime functions.
//!
//! These correspond to C functions provided by the HULK runtime library
//! (`hulk_runtime.c`).  We declare them as LLVM external functions so
//! the rest of codegen can call them directly.

use inkwell::types::BasicMetadataTypeEnum;

use super::context::CodegenContext;

impl<'ctx> CodegenContext<'ctx> {
    /// Declares all built-in runtime functions in the LLVM module.
    pub fn declare_builtins(&mut self) {
        let f64_ty = self.f64_type();
        let ptr_ty = self.ptr_type();
        let void_ty = self.void_type();
        let f64_meta: BasicMetadataTypeEnum = f64_ty.into();
        let ptr_meta: BasicMetadataTypeEnum = ptr_ty.into();

        // ── print ────────────────────────────────────────────────
        // void hulk_print_number(double)
        let print_num_ty = void_ty.fn_type(&[f64_meta], false);
        let f = self.module.add_function("hulk_print_number", print_num_ty, None);
        self.functions.insert("__hulk_print_number".into(), f);

        // void hulk_print_string(const char*)
        let print_str_ty = void_ty.fn_type(&[ptr_meta], false);
        let f = self.module.add_function("hulk_print_string", print_str_ty, None);
        self.functions.insert("__hulk_print_string".into(), f);

        // void hulk_print_bool(i1)  — we'll extend to i8 in the call
        // Actually, pass as i32 for C ABI compatibility:
        let i32_ty = self.context.i32_type();
        let i32_meta: BasicMetadataTypeEnum = i32_ty.into();
        let print_bool_ty = void_ty.fn_type(&[i32_meta], false);
        let f = self.module.add_function("hulk_print_bool", print_bool_ty, None);
        self.functions.insert("__hulk_print_bool".into(), f);

        // ── math functions ───────────────────────────────────────
        let unary_f64 = f64_ty.fn_type(&[f64_meta], false);
        let binary_f64 = f64_ty.fn_type(&[f64_meta, f64_meta], false);
        let nullary_f64 = f64_ty.fn_type(&[], false);

        for name in &["sin", "cos", "sqrt", "exp"] {
            let f = self.module.add_function(name, unary_f64, None);
            self.functions.insert(name.to_string(), f);
        }

        // log(base, x) → double
        let f = self.module.add_function("hulk_log", binary_f64, None);
        self.functions.insert("log".into(), f);

        // rand() → double
        let f = self.module.add_function("hulk_rand", nullary_f64, None);
        self.functions.insert("rand".into(), f);

        // ── string helper ────────────────────────────────────────
        // char* hulk_concat(const char*, const char*)
        let concat_ty = ptr_ty.fn_type(&[ptr_meta, ptr_meta], false);
        let f = self.module.add_function("hulk_concat", concat_ty, None);
        self.functions.insert("__hulk_concat".into(), f);

        // char* hulk_concat_spaced(const char*, const char*)
        let f = self.module.add_function("hulk_concat_spaced", concat_ty, None);
        self.functions.insert("__hulk_concat_spaced".into(), f);

        // ── conversion helpers ───────────────────────────────────
        // char* hulk_number_to_string(double)
        let n2s_ty = ptr_ty.fn_type(&[f64_meta], false);
        let f = self.module.add_function("hulk_number_to_string", n2s_ty, None);
        self.functions.insert("__hulk_number_to_string".into(), f);

        // char* hulk_bool_to_string(i32)
        let b2s_ty = ptr_ty.fn_type(&[i32_meta], false);
        let f = self.module.add_function("hulk_bool_to_string", b2s_ty, None);
        self.functions.insert("__hulk_bool_to_string".into(), f);

        // ── memory allocation ────────────────────────────────────
        // void* hulk_alloc(i64 size)
        let i64_ty = self.context.i64_type();
        let i64_meta: BasicMetadataTypeEnum = i64_ty.into();
        let alloc_ty = ptr_ty.fn_type(&[i64_meta], false);
        let f = self.module.add_function("hulk_alloc", alloc_ty, None);
        self.functions.insert("__hulk_alloc".into(), f);

        // double hulk_pow(double base, double exp)
        let f = self.module.add_function("hulk_pow", binary_f64, None);
        self.functions.insert("__hulk_pow".into(), f);
    }
}
