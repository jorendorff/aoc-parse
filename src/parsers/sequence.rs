//! Matching patterns in sequence.

use crate::{
    types::{ParserOutput, RawOutputConcat},
    ParseContext, ParseIter, Parser, Reported, Result,
};

#[derive(Clone, Copy)]
pub struct SequenceParser<Head, Tail, Op> {
    head: Head,
    tail: Tail,
    _op: Op,
}

pub struct SequenceParseIter<'parse, Head, Tail, Op>
where
    Head: Parser + 'parse,
    Tail: Parser + 'parse,
{
    parsers: &'parse SequenceParser<Head, Tail, Op>,
    head_iter: Head::Iter<'parse>,
    tail_iter: Tail::Iter<'parse>,
}

/// Operation that joins two values.
pub trait BinaryOp<Head, Tail>: Copy + Clone {
    type Output: ParserOutput;

    fn apply(head: Head, tail: Tail) -> Self::Output;
}

/// Combine sequenced parsers by concatenating output tuples.
#[derive(Debug, Copy, Clone)]
pub struct Concat;

impl<Head, Tail> BinaryOp<Head, Tail> for Concat
where
    Head: RawOutputConcat<Tail>,
{
    type Output = <Head as RawOutputConcat<Tail>>::Output;

    fn apply(head: Head, tail: Tail) -> Self::Output {
        head.concat(tail)
    }
}

/// Combine sequenced parsers by creating nested output tuples.
#[derive(Debug, Copy, Clone)]
pub struct Pair;

impl<Head, Tail> BinaryOp<Head, Tail> for Pair
where
    Head: ParserOutput,
    Tail: ParserOutput,
{
    type Output = (Head::UserType, Tail::UserType);

    fn apply(head: Head, tail: Tail) -> Self::Output {
        (head.into_user_type(), tail.into_user_type())
    }
}

impl<Head, Tail, Op> Parser for SequenceParser<Head, Tail, Op>
where
    Head: Parser,
    Tail: Parser,
    Op: BinaryOp<Head::RawOutput, Tail::RawOutput>,
{
    type Output = <Op::Output as ParserOutput>::UserType;
    type RawOutput = Op::Output;
    type Iter<'parse> = SequenceParseIter<'parse, Head, Tail, Op>
    where
        Head: 'parse,
        Tail: 'parse,
        Op: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
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
) -> Result<Tail::Iter<'parse>, Reported>
where
    Head: Parser,
    Tail: Parser,
{
    loop {
        let mid = head.match_end();
        if let Ok(tail_iter) = tail.parse_iter(context, mid) {
            return Ok(tail_iter);
        }
        head.backtrack(context)?;
    }
}

impl<'parse, Head, Tail, Op> ParseIter<'parse> for SequenceParseIter<'parse, Head, Tail, Op>
where
    Head: Parser,
    Tail: Parser,
    Op: BinaryOp<Head::RawOutput, Tail::RawOutput>,
    Op::Output: ParserOutput,
{
    type RawOutput = Op::Output;

    fn match_end(&self) -> usize {
        self.tail_iter.match_end()
    }

    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        self.tail_iter.backtrack(context).or_else(|Reported| {
            self.head_iter.backtrack(context)?;
            let tail_iter =
                first_tail_match::<Head, Tail>(context, &mut self.head_iter, &self.parsers.tail)?;
            self.tail_iter = tail_iter;
            Ok(())
        })
    }

    fn convert(&self) -> Self::RawOutput {
        let head = self.head_iter.convert();
        let tail = self.tail_iter.convert();
        Op::apply(head, tail)
    }
}

// Used by the `parser!()` macro to implement concatenation.
#[doc(hidden)]
pub fn sequence<Head, Tail>(head: Head, tail: Tail) -> SequenceParser<Head, Tail, Concat> {
    SequenceParser {
        head,
        tail,
        _op: Concat,
    }
}

// Used by the `parser!()` macro to implement `=>`-mapping.
#[doc(hidden)]
pub fn pair<Head, Tail>(head: Head, tail: Tail) -> SequenceParser<Head, Tail, Pair> {
    SequenceParser {
        head,
        tail,
        _op: Pair,
    }
}
