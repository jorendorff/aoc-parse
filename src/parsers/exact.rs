//! Parser that matches a particular exact string.

use crate::{error::Result, ParseError, ParseIter, Parser};

#[derive(Clone, Copy)]
pub struct ExactParser {
    s: &'static str,
}

pub struct ExactParseIter {
    end: usize,
    done: bool,
}

impl Parser for ExactParser {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = ExactParseIter;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<ExactParseIter> {
        if source[start..].starts_with(self.s) {
            Ok(ExactParseIter {
                end: start + self.s.len(),
                done: false,
            })
        } else {
            Err(ParseError::new_expected(source, start, self.s))
        }
    }
}

impl ParseIter for ExactParseIter {
    type RawOutput = ();

    fn next_parse(&mut self) -> Option<Result<usize>> {
        if self.done {
            None
        } else {
            self.done = true;
            Some(Ok(self.end))
        }
    }

    fn take_data(&mut self) {}
}

// Used by the `parser!()` macro to implement string-literal syntax.
#[doc(hidden)]
pub fn exact(s: &'static str) -> ExactParser {
    ExactParser { s }
}
