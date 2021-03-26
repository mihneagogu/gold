use crate::parsing::combinators::{ManyParser, CharParser, StringParser, StringParseErr, AlternativeParser};
use crate::parsing::{ParsingBaggage, ParserErr, Parser, ParsingContext};
use crate::parsing::literals::IdentParser;

use crate::ast::types::Ty;

/// Parses one of the primitive types: u8, i8, bool etc.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PrimitiveType();

#[derive(Debug)]
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
    
    let fst = inp.chars().nth(0).unwrap();

    let apply_to_ty = |res: Result<Ty, TypeParserErr>, on_succ: &dyn Fn(Ty) -> Ty| {
        match res {
            Ok(t) => Ok(on_succ(t)),
            e @ Err(_) => e
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
                // now we need to see that the thing inside the generic brackets is a valid 
                // type or sequence of types
                
                // Get the inside from SomeT< inside >
                let _inside = &inp[idx+1.. inp.len() - 1];

                // TODO(mike): Use sepBy(TypeParser, CharParser(',')) to get the inside
                todo!()

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

        // TODO(mike): better way of capturing the input. We want to parse until we find an = of a
        // declaration or a '-' from "->". We basically want to somehow delimit what is part of the
        // type and what follows after
        let inp = ctx.eat_until_cond(&|c| c == '-' || c == '=');
        println!("input: {}", inp);
        let is_usable = |c: char| c == ' ' || c == ',' || c == '_' || c == '<' || c == '>' || c == '&' || c == '*' || (c.is_ascii() && c.is_alphanumeric());
        let valid_chars = inp.chars().all(|c| is_usable(c));
        let inp_trimmed: String = inp.chars().filter(|c| *c != ' ').collect();
        if !valid_chars || inp_trimmed.len() == 0 {
            return Err(ContainsUnicode(inp.to_string()));
        }
        
        let r = parse_ty(&inp_trimmed);
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




