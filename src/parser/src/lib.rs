use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod errors;
pub mod tokens;
pub mod semantic;

pub use ast::Expression;
pub use ast::Program;
pub use errors::{SyntaxError, SyntaxErrorKind, SourceLocation};

lalrpop_mod!(pub grammar);

/// Parsea un programa HULK completo.
///
/// En caso de error devuelve un `SyntaxError` estructurado que puede
/// formatearse con `errors::format_syntax_error`.
pub fn parse_program(input: &str) -> Result<ast::Program, SyntaxError> {
    grammar::ProgramParser::new()
        .parse(input)
        .map_err(|err| errors::build_syntax_error(input, &err))
}

/// Parsea una expresión HULK suelta.
pub fn parse_expression(input: &str) -> Result<ast::Expression, SyntaxError> {
    grammar::ExprParser::new()
        .parse(input)
        .map_err(|err| errors::build_syntax_error(input, &err))
}

/// Formatea un `SyntaxError` como diagnóstico legible para la terminal.
pub fn format_error(err: &SyntaxError, filename: &str) -> String {
    errors::format_syntax_error(err, filename)
}
