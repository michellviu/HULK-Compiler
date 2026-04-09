//! LLVM code generation module for the HULK compiler.
//!
//! This module translates a type-checked HULK AST into LLVM IR using
//! the [inkwell](https://crates.io/crates/inkwell) safe wrapper, then
//! writes a native object file that can be linked with the HULK C
//! runtime to produce an executable.
//!
//! # Architecture
//!
//! - [`context`]     — `CodegenContext`: LLVM context/module/builder + lookup tables
//! - [`types`]       — `HulkType` → LLVM type mapping
//! - [`builtins`]    — declarations for runtime C functions
//! - [`functions`]   — top-level function codegen
//! - [`classes`]     — class struct layout, constructors, method codegen
//! - [`expressions`] — expression codegen (atoms, ops, calls, control flow, etc.)

mod context;
mod types;
mod builtins;
mod functions;
mod classes;
mod expressions;

pub use context::CodegenContext;

use std::path::{Path, PathBuf};
use std::process::Command;

use inkwell::context::Context;

use parser::ast;
use parser::semantic::SymbolTable;

/// Compiles a type-checked HULK program to a native executable.
///
/// 1. Generates LLVM IR in memory.
/// 2. Writes an object file (`.o`).
/// 3. Links with the HULK C runtime via the system C compiler.
///
/// `output` is the path for the final executable (e.g. `./output`).
pub fn compile(
    program: &ast::Program,
    symbols: SymbolTable,
    output: &Path,
    runtime_path: &Path,
) -> Result<PathBuf, String> {
    let context = Context::create();
    let mut cg = CodegenContext::new(&context, "hulk_module", symbols);

    // ── 1. Declare built-in / runtime functions ──────────────────
    cg.declare_builtins();

    // ── 2. Generate classes ──────────────────────────────────────
    cg.gen_classes(&program.classes);

    // ── 3. Generate top-level functions ──────────────────────────
    cg.gen_functions(&program.functions);

    // ── 4. Generate main() wrapping the entry expression ─────────
    gen_main(&mut cg, program);

    // ── 5. Verify the module ─────────────────────────────────────
    if let Err(msg) = cg.module.verify() {
        // Write IR for debugging even on failure.
        let ir_path = output.with_extension("ll");
        cg.write_ir(&ir_path);
        return Err(format!(
            "Error de verificación LLVM:\n{}\n(IR guardado en {})",
            msg.to_string(),
            ir_path.display()
        ));
    }

    // ── 6. Write LLVM IR (for debugging) ─────────────────────────
    let ir_path = output.with_extension("ll");
    cg.write_ir(&ir_path);

    // ── 7. Write object file ─────────────────────────────────────
    let obj_path = output.with_extension("o");
    cg.write_object_file(&obj_path)?;

    // ── 8. Compile runtime ───────────────────────────────────────
    let runtime_obj = output.with_extension("rt.o");
    let cc_status = Command::new("cc")
        .args([
            "-c",
            "-o",
            runtime_obj.to_str().unwrap(),
            runtime_path.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format!("No se pudo ejecutar el compilador de C: {}", e))?;

    if !cc_status.success() {
        return Err("Error al compilar el runtime de C".into());
    }

    // ── 9. Link ──────────────────────────────────────────────────
    let link_status = Command::new("cc")
        .args([
            "-o",
            output.to_str().unwrap(),
            obj_path.to_str().unwrap(),
            runtime_obj.to_str().unwrap(),
            "-lm", // link libm for sin/cos/sqrt/exp/log
        ])
        .status()
        .map_err(|e| format!("No se pudo ejecutar el linker: {}", e))?;

    if !link_status.success() {
        return Err("Error al enlazar el ejecutable".into());
    }

    // Clean up intermediate files.
    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_file(&runtime_obj);

    Ok(output.to_path_buf())
}

/// Generates the C `main()` function wrapping the HULK entry expression.
fn gen_main(cg: &mut CodegenContext<'_>, program: &ast::Program) {
    let i32_type = cg.context.i32_type();
    let main_type = i32_type.fn_type(&[], false);
    let main_func = cg.module.add_function("main", main_type, None);
    let entry_bb = cg.context.append_basic_block(main_func, "entry");
    cg.builder.position_at_end(entry_bb);

    cg.push_scope();

    if let Some(ref entry) = program.entry {
        cg.gen_expr_body(entry);
    }

    // Return 0.
    let current_bb = cg.builder.get_insert_block().unwrap();
    if current_bb.get_terminator().is_none() {
        cg.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .unwrap();
    }

    cg.pop_scope();
}
