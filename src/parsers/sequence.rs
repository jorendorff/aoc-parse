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
    Head: Parser<'parse>,
    Tail: Parser<'parse>,
{
    parsers: &'parse SequenceParser<Head, Tail>,
    is_at_start: bool,
    source: &'parse str,
    start: usize,
    head_iter: Option<Head::Iter>,
    tail_iter: Option<Tail::Iter>,
}

impl<'parse, Head, Tail> Parser<'parse> for SequenceParser<Head, Tail>
where
    Head: Parser<'parse> + 'parse,
    Tail: Parser<'parse> + 'parse,
    Head::RawOutput: RawOutputConcat<Tail::RawOutput>,
{
    type Output =
        <<Head::RawOutput as RawOutputConcat<Tail::RawOutput>>::Output as ParserOutput>::UserType;
    type RawOutput = <Head::RawOutput as RawOutputConcat<Tail::RawOutput>>::Output;
    type Iter = SequenceParseIter<'parse, Head, Tail>;

    fn parse_iter<'source>(&'parse self, source: &'source str, start: usize) -> Self::Iter
    where
        'source: 'parse,
    {
        SequenceParseIter {
            parsers: self,
            is_at_start: true,
            source,
            start,
            head_iter: None,
            tail_iter: None,
        }
    }
}

impl<'parse, Head, Tail> ParseIter for SequenceParseIter<'parse, Head, Tail>
where
    Head: Parser<'parse>,
    Tail: Parser<'parse>,
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
                        if foremost_error.as_ref().map(|err| err.location) < Some(err.location) {
                            foremost_error = Some(err);
                        }
                    }

                    Some(Ok(tail_end)) => return Some(Ok(tail_end)),
                }
                self.tail_iter = None;
            } else if let Some(head_iter) = &mut self.head_iter {
                match head_iter.next_parse() {
                    None => {}
                    Some(Err(err)) => {
                        if foremost_error.as_ref().map(|err| err.location) < Some(err.location) {
                            foremost_error = Some(err);
                        }
                    }
                    Some(Ok(head_end)) => {
                        self.tail_iter = Some(self.parsers.tail.parse_iter(self.source, head_end));
                        continue;
                    }
                }
                self.head_iter = None;
                return foremost_error.map(Err);
            } else if self.is_at_start {
                self.is_at_start = false;
                self.head_iter = Some(self.parsers.head.parse_iter(self.source, self.start));
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

pub fn sequence<Head, Tail>(head: Head, tail: Tail) -> SequenceParser<Head, Tail> {
    SequenceParser { head, tail }
}
