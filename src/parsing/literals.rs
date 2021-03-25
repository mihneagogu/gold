use std::num::IntErrorKind;
use super::ParsingContext;
use crate::parsing::{Parser, ParserErr, ParsingBaggage};


#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ParseNumData(i32, i32, String);

#[derive(Debug, PartialEq)]
pub(crate) enum NumberParseErr {
    InvalidNumber(ParseNumData),
    PosOverflow(ParseNumData),
    NegOverflow(ParseNumData),
    EmptyStr,
}

impl ParserErr for NumberParseErr {}

#[derive(Clone, Copy, Debug)]
pub(crate) struct NumberParser {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IdentParser{}

#[derive(Debug)]
pub(crate) enum IdentParserErrReason {
    IllicitChar(char),
    FoundKeyword(String),
    NoAlphaNum
}

#[derive(Debug)]
pub(crate) struct IdentParserErr {
    found: String,
    reason: IdentParserErrReason,
    row: usize,
    col: usize
}

impl IdentParserErr {
    fn new(found: &str, reason: IdentParserErrReason /* TODO:row, col */) -> Self {
        Self { found: found.to_string(), reason, row: 0, col: 0 }
    }
}

impl ParserErr for IdentParserErr {}

use IdentParserErrReason::*;
impl Parser for IdentParser {
    type Output = String;
    type PErr = IdentParserErr;


    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let inp = ctx.eat_until_ws();
        // @MAYBE(mike): could use a transmute to avoid the allocation, but it's not really that important
        let _inp = inp.to_string();
        let inp: &str = &_inp;
        if ctx.contains_keyword(inp) {
            return Err(IdentParserErr::new(inp, FoundKeyword(inp.to_string())));
        }
        let first = inp.chars().nth(0).unwrap();
        let first_is_ok = (first.is_alphabetic() && first.is_ascii()) || first == '_';
        if !first_is_ok {
            return Err(IdentParserErr::new(inp, IllicitChar(first)));
        }
        let mut alphanum_count = 0;
        for c in inp.chars() {
            if c.is_alphanumeric() && c.is_ascii() {
                alphanum_count += 1;
            } else if c != '_' {
                return Err(IdentParserErr::new(inp, IllicitChar(c)));
            }
        }
        
        if alphanum_count == 0 {
            Err(IdentParserErr::new(inp, NoAlphaNum))
        } else {
            Ok(inp.to_string())
        }
    }
}

impl Parser for NumberParser {
    type Output = i32;
    type PErr = NumberParseErr;

    fn parse(&self, _baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        use NumberParseErr::InvalidNumber;

        let inp = ctx.eat_until_ws();
        match inp.parse::<i32>() {
            Ok(n) => { ctx.eat_ws() ; Ok(n) }
            Err(e) => match e.kind() {
                IntErrorKind::InvalidDigit => Err(InvalidNumber(ParseNumData(0, 0, inp.to_string()))),
                IntErrorKind::PosOverflow => Err(NumberParseErr::PosOverflow(ParseNumData(0, 0, inp.to_string()))),
                IntErrorKind::NegOverflow => Err(NumberParseErr::NegOverflow(ParseNumData(0, 0, inp.to_string()))),
                IntErrorKind::Empty => Err(NumberParseErr::EmptyStr),
                _ => unreachable!()
            }
        }
    }
     
}


