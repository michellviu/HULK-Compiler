#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Position {
    pub start: usize,
    pub end: usize,
}

impl Position {
    pub fn new(start: usize, end: usize) -> Self {
        Position { start, end }
    }
}
