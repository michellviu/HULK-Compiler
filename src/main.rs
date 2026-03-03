use std::env;
use std::fs;

use parser::ast::{AstPrinterVisitor, Visitable};
use parser::semantic;

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
    let print_ast = env::args().any(|a| a == "--ast");
    if print_ast {
        let mut printer = AstPrinterVisitor::new();
        program.accept(&mut printer);
        println!();
    }

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

    // ── Phase 5: Codegen (not yet implemented) ──────────────────
    println!("\x1b[1;32m✓\x1b[0m Análisis semántico y de tipos completado exitosamente.");
}
