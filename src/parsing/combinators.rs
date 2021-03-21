use crate::parsing::{ParsingContext, ParserErr, Parser};


pub(crate) struct AttemptParser<P> {
    inside: P,
}

impl<P> AttemptParser<P> {
    pub fn new(inside: P) -> Self {
        Self { inside }
    }
}

impl<P: Parser> Parser for AttemptParser<P> {
    type Output = Option<P::Output>;
    // The attempt parser itself can never fail, but it might not find the thing
    // we are trying to parse
    type PErr = (); 

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, ()> {
        let state_before = ctx.current_state();
        
        let res = self.inside.parse(ctx);
        
        // If the operation didn't work, just roll back the parser as if nothing had happened
        match res {
            Ok(res) => { ctx.eat_ws(); Ok(Some(res)) }
            Err(_) => { ctx.roll_back_op(state_before) ; Ok(None) }
        }
    }
}

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


