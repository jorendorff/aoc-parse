//! Matching patterns in sequence.

use crate::{
    types::{ParserOutput, RawOutputConcat},
    ParseContext, ParseError, ParseIter, Parser, Result,
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
    head_iter: Head::Iter<'parse>,
    tail_iter: Tail::Iter<'parse>,
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
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        let mut head_iter = self.head.parse_iter(context, start)?;
        let tail_iter = first_tail_match::<Head, Tail>(context, &mut head_iter, &self.tail)?;
        Ok(SequenceParseIter {
            parsers: self,
            head_iter,
            tail_iter,
        })
    }
}

fn first_tail_match<'parse, Head, Tail>(
    context: &mut ParseContext<'parse>,
    head: &mut Head::Iter<'parse>,
    tail: &'parse Tail,
) -> Result<Tail::Iter<'parse>>
where
    Head: Parser,
    Tail: Parser,
{
    let mut any_err: ParseError;
    loop {
        let mid = head.match_end();
        match tail.parse_iter(context, mid) {
            Ok(tail_iter) => return Ok(tail_iter),
            Err(err) => any_err = err,
        }
        if !head.backtrack(context) {
            break;
        }
    }
    Err(any_err)
}

impl<'parse, Head, Tail> ParseIter<'parse> for SequenceParseIter<'parse, Head, Tail>
where
    Head: Parser,
    Tail: Parser,
    Head::RawOutput: RawOutputConcat<Tail::RawOutput>,
{
    type RawOutput = <Head::RawOutput as RawOutputConcat<Tail::RawOutput>>::Output;

    fn match_end(&self) -> usize {
        self.tail_iter.match_end()
    }

    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> bool {
        if self.tail_iter.backtrack(context) {
            return true;
        }
        if !self.head_iter.backtrack(context) {
            return false;
        }
        match first_tail_match::<Head, Tail>(context, &mut self.head_iter, &self.parsers.tail) {
            Ok(tail_iter) => {
                self.tail_iter = tail_iter;
                true
            }
            Err(_err) => {
                // todo: deal with _err
                false
            }
        }
    }

    fn into_raw_output(self) -> Self::RawOutput {
        let head = self.head_iter.into_raw_output();
        let tail = self.tail_iter.into_raw_output();
        head.concat(tail)
    }
}

// Used by the `parser!()` macro to implement concatenation.
#[doc(hidden)]
pub fn sequence<Head, Tail>(head: Head, tail: Tail) -> SequenceParser<Head, Tail> {
    SequenceParser { head, tail }
}
