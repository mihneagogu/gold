use crate::parsing::combinators::{OptionParser, CharParser, StringParser, StringParseErr, AlternativeParser, SepByParser};
use crate::parsing::{ParsingBaggage, ParserErr, Parser, ParsingContext};
use crate::parsing::literals::IdentParser;

use crate::ast::types::Ty;

/// Parses one of the primitive types: u8, i8, bool etc.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PrimitiveType();

#[derive(Debug)]
pub(crate) enum TypeParserErr {
    UnclosedGeneric(String),
    InvalidGeneric(String),
    ContainsUnicode(String),
    InvalidFormat(String), // i*nt or i&n&t is not faild
    EmptyStr
}

impl ParserErr for TypeParserErr {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Type;

fn parse_ty(inp: &str) -> Result<Ty, TypeParserErr> {
    let fst = inp.chars().nth(0).unwrap();

    let apply_to_ty = |res: Result<Ty, TypeParserErr>, on_succ: &dyn Fn(Ty) -> Ty| {
        match res {
            Ok(t) => Ok(on_succ(t)),
            err => err
        }
    };
    let box_ty = |input: &str, is_ref: bool| {
        let f: &dyn Fn(Ty) -> Ty = if is_ref { &|t| Ty::Ref(Box::new(t)) } else { &|t| Ty::Ptr(Box::new(t)) };
        apply_to_ty(parse_ty(&input[1..]), f)
    };

    match fst {
        '&' => return box_ty(inp, true),
        '*' => return box_ty(inp, false), 
        _ => (),
    };

    // Not a pointer type or reference type, just go on.
    // It might contain generics though
    let gen_start = inp.find('<');

    // Returns whether ID is a valid simple ident, which means the first char
    // is not a number, we have at least a letter, and there are no generics or pointer
    // or reference chars inside
    let is_valid_simple_ident = |id: &str| {
        let mut found_alpha = false;
        for (i, c) in id.chars().enumerate() {
            match c {
                ch if ch.is_numeric() => if i == 0 { return false },
                ',' | ' ' | '*' | '>' | '<' | '&' => return false,
                ch if ch.is_alphabetic() => { found_alpha = true },
                _ => unreachable!()
            }
        }
        if !found_alpha { false } else { true }
    };
    match gen_start {
        None => if is_valid_simple_ident(inp) { Ok(Ty::Userdefined(inp.to_string())) } else { Err(TypeParserErr::InvalidFormat(inp.to_string())) }
        Some(idx) => {
            if inp.chars().last().unwrap() != '>' {
                Err(TypeParserErr::UnclosedGeneric(inp.to_string()))
            } else {
                // Now we need to see that the thing inside the generic brackets is a valid 
                // type or sequence of types
                
                
                let name = &inp[0..idx]; // The name of the type
                let inside = &inp[idx+1.. inp.len() - 1]; // The generics args

                // We have a generic type, now parse the types inside the < > 
                let generics = SepByParser::new(Type, CharParser(','));
            
                match generics.run_parser(inside) {
                    Ok(tys) => { 
                        Ok(Ty::Generic(name.to_string(), tys)) 
                    },
                    Err(_) => Err(TypeParserErr::InvalidGeneric(inside.to_string()))
                }
            }
        }
    }
}

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
        match IdentParser.parse(baggage, ctx) {
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




impl Parser for PrimitiveType {
    type Output = &'static str;
    type PErr = ();

    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let btp = &baggage.base_type_parser;
        btp.parse(baggage, ctx)
    }

}




