//! Parser that matches a particular exact string.

use crate::{error::Result, ParseError, ParseIter, Parser};

pub struct ExactParseIter {
    end: usize,
}

impl Parser for &'static str {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = ExactParseIter;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<ExactParseIter> {
        if source[start..].starts_with(*self) {
            Ok(ExactParseIter {
                end: start + self.len(),
            })
        } else {
            Err(ParseError::new_expected(source, start, self))
        }
    }
}

impl Parser for char {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = ExactParseIter;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<ExactParseIter> {
        if source[start..].starts_with(*self) {
            Ok(ExactParseIter {
                end: start + self.len_utf8(),
            })
        } else {
            Err(ParseError::new_expected(source, start, &self.to_string()))
        }
    }
}

impl ParseIter for ExactParseIter {
    type RawOutput = ();
    fn match_end(&self) -> usize {
        self.end
    }
    fn backtrack(&mut self) -> bool {
        false
    }
    fn take_data(&mut self) {}
}
