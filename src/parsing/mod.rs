/// This represents the whole parsing module, which contains the context, 
/// and the parser trait and some parsing utilities. The parser combinator
/// themselves are in different files from this module
/// This parsing framework has most of its inspiration from Parsley Scala library
/// (https://github.com/j-mie6/Parsley). While it is popular to create the parsing machine
/// at compile-time via a state machine, we opt for a handwritten parser which operates at runtime
/// (since realistically parsing is never the longest part of a compiler and it allows us
/// to have perfect control over the erorr messages we give out).
/// There are however a few distinctions between parsley and this parsing framework, which
/// are stated in the combinators. Also, if you are familiar with parsing combinators from
/// functional languages, this parsing framework will feel very at home, albeit more verbose.
/// The parsing framework itself can be written as a proc macro with custom instructions 
/// but that would be a library on its own. What we are interested in is precisely to parse
/// correctly and give good errors, but exactly how we parse the source code.

use std::fs;
use std::collections::HashSet;

pub mod literals;
pub mod combinators;

// Empty for now
pub trait ParserErr {}


/// This is a struct which represents the whole parsing context,
/// which takes care of the positions in the input where we are currently at
/// (row, column, index in the input, and a cursor)
#[derive(Debug)]
pub struct ParsingContext<'inp> {
    pub row: usize,
    pub col: usize,
    pub index: usize, // The place where we are at in the input
    pub input: &'inp str, // The whole input
    pub cursor: &'inp str, // Where we are currently in the input
    keywords: HashSet<&'static str>
}

#[derive(Debug)]
struct DoubleParser<F, S> {
    first: F,
    second: S
}

impl<F, S> DoubleParser<F, S> {
    fn new(first: F, second: S) -> Self {
        Self { first, second }
    }
}

impl<F: Parser, S: Parser> Parser for DoubleParser<F, S> {
    type Output = (F::Output, S::Output);
    // @MAYBE(mike) maybe this should be a Box<dyn ParserErr> and we return the right one?
    // Like this it will return a bunch of FirstErr(SecondErr(FirstErr(... ) ) ) if we keep
    // chaining
    // the discard_then and then_discard
    type PErr = DoubleParserErr<F::PErr, S::PErr>;

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let r1;
        match self.first.parse(ctx) {
            Err(e) => return Err(DoubleParserErr::FirstError(e)),
            Ok(r) => { r1 = r; }
        };
        match self.second.parse(ctx) {
            Ok(r2) => Ok((r1, r2)),
            Err(e) => Err(DoubleParserErr::SecondError(e))
        }
    }
}

// TODO(mike): Refactor common functionality between ThenDiscard and DiscardThen
// and combine them into one
#[derive(Debug)]
pub struct ThenDiscardParser<F, S> {
    first: F,
    second: S
}

impl<F: Parser, S: Parser> Parser for ThenDiscardParser<F, S> {
    type Output = F::Output;
    type PErr = DoubleParserErr<F::PErr, S::PErr>;

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        DoubleParser::new(&self.first, &self.second).parse(ctx).map(|(fst, _)| fst)
    }
}

impl<F, S> ThenDiscardParser<F, S> {
    pub fn new(first: F, second: S) -> Self {
        Self { first, second }
    }
}

#[derive(Debug)]
pub struct DiscardThenParser<F, S> {
    first: F,
    second: S
}

impl<F, S> DiscardThenParser<F, S> {
    pub fn new(first: F, second: S) -> Self {
        Self { first, second }
    }
}

#[derive(Debug)]
pub enum DoubleParserErr<F, S> 
where F: ParserErr, S: ParserErr
{
    FirstError(F),
    SecondError(S)
}

impl<F: ParserErr, S: ParserErr> ParserErr for DoubleParserErr<F, S> {}

impl<F: Parser, S: Parser> Parser for DiscardThenParser<F, S> {
    type Output = S::Output;
    type PErr = DoubleParserErr<F::PErr, S::PErr>;

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        DoubleParser::new(&self.first, &self.second).parse(ctx).map(|(_, snd)| snd)
    }

}

impl<'a, E: ParserErr> ParserErr for &'a E {}
impl<'a, T: Parser> Parser for &'a T {
    type Output = T::Output;
    type PErr = T::PErr;
    fn parse(&self, ctx: &mut ParsingContext) -> Result<T::Output, T::PErr> {
        T::parse(self, ctx)
    }
}

pub trait Parser {
    // TODO(mike): Probably need to add require a label name
    type Output;
    type PErr: ParserErr;
    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr>;
    fn discard_then<P: Parser>(self, snd: P) -> DiscardThenParser<Self, P> 
        where Self: Sized
    {
        DiscardThenParser::new(self, snd)
    }

    /// Runs the parser on given input. Useful for small-scale testing
    fn run_praser(&self, inp: &str) -> Result<Self::Output, Self::PErr> {
        let mut ctx = ParsingContext::new(inp);
        self.parse(&mut ctx)
    }

    fn then_discard<P: Parser>(self, snd: P) -> ThenDiscardParser<Self, P>
        where Self: Sized
    {
        ThenDiscardParser::new(self, snd)
    }
}

impl<'inp> ParsingContext<'inp> {

    /// Peek the cursor and return the first character, if any
    #[allow(dead_code)]
    pub fn peek_char(&self) -> Option<char> {
        self.cursor.chars().peekable().peek().copied()
    }

    pub fn contains_keyword(&self, w: &str) -> bool {
        self.keywords.contains(w)
    }

    /// What is the row, column, index where the cursor is at the moment?
    #[inline]
    pub fn current_state(&self) -> (usize, usize, usize, &'inp str) {
        (self.row, self.col, self. index, self.cursor)
    }

    /// Roll back the parser state to that position (which is usually before an
    /// operation which failed was done). Used by the attempt parser to undo operations
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
        let keywords: HashSet<&'static str> = vec!["def", "for", "if", "else"].into_iter().collect();
        let mut s = Self { row: 1, col: 1, index: 0, input: input.as_ref(), cursor: input.as_ref(), keywords };
        s.eat_ws();
        s
    }

    /// Eat n characters from the input and spit them back, if there are enough in the input
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

    /// Eat everything until whitespace (or end of input) and spit it back
    pub fn eat_until_ws(&mut self) -> &str {
        let mut advanced = 0;
        let mut found_last = false;
        for (i, c) in self.cursor.chars().enumerate() {
            if c.is_whitespace() {
                advanced = i;
                break;
            }
            if i == self.cursor.len() - 1 {
               found_last = true;
               advanced = i + 1;
            }
        }
        if !found_last && advanced == 0 { 
            // It means we did actually reach the end of the string, but the additions
            // don't add up since we might have encountered a non-ascii char along the way
            advanced = self.cursor.len();
        }
        self.col += advanced;
        self.index += advanced;
        let until_ws = &self.cursor[..advanced];
        self.cursor = &self.cursor[advanced..];
        until_ws
    }

    /// Discard all whitespace. Returns self for chaining commodity
    pub fn eat_ws(&mut self) -> &mut Self {
        let mut non_ws = 0;
        let mut found = false;
        for (i, c) in self.cursor.chars().enumerate() {
            non_ws = i;
             match c {
                '\n' => { self.row += 1; self.col = 1 }
                ch if !ch.is_whitespace() => { found = true; break }
                _ => {  self.col += 1 }
            }
        }
        let non_ws = if found || non_ws == 0 { non_ws } else { non_ws + 1};
        self.cursor = &self.cursor[non_ws..];
        self.index = self.index + non_ws;
        self
    }

}


