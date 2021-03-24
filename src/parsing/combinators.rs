use crate::parsing::{ParsingContext, ParserErr, Parser};

// TODO(mike): Add labels to all parsers, so that the alternative parser
// can have its own label like this:
// "Expected one of <label1>, <label2> ... "

/// This parser parses 0 or more instances of INSIDE.
/// This is done by repeatedly applying the parser inside. When it fails, however,
/// it will not consume input (since we might be parsing 0 instances of INSIDE).
/// This parser is slightly different from equivalent versions of other combinators from other
/// parsing libraries.
/// Take Parsley Scala for example: 
/// consider the parser (the <~> takes two parsers, applies them and stores the result in a tuple):
/// val asd: Parser[(List[String], String)] = many(stringLift("123")) <~> stringLift("12")
/// if you run the parser on this input: "12"
/// asd.runParser("12") it will tell you that it was expecting "123" but found 12. 
/// It started eating the '1' and '2' since it was partially matching the very first parser.
/// However, ManyParser<P> in this case will just return an empty vec, and then a "12". 
/// It would find 0 instances of "123" but it will find afterwards the "12" we wanted it to find.
pub(crate) struct ManyParser<P> {
    inside: P
}

impl<P: Parser> Parser for ManyParser<P> {
    type Output = Vec<P::Output>;
    // The ManyParser always succeeds, since it might simply parse 0 instances
    // of inside
    type PErr = (); 

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let mut res = Vec::new();
        loop {
            // @MAYBE(mike): If the parsing fails just rollback and exit ?
            let before = ctx.current_state(); 
            match self.inside.parse(ctx) {
                Ok(r) => res.push(r),
                Err(_) => { ctx.roll_back_op(before); break; }
            }
        }
        Ok(res)
    }
}

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

impl<'ps, O, E: ParserErr> Parser for AlternativeParser<'ps, O, E> {
    type Output = O;
    // TODO(mike): This isn't right, we need a custom error type.
    type PErr = ();
    fn parse (&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let mut res = None;
        // Perform all parsers until we succeed or we run out of things to do
        for p in &self.variants {
            let r = p.parse(ctx);
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


#[derive(Debug)]
pub(crate) struct StringParser {
    expected: &'static str
}

impl StringParser {
    pub fn new(expected: &'static str) -> Self {
        Self { expected }
    }
}

impl Parser for StringParser {
    type Output = &'static str;
    type PErr = StringParseErr;
    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let res = AttemptParser::new(RawStringParser::new(self.expected)).parse(ctx);
        if res.is_ok() {
            if ctx.keywords.contains(self.expected) {
                if let Some(next) = ctx.peek_char() {
                    if next.is_alphanumeric() || next == '_' {
                        // We wanted an keyword, but we actually found an identifier (for example
                        // bools instead of the bool keyword)
                        let mut found = self.expected.to_string();
                        found.push(next);
                        return Err(StringParseErr::StringMismatch(self.expected, found));
                    } else {
                        ctx.eat_ws();
                    }
                }
            } else {
                ctx.eat_ws();
            }
        }
        res
    }

}


/// A parser which attempts the parser inside, rolling back the input
/// if it fails.
#[derive(Debug)]
pub(crate) struct AttemptParser<P> {
    inside: P,
}

/// Parses a string exactly equal to EXPECTED and DOES NOT consume whitespace after (beware, case-sensitive)
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct RawStringParser {
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
                if res.is_ok() { ctx.eat_ws(); }
                res
            }
        }
    }
}

impl RawStringParser {
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

impl Parser for RawStringParser {
    type Output = &'static str;
    type PErr = StringParseErr; // Char mismatch err

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let inp = ctx.eat_many(self.expected.len());
        let res = match inp {
            Some(i) if i == self.expected => Ok(self.expected),
            Some(i) => Err(StringParseErr::StringMismatch(self.expected, i.to_string())),
            None => Err(StringParseErr::NotEnoughInput)
        };
        // ctx.eat_ws();
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
            Ok(_) => (),
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


