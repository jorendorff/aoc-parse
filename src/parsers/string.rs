//! The parser used by the `string()` function.

use crate::{error::Result, ParseIter, Parser};

#[derive(Clone, Copy)]
pub struct StringParser<P> {
    pub(crate) parser: P,
}

pub struct StringParseIter<'parse, P>
where
    P: Parser + 'parse,
{
    source: &'parse str,
    start: usize,
    iter: P::Iter<'parse>,
}

impl<P> Parser for StringParser<P>
where
    P: Parser,
{
    type Output = String;
    type RawOutput = (String,);
    type Iter<'parse> = StringParseIter<'parse, P>
    where
        P: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        let iter = self.parser.parse_iter(source, start)?;
        Ok(StringParseIter {
            source,
            start,
            iter,
        })
    }
}

impl<'parse, P> ParseIter for StringParseIter<'parse, P>
where
    P: Parser,
{
    type RawOutput = (String,);

    fn match_end(&self) -> usize {
        self.iter.match_end()
    }

    fn backtrack(&mut self) -> bool {
        self.iter.backtrack()
    }

    fn take_data(&mut self) -> (String,) {
        let end = self.iter.match_end();
        let value = self.source[self.start..end].to_string();
        (value,)
    }
}
