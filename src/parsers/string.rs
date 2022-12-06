//! The parser used by the `string()` function.

use crate::{error::Result, ParseIter, Parser};

#[derive(Clone, Copy)]
pub struct StringParser<P> {
    pub(crate) parser: P,
}

pub struct StringParseIter<'parse, P>
where
    P: Parser<'parse>,
{
    source: &'parse str,
    start: usize,
    end: usize,
    iter: P::Iter,
}

impl<'parse, P> Parser<'parse> for StringParser<P>
where
    P: Parser<'parse>,
{
    type Output = String;
    type RawOutput = (String,);
    type Iter = StringParseIter<'parse, P>;

    fn parse_iter(&'parse self, source: &'parse str, start: usize) -> Self::Iter {
        StringParseIter {
            source,
            start,
            end: start,
            iter: self.parser.parse_iter(source, start),
        }
    }
}

impl<'parse, P> ParseIter for StringParseIter<'parse, P>
where
    P: Parser<'parse>,
{
    type RawOutput = (String,);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        let r = self.iter.next_parse();
        if let Some(Ok(end)) = r {
            self.end = end;
        }
        r
    }

    fn take_data(&mut self) -> (String,) {
        let value = self.source[self.start..self.end].to_string();
        (value,)
    }
}
