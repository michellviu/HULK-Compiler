use std::fmt;
use super::position::Position;

#[derive(Debug, Clone)]
pub enum BinOp {
    // Arithmetic
    Plus(Position),
    Minus(Position),
    Mul(Position),
    Div(Position),
    Mod(Position),
    Pow(Position),

    // Comparison
    EqualEqual(Position),
    NotEqual(Position),
    Less(Position),
    LessEqual(Position),
    Greater(Position),
    GreaterEqual(Position),

    // Logical
    And(Position),           // &
    Or(Position),            // |

    // String concatenation
    Concat(Position),        // @
    ConcatSpaced(Position),  // @@

    // Assignment / initialization
    Equal(Position),         // =
    Assign(Position),        // :=
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Plus(_) => "+",
            BinOp::Minus(_) => "-",
            BinOp::Mul(_) => "*",
            BinOp::Div(_) => "/",
            BinOp::Mod(_) => "%",
            BinOp::Pow(_) => "^",
            BinOp::EqualEqual(_) => "==",
            BinOp::NotEqual(_) => "!=",
            BinOp::Less(_) => "<",
            BinOp::LessEqual(_) => "<=",
            BinOp::Greater(_) => ">",
            BinOp::GreaterEqual(_) => ">=",
            BinOp::And(_) => "&",
            BinOp::Or(_) => "|",
            BinOp::Concat(_) => "@",
            BinOp::ConcatSpaced(_) => "@@",
            BinOp::Equal(_) => "=",
            BinOp::Assign(_) => ":=",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Minus(Position),
    Not(Position),
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnaryOp::Minus(_) => "-",
            UnaryOp::Not(_) => "!",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub enum GroupingOperator {
    OpenParen(Position),
    CloseParen(Position),
    OpenBrace(Position),
    CloseBrace(Position),
    OpenBracket(Position),
    CloseBracket(Position),
}

impl fmt::Display for GroupingOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GroupingOperator::OpenParen(_) => "(",
            GroupingOperator::CloseParen(_) => ")",
            GroupingOperator::OpenBrace(_) => "{",
            GroupingOperator::CloseBrace(_) => "}",
            GroupingOperator::OpenBracket(_) => "[",
            GroupingOperator::CloseBracket(_) => "]",
        };
        write!(f, "{}", s)
    }
}