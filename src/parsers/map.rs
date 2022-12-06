//! Mapping parsers.

use crate::{error::Result, types::ParserOutput, ParseIter, Parser};

pub struct MapRawParser<P, F> {
    parser: P,
    mapper: F,
}

pub struct MapRawParseIter<'parse, P, F>
where
    P: Parser<'parse>,
{
    iter: P::Iter,
    mapper: &'parse F,
}

impl<'parse, P, F, T> Parser<'parse> for MapRawParser<P, F>
where
    P: Parser<'parse>,
    F: Fn(P::RawOutput) -> T,
    F: 'parse,
    T: ParserOutput,
{
    type Output = <T as ParserOutput>::UserType;
    type RawOutput = T;
    type Iter = MapRawParseIter<'parse, P, F>;

    fn parse_iter<'source>(&'parse self, source: &'source str, start: usize) -> Self::Iter
    where
        'source: 'parse,
    {
        MapRawParseIter {
            iter: self.parser.parse_iter(source, start),
            mapper: &self.mapper,
        }
    }
}

impl<'parse, P, F, T> ParseIter for MapRawParseIter<'parse, P, F>
where
    P: Parser<'parse>,
    F: Fn(P::RawOutput) -> T,
{
    type RawOutput = T;

    fn next_parse(&mut self) -> Option<Result<usize>> {
        self.iter.next_parse()
    }

    fn take_data(&mut self) -> T {
        (self.mapper)(self.iter.take_data())
    }
}

#[derive(Clone, Copy)]
pub struct MapParser<P, F> {
    pub(crate) parser: P,
    pub(crate) mapper: F,
}

pub struct MapParseIter<'parse, P, F>
where
    P: Parser<'parse>,
{
    iter: P::Iter,
    mapper: &'parse F,
}

impl<P, F> MapParser<P, F> {
    pub fn new(parser: P, mapper: F) -> Self {
        MapParser { parser, mapper }
    }
}

impl<'parse, P, F, T> Parser<'parse> for MapParser<P, F>
where
    P: Parser<'parse>,
    F: Fn(P::Output) -> T,
    F: 'parse,
{
    type Output = T;
    type RawOutput = (T,);
    type Iter = MapParseIter<'parse, P, F>;

    fn parse_iter<'source>(&'parse self, source: &'source str, start: usize) -> Self::Iter
    where
        'source: 'parse,
    {
        MapParseIter {
            iter: self.parser.parse_iter(source, start),
            mapper: &self.mapper,
        }
    }
}

impl<'parse, P, F, T> ParseIter for MapParseIter<'parse, P, F>
where
    P: Parser<'parse>,
    F: Fn(P::Output) -> T,
{
    type RawOutput = (T,);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        self.iter.next_parse()
    }

    fn take_data(&mut self) -> (T,) {
        let value = (self.mapper)(self.iter.take_data().into_user_type());
        (value,)
    }
}
