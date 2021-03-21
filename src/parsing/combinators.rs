use crate::parsing::{ParsingContext, ParserErr, Parser};

// TODO(mike): Add labels to all parsers, so that the alternative parser
// can have its own label like this:
// "Expected one of <label1>, <label2> ... "

/// Alternative parser. Similar to
/// parser1 <|> parser2 <|> parser3.
/// This means: perform parser1, if successful exit,  otherwise parser2.
/// If parser2 is successful, exit, else perform parser3.
/// Disclaimer: If any parser consumes input, it won't be rolled back by default
/// If we do want the input to be rolled back, we use an attempt() arround the parser
/// (aka AttemptParser::new)
pub(crate) struct AlternativeParser<'ps, O, E> {
    variants: Vec<&'ps dyn Parser<Output = O, PErr = E>>
}

impl<'ps, O, E> AlternativeParser<'ps, O, E> {
    pub fn new(variants: Vec<&'ps dyn Parser<Output = O, PErr = E>>) -> Self {
        Self { variants }
    }
}

impl<'ps, O: std::fmt::Debug, E: ParserErr + std::fmt::Debug> Parser for AlternativeParser<'ps, O, E> {
    type Output = O;
    // TODO(mike): This isn't right, we need a custom error type.
    type PErr = ();
    fn parse (&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let mut res = None;
        // Perform all parsers until we succeed or we run out of things to do
        for p in &self.variants {
            let r = p.parse(ctx);
            println!("parser from attempt: {:?}", r);
            match r {
                Ok(o) => { res = Some(o); break; }
                _ => ()
            }
        }
        if let Some(o) = res {
            Ok(o)
        } else {
            Err( () ) // TODO(mike): Change this to actual error type
        }
    }

}




/// A parser which attempts the parser inside, rolling back the input
/// if it fails.
#[derive(Debug)]
pub(crate) struct AttemptParser<P> {
    inside: P,
}

/// Parses a string exactly equal to EXPECTED (beware, case-sensitive)
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct StringParser {
    expected: &'static str
}

/// Parses exactly the char inside
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CharParser(char);

impl CharParser {
    #[inline(always)]
    pub fn new(ch: char) -> Self {
        Self(ch)
    }
}

#[derive(Debug)]
pub(crate) enum CharParseErr {
    CharMismatch(char, char), // expected, found
    Empty
}
impl ParserErr for CharParseErr {}

impl Parser for CharParser {
    type Output = char;
    type PErr = CharParseErr;

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        match ctx.peek_char() {
            None => Err(CharParseErr::Empty),
            Some(ch) => {
                let res = if ch == self.0 {
                    Ok(ch)
                } else {
                    Err(CharParseErr::CharMismatch(self.0, ch))
                };
                ctx.index += 1;
                if ch == '\n' {
                   ctx.row += 1; ctx.col = 1;
                } else { 
                    ctx.col += 1;
                }
                res
            }
        }
    }
}

impl StringParser {
    pub fn new(expected: &'static str) -> Self {
        Self { expected }
    }
}

#[derive(Debug)]
pub(crate) enum StringParseErr {
    StringMismatch(&'static str, String),
    NotEnoughInput // We couldn't read that many chars, but we have STRING
}

impl ParserErr for StringParseErr {}

impl Parser for StringParser {
    type Output = &'static str;
    type PErr = StringParseErr; // Char mismatch err
    
    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let inp = ctx.eat_many(self.expected.len());
        let res = match inp {
            Some(i) if i == self.expected => Ok(self.expected),
            Some(i) => Err(StringParseErr::StringMismatch(self.expected, i.to_string())),
            None => Err(StringParseErr::NotEnoughInput)
        };
        ctx.eat_ws();
        res
    }
}

impl<P> AttemptParser<P> {
    pub fn new(inside: P) -> Self {
        Self { inside }
    }
}

impl<P: Parser> Parser for AttemptParser<P> {
    type Output = P::Output;
    // The attempt parser itself can never fail, but it might not find the thing
    // we are trying to parse
    // TODO(mike): this is wrong, it shpuld still behave like P, but just rollback input if it
    // doesn't work
    type PErr = P::PErr;

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let state_before = ctx.current_state();
        
        let res = self.inside.parse(ctx);
        
        // If the operation didn't work, just roll back the parser as if nothing had happened
        match &res {
            Ok(_) => { ctx.eat_ws(); },
            Err(_) => ctx.roll_back_op(state_before)
        };
        res
    }
}

/// Possibly parses what INSIDE parses, returning if it was successful or not.
/// If INSIDE eats input but fails, the parser will rollback the operation.
/// OptionParser never fails, but it might yield an Ok(None), in which case 
/// we know that the operation of INSIDE was not successful.
#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct OptionParser<P> {
    inside: P,
}

impl<P: Parser> OptionParser<P> {
    pub fn new(inside: P) -> Self {
        Self { inside }
    }

    fn parse_to_option(&self, ctx: &mut ParsingContext) -> Option<P::Output> {
        self.parse(ctx).unwrap()
    }
}


impl ParserErr for () { }

impl<P: Parser> Parser for OptionParser<P> {
    // The OptionParser never fails, it's just that it might not find 
    // the thing we optionally want, which would result in an Ok(None)
    type Output = Option<P::Output>;
    type PErr = ();

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, ()> {
        let state_before = ctx.current_state();
       
        // Try to parse it, and if we can't, just pretend it didn't happen
        match self.inside.parse(ctx) {
            Ok(res) => { ctx.eat_ws(); Ok(Some(res)) }
            Err(_) =>  { ctx.roll_back_op(state_before) ; Ok(None) }
        }
    }
}


