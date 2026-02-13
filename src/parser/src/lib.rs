use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod tokens;

pub use ast::Expression;

lalrpop_mod!(pub grammar);

pub fn parse_expression(input: &str) -> Result<ast::Expression, String> {
    grammar::ExpressionParser::new()
        .parse(input)
        .map_err(|err| err.to_string())
}
