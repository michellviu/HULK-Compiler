use std::env;
use std::fs;

use parser::ast::{AstPrinterVisitor, Visitable};

fn main() {
    let script_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "script.hulk".to_string());

    let source = fs::read_to_string(&script_path).unwrap_or_else(|err| {
        eprintln!("ERROR: No se pudo leer {}: {}", script_path, err);
        std::process::exit(1);
    });

    match parser::parse_program(&source) {
        Ok(program) => {
            let mut printer = AstPrinterVisitor::new();
            program.accept(&mut printer);
        }
        Err(syntax_err) => {
            let diagnostic = parser::format_error(&syntax_err, &script_path);
            eprintln!("{}", diagnostic);
            std::process::exit(1);
        }
    }
}
