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

    match parser::parse_expression(&source) {
        Ok(expr) => {
            let mut printer = AstPrinterVisitor::new();
            expr.accept(&mut printer);
        }
        Err(err) => {
            eprintln!("ERROR: Fallo el parseo: {}", err);
            std::process::exit(1);
        }
    }
}
