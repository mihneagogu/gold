use crate::parsing::{ParsingContext, ParserErr, Parser};


#[derive(Debug)]
pub(crate) struct AttemptParser<P> {
    inside: P,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct StringParser {
    expected: &'static str
}

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


