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

impl ParserErr for NumberParseErr {
    fn label(&self) -> String {
        String::from("number")
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct NumberParser {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IdentParser;

#[derive(Debug)]
pub(crate) enum IdentParserErrReason {
    IllicitChar(char),
    FoundKeyword,
    NoAlphaNum,
    EmptyInp
}

#[derive(Debug)]
pub(crate) struct IdentParserErr {
    pub found: String,
    pub reason: IdentParserErrReason,
    row: usize,
    col: usize
}

impl IdentParserErr {
    fn new(found: &str, reason: IdentParserErrReason /* TODO:row, col */) -> Self {
        Self { found: found.to_string(), reason, row: 0, col: 0 }
    }

}


impl ParserErr for IdentParserErr {
    fn label(&self) -> String {
        String::from("identifier")
    }
}

use IdentParserErrReason::*;
impl Parser for IdentParser {
    type Output = String;
    type PErr = IdentParserErr;


    fn parse(&self, _baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        const BASE: u32 = 10;

        let mut eaten = 0;
        let valid_char = |c: char| c == '_' || (c.is_alphanumeric() && c.is_ascii());
        for (idx, c) in ctx.cursor.chars().enumerate() {
            match c {
                ch if idx == 0 && c.is_digit(BASE) => return Err(IdentParserErr::new(&ctx.cursor[..eaten], IllicitChar(ch))),
                ch if valid_char(ch) => eaten += 1,
                _ => break
            }
        }

        if eaten == 0 {
            // We didn't manage to parse anything useful
            Err(IdentParserErr::new("", EmptyInp))
        } else {
            ctx.col += eaten;
            ctx.index += eaten;
            let eaten_str = &ctx.cursor[..eaten];
            ctx.cursor = &ctx.cursor[eaten..];
            ctx.eat_ws();

            let mut found_alpha = false;
            for c in eaten_str.chars() {
                if c.is_alphabetic() {
                    found_alpha = true;
                    break;
                }
            }

            if found_alpha {
                if ctx.contains_keyword(eaten_str) {
                    Err(IdentParserErr::new(eaten_str, FoundKeyword))
                } else {
                    Ok(eaten_str.to_string())
                }
            } else {
                Err(IdentParserErr::new(eaten_str, NoAlphaNum))
            }

        }
    }

}

impl Parser for NumberParser {
    type Output = i32;
    type PErr = NumberParseErr;

    fn parse(&self, _baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        use NumberParseErr::InvalidNumber;
        // We follow a similar approach to the IdentParser. We can't really
        // eat everything until whitespace, since the ' ' might not necessarily
        // be what we have after a declaration of a number
        
        // TODO(mike): hex numbers
        
        const BASE: u32 = 10;

        let mut eaten = 0;
        let mut is_neg = false;
        for (idx, c) in ctx.cursor.chars().enumerate() {
            match c {
                '-' if idx == 0 => { is_neg = true; eaten += 1 },
                ch if !ch.is_digit(BASE) => break,
                _ => eaten += 1,
            };
            
        }

        if eaten == 0 || (eaten == 1 && is_neg) {
            // We may have no input or the string "-"
            return Err(NumberParseErr::EmptyStr);
        }

        let inp = &ctx.cursor[..eaten];
        ctx.advance_many(eaten);
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


