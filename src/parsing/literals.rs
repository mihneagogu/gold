use std::num::IntErrorKind;
use super::ParsingContext;
use crate::parsing::ParserErr;



#[derive(Debug, PartialEq)]
pub(crate) enum NumberParseErr {
    InvalidNumber(ParseNumData),
    PosOverflow(ParseNumData),
    NegOverflow(ParseNumData),
    EmptyStr,
}

impl ParserErr for NumberParseErr {}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ParseNumData(i32, i32, String);

#[warn(type_alias_bounds)]
pub(crate) type Parser<T, E /*: ParserErr*/> = dyn Fn(&mut ParsingContext) -> Result<T, E>;

pub fn option_parse<T, E: ParserErr>(p: &Parser<T, E>, ctx: &mut ParsingContext) -> Option<T> {
    p(ctx).map_or(Option::default(), |res| Some(res))
}

pub(crate) fn parse_number(ctx: &mut ParsingContext) -> Result<i32, NumberParseErr> {
    use NumberParseErr::InvalidNumber;

    let inp = ctx.eat_until_ws();
    match inp.parse::<i32>() {
        Ok(n) => Ok(n),
        Err(e) => match e.kind() {
             IntErrorKind::InvalidDigit => Err(InvalidNumber(ParseNumData(0, 0, inp.to_string()))),
             IntErrorKind::PosOverflow => Err(NumberParseErr::PosOverflow(ParseNumData(0, 0, inp.to_string()))),
             IntErrorKind::NegOverflow => Err(NumberParseErr::NegOverflow(ParseNumData(0, 0, inp.to_string()))),
             IntErrorKind::Empty => Err(NumberParseErr::EmptyStr),
            _ => unreachable!()
        }
    }
}


