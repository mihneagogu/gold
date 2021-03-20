use std::marker::PhantomData;

use crate::parsing::{ParsingContext, ParserErr, Parser};

#[warn(type_alias_bounds)]
pub(crate) type ParserFun<T, E /*: ParserErr*/> = dyn Fn(&mut ParsingContext) -> Result<T, E>;

pub fn option_parse<T, E: ParserErr>(p: &ParserFun<T, E>, ctx: &mut ParsingContext) -> Option<T> {
    p(ctx).map_or(Option::default(), |res| Some(res))
}

struct OptionParser<P, E> {
    inside: P,
    _data: PhantomData<E> 
}

impl<P: Parser<E>, E: ParserErr> OptionParser<P, E> {
    pub fn new(inside: P) -> Self {
        Self { inside, _data: PhantomData }
    }

    fn parse_to_option(&self, ctx: &mut ParsingContext) -> Option<P::Output> {
        self.parse(ctx).unwrap()
    }
}


impl ParserErr for () { }

impl<P: Parser<E>, E: ParserErr> Parser<()> for OptionParser<P, E> {
    type Output = Option<P::Output>;

    fn parse(&self, ctx: &mut ParsingContext) -> Result<Self::Output, ()> {
        let idx_before = ctx.index;
        let cursor_before = ctx.cursor;
       
        match self.inside.parse(ctx) {
            Ok(res) => Ok(Some(res )),
            Err(_) =>  { ctx.index = idx_before; ctx.cursor = cursor_before; Ok(None) }
        }
    }
}


