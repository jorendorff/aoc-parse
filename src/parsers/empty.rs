//! Parser that successfully matches the empty string.

use crate::{error::Result, ParseIter, Parser};

#[derive(Clone, Copy)]
pub struct EmptyParser;

impl Parser for EmptyParser {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = EmptyParseIter;

    fn parse_iter(&self, _source: &str, start: usize) -> Result<EmptyParseIter> {
        Ok(EmptyParseIter {
            used: false,
            location: start,
        })
    }
}

pub struct EmptyParseIter {
    used: bool,
    location: usize,
}

impl ParseIter for EmptyParseIter {
    type RawOutput = ();

    fn next_parse(&mut self) -> Option<Result<usize>> {
        if self.used {
            None
        } else {
            self.used = true;
            Some(Ok(self.location))
        }
    }

    fn take_data(&mut self) {}
}

// Used by the `parser!()` macro to implement the empty pattern, `()`.
#[doc(hidden)]
pub fn empty() -> EmptyParser {
    EmptyParser
}
