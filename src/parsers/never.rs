//! Parser that never matches anything.

use crate::{error::Result, types::Never, ParseError, ParseIter, Parser};

pub struct NeverParser;

pub struct NeverParseIter<'source> {
    source: &'source str,
    start: usize,
}

impl<'parse, 'source> Parser<'parse, 'source> for NeverParser {
    type Output = Never;
    type RawOutput = Never;
    type Iter = NeverParseIter<'source>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> NeverParseIter<'source> {
        NeverParseIter { source, start }
    }
}

impl<'source> ParseIter for NeverParseIter<'source> {
    type RawOutput = Never;

    fn next_parse(&mut self) -> Option<Result<usize>> {
        Some(Err(ParseError::new_cannot_match(self.source, self.start)))
    }

    fn take_data(&mut self) -> Never {
        unreachable!("never matches");
    }
}
