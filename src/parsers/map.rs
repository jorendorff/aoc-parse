//! Mapping parsers.

use crate::{types::ParserOutput, ParseContext, ParseIter, Parser, Reported, Result};

#[derive(Clone, Copy)]
pub struct MapParser<P, F> {
    pub(crate) parser: P,
    pub(crate) mapper: F,
}

pub struct MapParseIter<'parse, P, F>
where
    P: Parser + 'parse,
{
    iter: P::Iter<'parse>,
    mapper: &'parse F,
}

impl<P, F, T> Parser for MapParser<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> T,
{
    type Output = T;
    type RawOutput = (T,);
    type Iter<'parse> = MapParseIter<'parse, P, F>
    where
        P: 'parse,
        F: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        let iter = self.parser.parse_iter(context, start)?;
        let mapper = &self.mapper;
        Ok(MapParseIter { iter, mapper })
    }
}

impl<'parse, P, F, T> ParseIter<'parse> for MapParseIter<'parse, P, F>
where
    P: Parser,
    F: Fn(P::Output) -> T,
{
    type RawOutput = (T,);

    fn match_end(&self) -> usize {
        self.iter.match_end()
    }

    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        self.iter.backtrack(context)
    }

    fn into_raw_output(self) -> (T,) {
        let value = (self.mapper)(self.iter.into_raw_output().into_user_type());
        (value,)
    }
}

/// A parser that matches the same strings as `P` but after performing
/// conversion just discards the values and returns `()`.
///
/// In `parser!(x:i32 delim y:i32 => Point(x, y))` we use a SkipParser to make
/// sure `delim` doesn't produce a value, as we need exactly two values to pass
/// to the mapper `|(x, y)| Point(x, y)`.
#[derive(Debug, Clone, Copy)]
pub struct SkipParser<P> {
    inner: P,
}

#[derive(Debug, Clone, Copy)]
pub struct SkipParseIter<Iter> {
    inner: Iter,
}

impl<P> Parser for SkipParser<P>
where
    P: Parser,
{
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = SkipParseIter<P::Iter<'parse>> where P: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut crate::ParseContext<'parse>,
        start: usize,
    ) -> crate::error::Result<Self::Iter<'parse>, crate::Reported> {
        let inner = self.inner.parse_iter(context, start)?;
        Ok(SkipParseIter { inner })
    }
}

impl<'parse, Iter> ParseIter<'parse> for SkipParseIter<Iter>
where
    Iter: ParseIter<'parse>,
{
    type RawOutput = ();

    fn match_end(&self) -> usize {
        self.inner.match_end()
    }

    fn backtrack(
        &mut self,
        context: &mut crate::ParseContext<'parse>,
    ) -> crate::error::Result<(), crate::Reported> {
        self.inner.backtrack(context)
    }

    fn into_raw_output(self) {
        let _ = self.inner.into_raw_output();
    }
}

/// A parser that matches the same strings as `P` and has a singleton tuple as
/// its RawOutput type.
#[derive(Debug, Clone, Copy)]
pub struct SingleValueParser<P> {
    inner: P,
}

#[derive(Debug)]
pub struct SingleValueParseIter<'parse, P>
where
    P: Parser + 'parse,
{
    inner: P::Iter<'parse>,
}

impl<P> Parser for SingleValueParser<P>
where
    P: Parser,
{
    type Output = P::Output;
    type RawOutput = (P::Output,);
    type Iter<'parse> = SingleValueParseIter<'parse, P> where P: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut crate::ParseContext<'parse>,
        start: usize,
    ) -> crate::error::Result<Self::Iter<'parse>, crate::Reported> {
        let inner = self.inner.parse_iter(context, start)?;
        Ok(SingleValueParseIter { inner })
    }
}

impl<'parse, P> ParseIter<'parse> for SingleValueParseIter<'parse, P>
where
    P: Parser + 'parse,
{
    type RawOutput = (P::Output,);

    fn match_end(&self) -> usize {
        self.inner.match_end()
    }

    fn backtrack(
        &mut self,
        context: &mut crate::ParseContext<'parse>,
    ) -> crate::error::Result<(), crate::Reported> {
        self.inner.backtrack(context)
    }

    fn into_raw_output(self) -> Self::RawOutput {
        (self.inner.into_raw_output().into_user_type(),)
    }
}

/// Produce a new parser that behaves like this parser but additionally
/// applies the given closure when producing the value.
///
/// ```
/// use aoc_parse::{parser, prelude::*, macros::map};
/// let p = map(u32, |x| x * 1_000_001);
/// assert_eq!(p.parse("123").unwrap(), 123_000_123);
/// ```
///
/// This is used to implement the `=>` feature of `parser!`.
///
/// ```
/// # use aoc_parse::{parser, prelude::*};
/// let p = parser!(x:u32 => x * 1_000_001);
/// assert_eq!(p.parse("123").unwrap(), 123_000_123);
/// ```
///
/// The closure is called after the *overall* parse succeeds, as part of
/// turning the parse into Output values. This means the function
/// will not be called during a partly-successful parse that later fails.
///
/// ```
/// # use aoc_parse::{parser, prelude::*};
/// let p = parser!(("A" => panic!()) "B" "C");
/// assert!(p.parse("ABX").is_err());
///
/// let p2 = parser!({
///    (i32 => panic!()) " ft" => 1,
///    i32 " km" => 2,
/// });
/// assert_eq!(p2.parse("37 km").unwrap(), 2);
/// ```
pub fn map<P, T, F>(parser: P, mapper: F) -> MapParser<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> T,
{
    MapParser { parser, mapper }
}

/// Return a parser that matches the same strings as `parser`, but after
/// performing conversion just discards the values and returns `()`.
///
/// In `parser!(x:i32 delim y:i32 => Point(x, y))` we use `skip()` to make
/// sure `delim` doesn't produce a value, as we need exactly two values to pass
/// to the mapper `|(x, y)| Point(x, y)`.
pub fn skip<P>(parser: P) -> SkipParser<P>
where
    P: Parser,
{
    SkipParser { inner: parser }
}

/// Return a parser that matches the same strings as `parser` and has a
/// singleton tuple as its RawOutput type.
///
/// Used by the `parser!()` macro to implement grouping parentheses.
/// Parenthesizing an expression makes a semantic difference to prevent it from
/// disappearing in concatenation.
///
/// Example 1: In `parser!("hello " (x: i32) => x)` the raw output type of
/// `"hello "` is `()` and it disappears when concatenated with `(x: i32)`. Now
/// if we label `"hello"` `parser!((a: "hello ") (x: i32) => (a, x))` we have to
/// make sure that doesn't happen so that we can build a pattern that matches
/// both `a` and `x`.
///
/// Example 2: `parser!((i32 " " i32) " " (i32))` should have the output type
/// `((i32, i32), i32)`; but conatenating the three top-level RawOutput types,
/// `(i32, i32)` `()` and `(i32,)`, would produce the flat `(i32, i32, i32)`
/// instead.
///
/// It turns out all we need is to ensure the `RawOutput` type of the
/// parenthesized parser is a singleton tuple type.
pub fn single_value<P>(parser: P) -> SingleValueParser<P>
where
    P: Parser,
{
    SingleValueParser { inner: parser }
}
