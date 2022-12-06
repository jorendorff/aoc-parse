//! Alternation.

use crate::{
    error::Result, parsers::MapParser, types::ParserOutput, ParseError, ParseIter, Parser,
};

#[derive(Debug)]
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
    A: Parser<'parse>,
    B: Parser<'parse>,
{
    source: &'parse str,
    start: usize,
    parsers: &'parse EitherParser<A, B>,
    iter: Either<A::Iter, B::Iter>,
}

impl<'parse, A, B> Parser<'parse> for EitherParser<A, B>
where
    A: Parser<'parse> + 'parse,
    B: Parser<'parse> + 'parse,
{
    type Output = Either<A::Output, B::Output>;
    type RawOutput = (Either<A::Output, B::Output>,);
    type Iter = EitherParseIter<'parse, A, B>;

    fn parse_iter(&'parse self, source: &'parse str, start: usize) -> Self::Iter {
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
    A: Parser<'parse>,
    B: Parser<'parse>,
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

pub fn alt<A, B, T>(left: A, right: B) -> AltParser<A, B, T>
where
    A: for<'parse> Parser<'parse, Output = T>,
    B: for<'parse> Parser<'parse, Output = T>,
{
    either(left, right).map(|out| match out {
        Either::Left(value) => value,
        Either::Right(value) => value,
    })
}
