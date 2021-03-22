use std::num::IntErrorKind;
use super::ParsingContext;
use crate::parsing::{Parser, ParserErr};


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

#[derive(Clone, Copy)]
pub(crate) struct NumberParser {}

impl Parser for NumberParser {
    type Output = i32;
    type PErr = NumberParseErr;

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
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

pub(crate) struct IdentParser {}

