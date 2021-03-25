use crate::parsing::combinators::{StringParser, StringParseErr, AlternativeParser};
use crate::parsing::{ParsingBaggage, ParserErr, Parser, ParsingContext};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PrimitiveType();

impl Parser for PrimitiveType {
    type Output = &'static str;
    type PErr = ();

    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let btp = &baggage.base_type_parser;
        btp.parse(baggage, ctx)
    }

}




