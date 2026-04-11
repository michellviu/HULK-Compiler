use std::env;
use std::fs;
use std::path::Path;

use parser::ast::{AstPrinterVisitor, Visitable};
use parser::semantic;

mod codegen;

fn main() {
    let script_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "script.hulk".to_string());

    let source = fs::read_to_string(&script_path).unwrap_or_else(|err| {
        eprintln!("ERROR: No se pudo leer {}: {}", script_path, err);
        std::process::exit(1);
    });

    // ── Phase 1: Parse ──────────────────────────────────────────
    let program = match parser::parse_program(&source) {
        Ok(p) => p,
        Err(syntax_err) => {
            let diagnostic = parser::format_error(&syntax_err, &script_path);
            eprintln!("{}", diagnostic);
            std::process::exit(1);
        }
    };

    // (Optional) Print AST if --ast flag is given.
    // let print_ast = env::args().any(|a| a == "--ast");
    // if print_ast {
        let mut printer = AstPrinterVisitor::new();
        program.accept(&mut printer);
        println!();
    // }

    // ── Phases 2-4: Semantic analysis ───────────────────────────
    let result = semantic::analyze(&program);

    // Print all diagnostics (errors and warnings).
    let mut has_errors = false;
    for diag in &result.diagnostics {
        let formatted = semantic::format_compiler_error(diag, &source, &script_path);
        match diag.severity {
            semantic::Severity::Error => {
                has_errors = true;
                eprint!("{}", formatted);
            }
            semantic::Severity::Warning => {
                eprint!("{}", formatted);
            }
        }
    }

    if has_errors {
        let error_count = result.errors().len();
        let warning_count = result.warnings().len();
        eprintln!(
            "\n\x1b[1;31merror\x1b[0m: No se puede compilar debido a {} error(es){}",
            error_count,
            if warning_count > 0 {
                format!(" y {} advertencia(s)", warning_count)
            } else {
                String::new()
            }
        );
        std::process::exit(1);
    }

    let warning_count = result.warnings().len();
    if warning_count > 0 {
        eprintln!(
            "\x1b[1;33mwarning\x1b[0m: {} advertencia(s) generada(s)",
            warning_count
        );
    }

    // ── Phase 5: Code generation ──────────────────────────────────
    let output_name = Path::new(&script_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let output_path = Path::new(output_name);

    // Find runtime relative to the executable or in standard locations.
    let runtime_path = find_runtime();

    eprintln!(
        "\x1b[1;34m→\x1b[0m Generando código LLVM…"
    );

    match codegen::compile(
        &program,
        result.symbols,
        output_path,
        &runtime_path,
        &script_path,
        &source,
    ) {
        Ok(exe_path) => {
            eprintln!(
                "\x1b[1;32m✓\x1b[0m Compilación exitosa → \x1b[1m{}\x1b[0m",
                exe_path.display()
            );
        }
        Err(err) => {
            eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
            std::process::exit(1);
        }
    }
}

/// Searches for the HULK C runtime source file.
fn find_runtime() -> std::path::PathBuf {
    // Try relative to CWD.
    let candidates = [
        "runtime/hulk_runtime.c",
        "src/runtime/hulk_runtime.c",
        "../runtime/hulk_runtime.c",
    ];
    for c in &candidates {
        let p = Path::new(c);
        if p.exists() {
            return p.to_path_buf();
        }
    }

    // Try relative to the executable.
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("../../runtime/hulk_runtime.c");
            if p.exists() {
                return p;
            }
        }
    }

    // Fallback.
    eprintln!("\x1b[1;31merror\x1b[0m: No se encontró runtime/hulk_runtime.c");
    std::process::exit(1);
}
