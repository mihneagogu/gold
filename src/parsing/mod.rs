use std::fs;

pub mod literals;
pub mod combinators;

pub trait ParserErr {}


#[derive(Debug)]
pub struct ParsingContext<'inp> {
    pub row: usize,
    pub col: usize,
    pub index: usize, // The place where we are at in the input
    pub input: &'inp str, // The whole input
    pub cursor: &'inp str, // Where we are currently in the input
}

pub trait Parser {
    type Output;
    type PErr: ParserErr;
    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr>;
}

impl<'inp> ParsingContext<'inp> {

    #[allow(dead_code)]
    pub fn peek_char(&self) -> Option<char> {
        self.cursor.chars().peekable().peek().copied()
    }

    pub fn new<T>(input: &'inp T) -> Self
        where T: AsRef<str> + ?Sized
    {
        Self { row: 1, col: 0, index: 0, input: input.as_ref(), cursor: input.as_ref()}
    }

    pub fn eat_until_ws(&mut self) -> &str {
        let mut advanced = 0;
        for (i, c) in self.cursor.chars().enumerate() {
            if c.is_whitespace() {
                advanced = i;
                break;
            }
            self.index += 1;
            if i == self.cursor.len() - 1 {
               advanced = i + 1;
            }
        }
        self.index += advanced;
        let until_ws = &self.cursor[..advanced];
        self.cursor = &self.cursor[advanced..];
        until_ws
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


