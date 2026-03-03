#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub start: usize,
    pub end: usize,
}

impl Position {
    pub fn new(start: usize, end: usize) -> Self {
        Position { start, end }
    }
}

/// A source span covering a range of bytes in the input.
/// Used on every AST node for error reporting.
pub type Span = Position;
