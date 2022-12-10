//! Alternation.

use crate::{
    parsers::MapParser, types::ParserOutput, ParseContext, ParseIter, Parser, Reported, Result,
};

#[derive(Debug, PartialEq, Eq)]
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

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        let iter = match self.left.parse_iter(context, start) {
            Ok(iter) => Either::Left(iter),
            Err(Reported) => Either::Right(self.right.parse_iter(context, start)?),
        };
        Ok(EitherParseIter {
            start,
            parsers: self,
            iter,
        })
    }
}

impl<'parse, A, B> ParseIter<'parse> for EitherParseIter<'parse, A, B>
where
    A: Parser,
    B: Parser,
{
    type RawOutput = (Either<A::Output, B::Output>,);

    fn match_end(&self) -> usize {
        match &self.iter {
            Either::Left(iter) => iter.match_end(),
            Either::Right(iter) => iter.match_end(),
        }
    }

    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        match &mut self.iter {
            Either::Left(iter) => {
                if iter.backtrack(context).is_ok() {
                    return Ok(());
                }
                self.iter = Either::Right(self.parsers.right.parse_iter(context, self.start)?);
                Ok(())
            }
            Either::Right(iter) => iter.backtrack(context),
        }
    }

    fn into_raw_output(self) -> (Either<A::Output, B::Output>,) {
        (match self.iter {
            Either::Left(iter) => Either::Left(iter.into_raw_output().into_user_type()),
            Either::Right(iter) => Either::Right(iter.into_raw_output().into_user_type()),
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
