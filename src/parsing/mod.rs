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
use std::cell::UnsafeCell;
use std::collections::VecDeque;

pub mod statements;
pub mod literals;
pub mod combinators;
pub mod types;
use combinators::{StringParseErr, StringParser};

use self::combinators::AlternativeParser;

// Empty for now
pub(crate) trait ParserErr: Debug {}


/// This is a struct which represents the whole parsing context,
/// which takes care of the positions in the input where we are currently at
/// (row, column, index in the input, and a cursor)
#[derive(Debug)]
pub(crate) struct ParsingContext<'inp> {
    pub row: usize,
    pub col: usize,
    pub index: usize, // The place where we are at in the input
    pub input: &'inp str, // The whole input
    pub cursor: &'inp str, // Where we are currently in the input
    keywords: HashSet<&'static str>
}

// TODO(mike): Refactor so that ParsingBaggage is different from ParsingContext,
// so we can avoid the awkward transmute in parsing/types.rs

/// Metadata about the special things to consider when parsing
#[derive(Debug)]
pub(crate) struct ParsingBaggage<'pctx> {
    // The order of the base types matter, since the base type parser will be
    // in the order of base_types.
    pub base_types: Vec<&'static str>,
    pub base_type_string_parers: Vec<StringParser>,
    pub base_type_parser: AlternativeParser<'pctx, &'static str, StringParseErr>,
}

