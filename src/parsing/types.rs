use crate::parsing::combinators::{OptionParser, CharParser, StringParser, StringParseErr, AlternativeParser, SepByParser};
use crate::parsing::{ParsingBaggage, ParserErr, Parser, ParsingContext};
use crate::parsing::literals::{IdentParser, IdentParserErr, IdentParserErrReason};

use crate::ast::types::Ty;
use crate::parsing::literals::IdentParserErrReason::FoundKeyword;

#[derive(Debug)]
pub(crate) enum TypeParserErr {
    UnclosedGeneric(String),
    InvalidGeneric(String),
    ContainsUnicode(String),
    InvalidFormat(String),
    EmptyStr
}

#[derive(Debug)]
struct SimpleType;

impl Parser for SimpleType {
    type Output = String;
    type PErr = IdentParserErr;
    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        match IdentParser.parse(baggage, ctx) {
            Ok(id) => Ok(id.to_string()),
            Err(ier) => {
                match ier.reason {
                    FoundKeyword => Ok(ier.found),
                    _ => Err(ier)
                }
            }
        }
    }
}

impl ParserErr for TypeParserErr {
    fn label(&self) -> String {
        String::from("type")
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Type;

// Corresponding EBNF for types
// Ty -> '&' Ty | '*' Ty | Ident | Ident '<' Generics '>'
// Generics -> Ty (',' Ty)*

// Check the definition of Ty in ast/types.rs if confused
impl Parser for Type {
    type Output = Ty;
    type PErr = TypeParserErr;

    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        use TypeParserErr::*;
        // We opt for a more functional way of declaring the parser. We could do it all 
        // by hand but we can also use known combinators for simplicity

        AlternativeParser::new(vec![&RefTy, &GenericOrSimpleTy]).parse(baggage, ctx).map_err(|_| InvalidFormat("__type parse err__".to_string()))
    }
}


#[derive(Debug)]
struct GenericOrSimpleTy;

impl Parser for GenericOrSimpleTy {
    type Output = Ty;
    type PErr = TypeParserErr;

    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        use TypeParserErr::*;
        match SimpleType.parse(baggage, ctx) {
            Ok(id) => {
                let generics = CharParser('<').discard_then(SepByParser::new(Type, CharParser(','))).then_discard(CharParser('>'));
                let mby_gens = OptionParser::new(generics).parse_to_option(baggage, ctx);
                let id = id.to_string();
                match mby_gens {
                    Some(tys) => Ok(Ty::Generic(id, tys)),
                    None => Ok(Ty::Userdefined(id))
                }
            }
            _ => Err(InvalidFormat("__invalid simple or generic ty format__".to_string()))
        }
    }

}

#[derive(Debug)]
struct RefTy;
impl Parser for RefTy {
    type Output = Ty;
    type PErr = TypeParserErr;

    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        use TypeParserErr::*;

        match ctx.peek_char() {
            Some(c) if c == '&' || c == '*' => {
                ctx.advance_one();
                ctx.eat_ws();
                let res = Type.parse(baggage, ctx);
                if res.is_err() {
                    res
                } else {
                    let ty = res.unwrap();
                    let ty = if c == '&' { Ty::Ref(Box::new(ty)) } else { Ty::Ptr(Box::new(ty)) };
                    Ok(ty)
                }
            },
            _ => Err(InvalidFormat("__not ptr or ref type__".to_string()))
        }
    }
}




