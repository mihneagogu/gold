use std::fs;

pub mod literals;
pub trait ParserErr {}


pub(crate) struct Parser<'inp> {
    pub row: usize,
    pub col: usize,
    pub index: usize, // The place where we are at in the input
    pub input: &'inp str, // The whole input
    pub cursor: &'inp str, // Where we are currently in the input
}

impl<'inp> Parser<'inp> {


    pub fn from_str(input: &'inp str) -> Self {
        Parser { row: 1, col: 0, index: 0, input, cursor: input }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.cursor.chars().peekable().peek().copied()
    }

    pub fn new(file_path: &str) -> Self {
        todo!()
    }

    pub fn eat_ws(&mut self) -> &mut Self {
        let mut non_ws = 0;
        for (i, c) in self.cursor.chars().enumerate() {
             match c {
                '\n' => { self.row += 1; self.col = 0 }
                ch if !ch.is_whitespace() => { non_ws = i; self.col += 1; break },
                _ => self.col += 1
            }
        }
        self.cursor = &self.cursor[non_ws..];
        self.index = self.index + non_ws;
        self
    }

}


