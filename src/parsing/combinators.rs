use crate::parsing::{ParsingContext, ParserErr, Parser};


struct AttemptParser<P> {
    inside: P,
}

impl<P> AttemptParser<P> {
    pub fn new(inside: P) -> Self {
        Self { inside }
    }
}

impl<P: Parser> Parser for AttemptParser<P> {
    type Output = P::Output;
    type PErr = P::PErr;

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, Self::PErr> {
        let idx_before = ctx.index;
        let cursor_before = ctx.cursor;
        
        let res = self.inside.parse(ctx);
        if res.is_err() {
            ctx.index = idx_before;
            ctx.cursor = cursor_before;
        }
        res
    }
}

#[repr(transparent)]
struct OptionParser<P> {
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
    type Output = Option<P::Output>;
    type PErr = ();

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, ()> {
        let idx_before = ctx.index;
        let cursor_before = ctx.cursor;
       
        match self.inside.parse(ctx) {
            Ok(res) => Ok(Some(res )),
            Err(_) =>  { ctx.index = idx_before; ctx.cursor = cursor_before; Ok(None) }
        }
    }
}


