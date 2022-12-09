//! Parser that successfully matches the empty string.

use crate::{error::Result, ParseIter, Parser};

#[derive(Clone, Copy)]
pub struct EmptyParser;

impl Parser for EmptyParser {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = EmptyParseIter;

    fn parse_iter(&self, _source: &str, start: usize) -> Result<EmptyParseIter> {
        Ok(EmptyParseIter { location: start })
    }
}

pub struct EmptyParseIter {
    location: usize,
}

impl ParseIter for EmptyParseIter {
    type RawOutput = ();
    fn match_end(&self) -> usize {
        self.location
    }
    fn backtrack(&mut self) -> bool {
        false
    }
    fn into_raw_output(self) -> Self::RawOutput {}
}

// Used by the `parser!()` macro to implement the empty pattern, `()`.
#[doc(hidden)]
pub fn empty() -> EmptyParser {
    EmptyParser
}
