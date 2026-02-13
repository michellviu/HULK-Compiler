use std::fmt;
use super::position::Position;
#[derive(Debug,Clone)]
pub enum BinOp {
    // Binary operators
    Mul(Position),
    Div(Position),
    Mod(Position),
    Pow(Position),
    Plus(Position),
    Minus(Position),

    // Comparison operators
    EqualEqual(Position),
    NotEqual(Position),      // !=
    Less(Position),
    LessEqual(Position),
    Greater(Position),
    GreaterEqual(Position),

    // Logical operators
    AndAnd(Position),        // &&
    OrOr(Position),          // ||
    Equal(Position),         // =
    Assign(Position),        // :=
    ConcatString(Position),  // @
}


impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Mul(_) => "*",
            BinOp::Div(_) => "/",
            BinOp::Plus(_) => "+",
            BinOp::Minus(_) => "-",
            BinOp::Mod(_) => "%",
            BinOp::Pow(_) => "^",
            BinOp::EqualEqual(_) => "==",
            BinOp::NotEqual(_) => "!=",
            BinOp::Less(_) => "<",
            BinOp::LessEqual(_) => "<=",
            BinOp::Greater(_) => ">",
            BinOp::GreaterEqual(_) => ">=",
            BinOp::AndAnd(_) => "&&",
            BinOp::OrOr(_) => "||",
            BinOp::Equal(_) => "=",
            BinOp::Assign(_) => ":=",
            BinOp::ConcatString(_) => "@",
        };
        write!(f, "{}", s)
    }
}
#[derive(Debug,Clone)]
pub enum UnaryOp {
    Plus(Position),
    Minus(Position),
    Not(Position),
}


impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnaryOp::Plus(_) => "+",
            UnaryOp::Minus(_) => "-",
            UnaryOp::Not(_) => "!",
        };
        write!(f, "{}", s)
    }
}
#[derive(Debug)]
pub enum SpecialOp {
    Semicolon(Position),
    Comma(Position),
    Colon(Position),
    Dot(Position),
    Arrow(Position),
}

impl fmt::Display for SpecialOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SpecialOp::Semicolon(_) => ";",
            SpecialOp::Comma(_) => ",",
            SpecialOp::Colon(_) => ":",
            SpecialOp::Dot(_) => ".",
            SpecialOp::Arrow(_) => "=>",
        };
        write!(f, "{}", s)
    }
}
#[derive(Debug,Clone)]
pub enum GroupingOperator {
    OpenParen(Position),
    CloseParen(Position),
    OpenBrace(Position),
    CloseBrace(Position),
}

impl fmt::Display for GroupingOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GroupingOperator::OpenParen(_) => "(",
            GroupingOperator::CloseParen(_) => ")",
            GroupingOperator::OpenBrace(_) => "{",
            GroupingOperator::CloseBrace(_) => "}",
        };
        write!(f, "{}", s)
    }
}