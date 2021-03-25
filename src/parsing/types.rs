use crate::parsing::combinators::{ManyParser, CharParser, StringParser, StringParseErr, AlternativeParser};
use crate::parsing::{ParsingBaggage, ParserErr, Parser, ParsingContext};
use crate::parsing::literals::IdentParser;

use crate::ast::types::Ty;

/// Parses one of the primitive types: u8, i8, bool etc.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PrimitiveType();

pub(crate) enum TypeParserErr {
    UnclosedGeneric(String),
    ContainsUnicode(String),
    InvalidFormat(String), // i*nt or i&n&t is not faild
}

impl ParserErr for TypeParserErr {}

/// A userdefined type is just another identifier.
type UserType = IdentParser;

#[derive(Debug)]
pub(crate) struct Type();

fn parse_ty(inp: &str) -> Result<Ty, TypeParserErr> {
    // TODO: Change recursive approach to iterative. Sadly Rust
    // doesn't provide tail call optimisation so we would like to not use recursion,
    // although if the stack overflows from prasing a type, it means you're probably doing
    // something very wrong
    let fst = *inp.chars().peekable().peek().unwrap();

    let apply_to_ty = |res: Result<Ty, TypeParserErr>, onSucc: &dyn Fn(Ty) -> Ty| {
        match res {
            Ok(t) => Ok(onSucc(t)),
            e @ Err(_) => e
        }
    };
    let box_ty = |input: &str, is_ref: bool| {
        let f: &dyn Fn(Ty) -> Ty = if is_ref { &|t| Ty::Ref(Box::new(t)) } else { &|t| Ty::Ptr(Box::new(t)) };
        apply_to_ty(parse_ty(&inp[1..]), f)
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
                '*' | '>' | '<' | '&' => return false,
                ch if ch.is_alphabetic() => { found_alpha = true }
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
                // now we need to see that the thing inside the generic brackets is a valid 
                // type or sequence of types
                let many_tys = ManyParser::new(Type().then_discard(CharParser(',')));
                
                // Get the inside from SomeT< inside >
                let inside = &inp[idx.. inp.len() - 1];
                let inside = many_tys.run_parser(inside);
                if inside.is_err() {
                    Err(TypeParserErr::InvalidFormat(inp.to_string()))
                } else {
                    Ok(unimplemented!())
                }
            }
        }
    }
}

// Check the definition of Ty in ast/types.rs if confused

impl Parser for Type {
    type Output = Ty;
    type PErr = TypeParserErr;

    fn parse(&self, baggage: &ParsingBaggage, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        use TypeParserErr::*;

        // We are going hardcore here. We eat everything until the next whitespace then try 
        // to figure out whatever the type is
        let inp = ctx.eat_until_ws();
        let is_usable = |c: char| c == '_' || c == '<' || c == '>' || c == '&' || c == '*' || (c.is_ascii() && c.is_alphanumeric());
        let valid_chars = inp.chars().all(|c| is_usable(c));
        if !valid_chars {
            return Err(ContainsUnicode(inp.to_string()));
        }
        
        let r = parse_ty(inp);
        if r.is_ok() {
            ctx.eat_ws();
        }
        r
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