impl<'pctx> ParsingBaggage<'pctx> {
    pub fn init() -> Self {

        let base_types = vec!["i128", "i64", "i32", "i16", "i8", "u128", "u64", "u32", "u16", "u8", "bool", "()", "f64", "f32"]; 
        let ps: Vec<_> = base_types.clone().into_iter().map(|t| StringParser::new(t)).collect();
        let ps: UnsafeCell<Vec<_>> = ps.into();

        // TODO(mike): Make an AlternativeParser which takes an owned type, 
        // this is kind of awkward since we are storing ps in the struct just to keep the
        // references of the AlternativeParser alive, but they serve no other purpose.
        // We should just an alternative parser which takes either a Vec<T: Parser> 
        // and one which takes a Vec<Box<dyn Parser>> so that we can own the parsers inside
        let parsers: Vec<_> = unsafe {
            (&*ps.get()).iter().map(|p| p as &dyn Parser<Output = &'static str, PErr = StringParseErr>).collect()
        };
        let ap = AlternativeParser::new(parsers);

        let ps = ps.into_inner();
        Self { base_types, base_type_string_parers: ps, base_type_parser: ap }
    }
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

    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let r1;
        match self.first.parse(baggage, ctx) {
            Err(e) => return Err(DoubleParserErr::FirstError(e)),
            Ok(r) => { r1 = r; }
        };
        match self.second.parse(baggage, ctx) {
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

    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        DoubleParser::new(&self.first, &self.second).parse(baggage, ctx).map(|(fst, _)| fst)
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
pub(crate) enum DoubleParserErr<F, S> 
where F: ParserErr, S: ParserErr
{
    FirstError(F),
    SecondError(S)
}

impl<F: ParserErr, S: ParserErr> ParserErr for DoubleParserErr<F, S> {}

impl<F: Parser, S: Parser> Parser for DiscardThenParser<F, S> {
    type Output = S::Output;
    type PErr = DoubleParserErr<F::PErr, S::PErr>;

    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        DoubleParser::new(&self.first, &self.second).parse(baggage, ctx).map(|(_, snd)| snd)
    }

}

impl<'a, E: ParserErr> ParserErr for &'a E {}
impl<'a, T: Parser> Parser for &'a T {
    type Output = T::Output;
    type PErr = T::PErr;
    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<T::Output, T::PErr> {
        T::parse(self, baggage, ctx)
    }
}

use std::fmt::Debug;
pub(crate) trait Parser: Debug {
    // TODO(mike): Probably need to add require a label name
    type Output: Debug;
    type PErr: ParserErr + Debug;
    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr>;
    fn discard_then<P: Parser>(self, snd: P) -> DiscardThenParser<Self, P> 
        where Self: Sized
    {
        DiscardThenParser::new(self, snd)
    }

    /// Runs the parser on given input. Useful for small-scale testing
    fn run_parser(&self, inp: &str) -> Result<Self::Output, Self::PErr> {
        let mut ctx = ParsingContext::new(inp);
        let baggage = ParsingBaggage::init();
        self.parse(&baggage, &mut ctx)
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

    pub fn advance_one(&mut self) -> &mut Self {
        self.col += 1;
        self.index += 1;
        self.cursor = &self.cursor[1..];
        self
    }

    pub fn new<T>(input: &'inp T) -> Self
        where T: AsRef<str> + ?Sized
    {
        let kw = vec!["let", "let", "for", "def", "if", "else", "bool", "()", "f32", "f64","i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "&StaticString"];
        let keywords: HashSet<&'static str> = kw.into_iter().collect();
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
        self.eat_until_cond(&|c| c.is_whitespace())
    }

    /// Eats a whole type definition. This is non-trivial because we want
    /// to capture also generic types and simple types
    /// For example: if we use sepBy(TypeParser, CharParser(',')) on
    /// Basic1, Basic2, Basic3 we would like to get ["Basic1", "Basic2", "Basic3"],
    /// so the type eaten by the TypeParser is "Basic1", then "Basic2", then "Basic3"
    /// but if we want to parse Gen<Basic1, Basic2, Basic3> what we want the parser to eat
    /// is "Gen<Basic1, Basic2, Basic3>", since those commas are part of the type definition
    pub fn eat_type_definition(&mut self) -> Option<&str> {
        // TODO(mike): return errors instead of None, since there are multiple ways the input can be ill-formed
        
        // Tactic: Eat based on the angle brackets. When we reach a '-', a ')' or a '=' we know
        // the type signature must be done. However, when we reach a comma we stop eating if and only if
        // our stack of angle brackets is empty, otherwise we keep going. 
        // If somehow the angle brackets don't match and we have consumed all input or must stop,
        // then we return None, in which case the type parser should issue an angle bracket mismatch err
        // or we found a newline or an illicit character

        let should_end = |c: char| c == '-' || c == '=' || c == ')' || c == '{';
        let allowed_char = |c: char| c == '&' || c == '*' || (c.is_alphanumeric() && c.is_ascii()) || c == ' ' || c == '<' || c == '>' || c == ',' || c == '_';
        let mut advanced = 0;
        let mut reached_end = false;
        let mut brackets = VecDeque::new();
        for (idx, c) in self.cursor.chars().enumerate() {
            match c {
                ch if should_end(ch) => { 
                    break // If we found an character which is no longer part of the type, we exit
                },
                ',' => {
                    // If the angle bracket stack is empty, it means we need to stop here. Otherwise it means we need to carry on
                    if brackets.is_empty() {
                        self.index += advanced;
                        self.col += advanced;
                        let eaten = &self.cursor[..advanced];
                        self.cursor = &self.cursor[advanced..];
                        return Some(eaten);
                    }
                    advanced = idx;
                    // Otherwise we just go on 
                },
                '\n' | '\t' | '\r' => { return None },
                '<' => {
                    advanced = idx;
                    brackets.push_front('<');
                }
                '>' => {
                    advanced = idx;
                    if brackets.is_empty() {
                        // We found a > without a <
                        return None;
                    }
                    brackets.pop_front();
                }
                ch if !allowed_char(ch) => { return None },
                _ => { advanced = idx;  }
                    

            };
            if idx == self.cursor.len() - 1 {
                reached_end = true;
            }
        }

        let advanced = if reached_end { self.cursor.len() } else { advanced };
        // We reached here, so we must either have consumed all input or found a character which signalled us to stop
        // There still is a chance of having unclosed angle brackets
        if brackets.is_empty() {
            self.index += advanced;
            self.col += advanced;
            let eaten = &self.cursor[..advanced];
            self.cursor = &self.cursor[advanced..];
            Some(eaten)
        } else { None }
    }


    pub fn eat_until_cond(&mut self, cond: &dyn Fn(char) -> bool) -> &str {
        let mut advanced = 0;
        let mut found_last = false;
        for (i, c) in self.cursor.chars().enumerate() {
            if cond(c) {
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


