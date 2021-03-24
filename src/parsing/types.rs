use crate::parsing::combinators::{StringParser, StringParseErr, AlternativeParser};
use crate::parsing::{ParserErr, Parser, ParsingContext};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PrimitiveType();

impl Parser for PrimitiveType {
    type Output = &'static str;
    type PErr = ();

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let btp = &ctx.baggage.base_type_parser;
        // SAFETY: The only problem here is that we are trying to use the parser which is located
        // inside the Parsing Context. We are using this so that we don't need to reallocate
        // the AlternativeParser every time we parse an ident, we use the existing one inside the
        // Parsing Context. However, the AlternativeParser wants to operate on the context itself,
        // which makes the compiler issue a double-borrow of &mut Parsing Context.
        // What it doesn't understand is that the base_type_parser itself has no relation 
        // to the context, but we do need to have it attached to it somehow (either in the Parser
        // trait as a parameter or to the ParsingContext itself).
        //The transmute is fine since we definitely know that it lives long enough.
        // MAYBE(mike): Pass the ParsingBaggage as a param, so it is not tied to the
        // ParsingContext and we can avoid this ugly transmukte
        let _btp: &AlternativeParser<&str, StringParseErr> = unsafe { std::mem::transmute(btp) };
        drop(btp);
        _btp.parse(ctx)
    }

}




