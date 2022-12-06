//! Parser that matches a particular exact string.

use crate::{error::Result, ParseError, ParseIter, Parser};

#[derive(Clone, Copy)]
pub struct ExactParser {
    s: &'static str,
}

pub struct ExactParseIter<'parse> {
    expected: &'parse str,
    input: &'parse str,
    start: usize,
    done: bool,
}

impl<'parse> Parser<'parse> for ExactParser {
    type Output = ();
    type RawOutput = ();
    type Iter = ExactParseIter<'parse>;

    fn parse_iter<'source>(
        &'parse self,
        source: &'source str,
        start: usize,
    ) -> ExactParseIter<'parse>
    where
        'source: 'parse,
    {
        ExactParseIter {
            expected: &self.s,
            input: source,
            start,
            done: false,
        }
    }
}

impl<'parse> ParseIter for ExactParseIter<'parse> {
    type RawOutput = ();

    fn next_parse(&mut self) -> Option<Result<usize>> {
        if self.done {
            None
        } else if self.input[self.start..].starts_with(self.expected) {
            self.done = true;
            Some(Ok(self.start + self.expected.len()))
        } else {
            Some(Err(ParseError::new_expected(
                self.input,
                self.start,
                self.expected,
            )))
        }
    }

    fn take_data(&mut self) {}
}

pub fn exact(s: &'static str) -> ExactParser {
    ExactParser { s }
}
