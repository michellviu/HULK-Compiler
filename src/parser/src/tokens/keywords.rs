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
        };
        write!(f, "{}", s)
    }
}