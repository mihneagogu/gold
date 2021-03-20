use std::num::IntErrorKind;
use crate::parsing::ParserErr;



#[derive(Debug, PartialEq)]
pub(crate) enum NumberParseErr {
    InvalidNumber(ParseNumData),
    PosOverflow(ParseNumData),
    NegOverflow(ParseNumData),
}

impl ParserErr for NumberParseErr {}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ParseNumData(i32, i32, String);

#[warn(type_alias_bounds)]
pub(crate) type Parser<T, E /*: ParserErr*/> = dyn Fn(&str) -> Result<T, E>;

pub fn option_parse<T, E: ParserErr>(p: &Parser<T, E>, inp: &str) -> Option<T> {
    p(inp).map_or(Option::default(), |res| Some(res))
}

pub(crate) fn parse_number(inp: &str) -> Result<i32, NumberParseErr> {
    use NumberParseErr::InvalidNumber;

    match inp.parse::<i32>() {
        Ok(n) => Ok(n),
        Err(e) => match e.kind() {
            IntErrorKind::InvalidDigit => Err(InvalidNumber(ParseNumData(0, 0, inp.to_string()))),
            IntErrorKind::PosOverflow => Err(NumberParseErr::PosOverflow(ParseNumData(0, 0, inp.to_string()))),
            IntErrorKind::NegOverflow => Err(NumberParseErr::NegOverflow(ParseNumData(0, 0, inp.to_string()))),
            _ => unreachable!()
        }
    }
}


