//! Matching patterns in sequence.

use crate::{
    error::Result,
    types::{ParserOutput, TupleConcat},
    ParseError, ParseIter, Parser,
};

#[derive(Clone, Copy)]
pub struct SequenceParser<Head, Tail> {
    head: Head,
    tail: Tail,
}

pub struct SequenceParseIter<'parse, 'source, Head, Tail>
where
    Head: Parser<'parse, 'source>,
    Tail: Parser<'parse, 'source>,
{
    parsers: &'parse SequenceParser<Head, Tail>,
    is_at_start: bool,
    source: &'source str,
    start: usize,
    head_iter: Option<Head::Iter>,
    tail_iter: Option<Tail::Iter>,
}

impl<'parse, 'source, Head, Tail> Parser<'parse, 'source> for SequenceParser<Head, Tail>
where
    Head: Parser<'parse, 'source> + 'parse,
    Tail: Parser<'parse, 'source> + 'parse,
    Head::RawOutput: TupleConcat<Tail::RawOutput>,
{
    type Output =
        <<Head::RawOutput as TupleConcat<Tail::RawOutput>>::Output as ParserOutput>::UserType;
    type RawOutput = <Head::RawOutput as TupleConcat<Tail::RawOutput>>::Output;
    type Iter = SequenceParseIter<'parse, 'source, Head, Tail>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> Self::Iter {
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

impl<'parse, 'source, Head, Tail> ParseIter for SequenceParseIter<'parse, 'source, Head, Tail>
where
    Head: Parser<'parse, 'source>,
    Tail: Parser<'parse, 'source>,
    Head::RawOutput: TupleConcat<Tail::RawOutput>,
{
    type RawOutput = <Head::RawOutput as TupleConcat<Tail::RawOutput>>::Output;

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
