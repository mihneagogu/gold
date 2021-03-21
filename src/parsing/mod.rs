use std::fs;
use std::collections::HashSet;

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
    keywords: HashSet<&'static str>
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

    #[inline]
    pub fn current_state(&self) -> (usize, usize, usize, &'inp str) {
        (self.row, self.col, self. index, self.cursor)
    }

    #[inline]
    pub fn roll_back_op(&mut self, (row, col, idx, cursor) : (usize, usize, usize, &'inp str)) {
        self.row = row;
        self.col = col;
        self.index = idx;
        self.cursor = cursor;
    }

    pub fn new<T>(input: &'inp T) -> Self
        where T: AsRef<str> + ?Sized
    {
        // @MAYBE(mike) we could use lazy_static! here but we would like to avoid the unnecessary dependencies
        let keywords: HashSet<&'static str> = vec!["def", "for", "if", "else"].into_iter().collect();
        let mut s = Self { row: 1, col: 1, index: 0, input: input.as_ref(), cursor: input.as_ref(), keywords };
        s.eat_ws();
        s
    }

    pub fn eat_many(&mut self, n: usize) -> Option<&str> {
        assert_eq!(n != 0, true, "Cannot eat 0 chars");
        if self.cursor.len() < n {
            return None;
        }         

        for (i, c) in self.cursor.chars().enumerate() {
            if i == n {
                break;
            }
            if c == '\n' {
                self.row += 1; self.col = 1;
            } else {
                self.col += 1;
            }
        }
        self.index += n;
        let eaten = &self.cursor[.. n];
        self.cursor = &self.cursor[n ..];
        Some(eaten)
    }

    pub fn eat_until_ws(&mut self) -> &str {
        let mut advanced = 0;
        for (i, c) in self.cursor.chars().enumerate() {
            if c.is_whitespace() {
                advanced = i;
                break;
            }
            if i == self.cursor.len() - 1 {
               advanced = i + 1;
            }
        }
        self.col += advanced;
        self.index += advanced;
        let until_ws = &self.cursor[..advanced];
        self.cursor = &self.cursor[advanced..];
        until_ws
    }

    pub fn eat_ws(&mut self) -> &mut Self {
        let mut non_ws = 0;
        for (i, c) in self.cursor.chars().enumerate() {
             match c {
                '\n' => { self.row += 1; self.col = 1 }
                ch if !ch.is_whitespace() => { non_ws = i; break },
                _ => self.col += 1
            }
        }
        self.cursor = &self.cursor[non_ws..];
        self.index = self.index + non_ws;
        self
    }

}


