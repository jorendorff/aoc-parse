//! Matching patterns in sequence.

use crate::{
    error::Result,
    types::{ParserOutput, RawOutputConcat},
    ParseError, ParseIter, Parser,
};

#[derive(Clone, Copy)]
pub struct SequenceParser<Head, Tail> {
    head: Head,
    tail: Tail,
}

pub struct SequenceParseIter<'parse, Head, Tail>
where
    Head: Parser + 'parse,
    Tail: Parser + 'parse,
{
    parsers: &'parse SequenceParser<Head, Tail>,
    source: &'parse str,
    head_iter: Option<Head::Iter<'parse>>,
    tail_iter: Option<Tail::Iter<'parse>>,
}

impl<Head, Tail> Parser for SequenceParser<Head, Tail>
where
    Head: Parser,
    Tail: Parser,
    Head::RawOutput: RawOutputConcat<Tail::RawOutput>,
{
    type Output =
        <<Head::RawOutput as RawOutputConcat<Tail::RawOutput>>::Output as ParserOutput>::UserType;
    type RawOutput = <Head::RawOutput as RawOutputConcat<Tail::RawOutput>>::Output;
    type Iter<'parse> = SequenceParseIter<'parse, Head, Tail>
    where
        Head: 'parse,
        Tail: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        let iter = self.head.parse_iter(source, start)?;
        Ok(SequenceParseIter {
            parsers: self,
            source,
            head_iter: Some(iter),
            tail_iter: None,
        })
    }
}

impl<'parse, Head, Tail> ParseIter for SequenceParseIter<'parse, Head, Tail>
where
    Head: Parser,
    Tail: Parser,
    Head::RawOutput: RawOutputConcat<Tail::RawOutput>,
{
    type RawOutput = <Head::RawOutput as RawOutputConcat<Tail::RawOutput>>::Output;

    fn next_parse(&mut self) -> Option<Result<usize>> {
        let mut foremost_error: Option<ParseError> = None;
        loop {
            if let Some(tail_iter) = &mut self.tail_iter {
                match tail_iter.next_parse() {
                    None => {}
                    Some(Err(err)) => {
                        ParseError::keep_best(&mut foremost_error, err);
                    }
                    Some(Ok(tail_end)) => return Some(Ok(tail_end)),
                }
                self.tail_iter = None;
            } else if let Some(head_iter) = &mut self.head_iter {
                match head_iter.next_parse() {
                    None => {}
                    Some(Err(err)) => {
                        ParseError::keep_best(&mut foremost_error, err);
                    }
                    Some(Ok(head_end)) => {
                        match self.parsers.tail.parse_iter(self.source, head_end) {
                            Ok(iter) => {
                                self.tail_iter = Some(iter);
                            }
                            Err(err) => {
                                ParseError::keep_best(&mut foremost_error, err);
                            }
                        }
                        continue;
                    }
                }
                self.head_iter = None;
                return foremost_error.map(Err);
            } else {
                return None;
            }
        }
    }

    fn take_data(&mut self) -> Self::RawOutput {
        let head = self.head_iter.as_mut().unwrap().take_data();
        let tail = self.tail_iter.as_mut().unwrap().take_data();
        head.concat(tail)
    }
}

// Used by the `parser!()` macro to implement concatenation.
#[doc(hidden)]
pub fn sequence<Head, Tail>(head: Head, tail: Tail) -> SequenceParser<Head, Tail> {
    SequenceParser { head, tail }
}
