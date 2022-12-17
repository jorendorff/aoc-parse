//! Parser that successfully matches the empty string.

use crate::{ParseContext, ParseIter, Parser, Reported, Result};

#[derive(Clone, Copy)]
pub struct EmptyParser;

impl Parser for EmptyParser {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = EmptyParseIter;

    fn parse_iter<'parse>(
        &self,
        _context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<EmptyParseIter, Reported> {
        Ok(EmptyParseIter { location: start })
    }
}

pub struct EmptyParseIter {
    location: usize,
}

impl<'parse> ParseIter<'parse> for EmptyParseIter {
    type RawOutput = ();
    fn match_end(&self) -> usize {
        self.location
    }
    fn backtrack(&mut self, _context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        Err(Reported)
    }
    fn into_raw_output(self) -> Self::RawOutput {}
}

/// Used by the `parser!()` macro to implement the empty pattern, `()`.
#[doc(hidden)]
pub fn empty() -> EmptyParser {
    EmptyParser
}
