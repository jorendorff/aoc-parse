//! Alternation.

use crate::{
    error::Result, parsers::MapParser, types::ParserOutput, ParseError, ParseIter, Parser,
};

#[derive(Debug, PartialEq)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

#[derive(Copy, Clone)]
pub struct EitherParser<A, B> {
    left: A,
    right: B,
}

pub struct EitherParseIter<'parse, A, B>
where
    A: Parser + 'parse,
    B: Parser + 'parse,
{
    source: &'parse str,
    start: usize,
    parsers: &'parse EitherParser<A, B>,
    iter: Either<A::Iter<'parse>, B::Iter<'parse>>,
}

impl<A, B> Parser for EitherParser<A, B>
where
    A: Parser,
    B: Parser,
{
    type Output = Either<A::Output, B::Output>;
    type RawOutput = (Either<A::Output, B::Output>,);
    type Iter<'parse> = EitherParseIter<'parse, A, B>
    where
        A: 'parse,
        B: 'parse;

    fn parse_iter<'parse>(&'parse self, source: &'parse str, start: usize) -> Self::Iter<'parse> {
        EitherParseIter {
            source,
            start,
            parsers: self,
            iter: Either::Left(self.left.parse_iter(source, start)),
        }
    }
}

impl<'parse, A, B> ParseIter for EitherParseIter<'parse, A, B>
where
    A: Parser,
    B: Parser,
{
    type RawOutput = (Either<A::Output, B::Output>,);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        let mut foremost_error: Option<ParseError> = None;
        loop {
            match &mut self.iter {
                Either::Left(iter) => {
                    match iter.next_parse() {
                        None => {}
                        Some(Err(err)) => {
                            if Some(err.location) > foremost_error.as_ref().map(|err| err.location)
                            {
                                foremost_error = Some(err);
                            }
                        }
                        Some(Ok(end)) => return Some(Ok(end)),
                    }
                    self.iter =
                        Either::Right(self.parsers.right.parse_iter(self.source, self.start));
                }
                Either::Right(iter) => {
                    match iter.next_parse() {
                        None => {}
                        Some(Err(err)) => {
                            if Some(err.location) > foremost_error.as_ref().map(|err| err.location)
                            {
                                foremost_error = Some(err);
                            }
                        }
                        Some(Ok(end)) => return Some(Ok(end)),
                    }
                    return foremost_error.map(Err);
                }
            }
        }
    }

    fn take_data(&mut self) -> (Either<A::Output, B::Output>,) {
        (match &mut self.iter {
            Either::Left(iter) => Either::Left(iter.take_data().into_user_type()),
            Either::Right(iter) => Either::Right(iter.take_data().into_user_type()),
        },)
    }
}

pub fn either<A, B>(left: A, right: B) -> EitherParser<A, B> {
    EitherParser { left, right }
}

pub type AltParser<A, B, T> = MapParser<EitherParser<A, B>, fn(Either<T, T>) -> T>;

// Used by the `parser!()` macro to implement `{p1, p2, ...}` syntax.
#[doc(hidden)]
pub fn alt<A, B, T>(left: A, right: B) -> AltParser<A, B, T>
where
    A: Parser<Output = T>,
    B: Parser<Output = T>,
{
    either(left, right).map(|out| match out {
        Either::Left(value) => value,
        Either::Right(value) => value,
    })
}
