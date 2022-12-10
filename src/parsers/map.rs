//! Mapping parsers.

use crate::{types::ParserOutput, ParseContext, ParseIter, Parser, Result};

pub struct MapRawParser<P, F> {
    parser: P,
    mapper: F,
}

pub struct MapRawParseIter<'parse, P, F>
where
    P: Parser + 'parse,
{
    iter: P::Iter<'parse>,
    mapper: &'parse F,
}

impl<P, F, T> Parser for MapRawParser<P, F>
where
    P: Parser,
    F: Fn(P::RawOutput) -> T,
    T: ParserOutput,
{
    type Output = <T as ParserOutput>::UserType;
    type RawOutput = T;
    type Iter<'parse> = MapRawParseIter<'parse, P, F>
    where
        P: 'parse,
        F: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        let iter = self.parser.parse_iter(context, start)?;
        let mapper = &self.mapper;
        Ok(MapRawParseIter { iter, mapper })
    }
}

impl<'parse, P, F, T> ParseIter<'parse> for MapRawParseIter<'parse, P, F>
where
    P: Parser,
    F: Fn(P::RawOutput) -> T,
{
    type RawOutput = T;

    fn match_end(&self) -> usize {
        self.iter.match_end()
    }

    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> bool {
        self.iter.backtrack(context)
    }

    fn into_raw_output(self) -> T {
        (self.mapper)(self.iter.into_raw_output())
    }
}

#[derive(Clone, Copy)]
pub struct MapParser<P, F> {
    pub(crate) parser: P,
    pub(crate) mapper: F,
}

pub struct MapParseIter<'parse, P, F>
where
    P: Parser + 'parse,
{
    iter: P::Iter<'parse>,
    mapper: &'parse F,
}

impl<P, F> MapParser<P, F> {
    pub fn new(parser: P, mapper: F) -> Self {
        MapParser { parser, mapper }
    }
}

impl<P, F, T> Parser for MapParser<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> T,
{
    type Output = T;
    type RawOutput = (T,);
    type Iter<'parse> = MapParseIter<'parse, P, F>
    where
        P: 'parse,
        F: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        let iter = self.parser.parse_iter(context, start)?;
        let mapper = &self.mapper;
        Ok(MapParseIter { iter, mapper })
    }
}

impl<'parse, P, F, T> ParseIter<'parse> for MapParseIter<'parse, P, F>
where
    P: Parser,
    F: Fn(P::Output) -> T,
{
    type RawOutput = (T,);

    fn match_end(&self) -> usize {
        self.iter.match_end()
    }

    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> bool {
        self.iter.backtrack(context)
    }

    fn into_raw_output(self) -> (T,) {
        let value = (self.mapper)(self.iter.into_raw_output().into_user_type());
        (value,)
    }
}
