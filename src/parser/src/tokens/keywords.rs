use std::fmt::Display;
use super::position::Position;

#[derive(Debug,Clone, Copy)]
pub enum Keyword {
    Let(Position),
    In(Position),
    If(Position),
    Else(Position),
    Elif(Position),
    Print(Position),
    While(Position),
    For(Position),
    Function(Position),
    Class(Position),
    Case(Position),
    Of(Position),
    New(Position),
    Is(Position),
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       let s = match self {
            Keyword::Let(_) => "let",
            Keyword::If(_) => "if",
            Keyword::Else(_) => "else",
            Keyword::While(_) => "while",
            Keyword::Print(_) => "print",
            Keyword::In(_) => "in",
            Keyword::Elif(_) => "elif",
            Keyword::For(_) => "for",
            Keyword::Function(_) => "function",
            Keyword::Class(_) => "class",
            Keyword::Case(_) => "case",
            Keyword::Of(_) => "of",
            Keyword::New(_) => "new",
            Keyword::Is(_) => "is",
        };
        write!(f, "{}", s)
    }
}