use crate::parsing::combinators::{StringParser, StringParseErr, AlternativeParser};
use crate::parsing::{ParserErr, Parser, ParsingContext};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PrimitiveType();

impl Parser for PrimitiveType {
    type Output = &'static str;
    type PErr = ();

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let btp = &ctx.baggage.base_type_parser;
        
        // XXX: Look at comment at ParsingBaggage. ParsingContext and ParsingBaggage
        // will be moved to avoid this transmute
        let _btp: &AlternativeParser<&str, StringParseErr> = unsafe { std::mem::transmute(btp) };
        drop(btp);
        btp.parse(ctx)
    }

}




