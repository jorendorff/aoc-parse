//! Mapping parsers.

use crate::{types::ParserOutput, ParseContext, ParseIter, Parser, Reported, Result};

pub trait Mapping<T> {
    type RawOutput: ParserOutput;

    fn apply(&self, value: T) -> Self::RawOutput;
}

impl<F, T, U> Mapping<T> for F
where
    T: ParserOutput,
    F: Fn(T::UserType) -> U,
{
    type RawOutput = (U,);

    fn apply(&self, value: T) -> Self::RawOutput {
        (self(value.into_user_type()),)
    }
}

#[derive(Clone, Copy)]
pub struct MapParser<P, F> {
    pub(crate) inner: P,
    pub(crate) mapper: F,
}

pub struct MapParseIter<'parse, P, F>
where
    P: Parser + 'parse,
{
    inner: P::Iter<'parse>,
    mapper: &'parse F,
}

impl<P, F> Parser for MapParser<P, F>
where
    P: Parser,
    F: Mapping<P::RawOutput>,
{
    type Output = <F::RawOutput as ParserOutput>::UserType;
    type RawOutput = F::RawOutput;
    type Iter<'parse> = MapParseIter<'parse, P, F>
    where
        P: 'parse,
        F: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        let iter = self.inner.parse_iter(context, start)?;
        let mapper = &self.mapper;
        Ok(MapParseIter {
            inner: iter,
            mapper,
        })
    }
}

impl<'parse, P, F> ParseIter<'parse> for MapParseIter<'parse, P, F>
where
    P: Parser,
    F: Mapping<P::RawOutput>,
{
    type RawOutput = F::RawOutput;

    fn match_end(&self) -> usize {
        self.inner.match_end()
    }

    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        self.inner.backtrack(context)
    }

    fn into_raw_output(self) -> F::RawOutput {
        let value = self.inner.into_raw_output();
        self.mapper.apply(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Skip;

impl<T> Mapping<T> for Skip {
    type RawOutput = ();
    fn apply(&self, _value: T) {}
}

/// A parser that matches the same strings as `P` but after performing
/// conversion just discards the values and returns `()`.
///
/// In `parser!(x:i32 delim y:i32 => Point(x, y))` we use a SkipParser to make
/// sure `delim` doesn't produce a value, as we need exactly two values to pass
/// to the mapper `|(x, y)| Point(x, y)`.
pub type SkipParser<P> = MapParser<P, Skip>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SingleValue;

impl<T: ParserOutput> Mapping<T> for SingleValue {
    type RawOutput = (T::UserType,);

    fn apply(&self, value: T) -> Self::RawOutput {
        (value.into_user_type(),)
    }
}

/// A parser that matches the same strings as `P` and has a singleton tuple as
/// its RawOutput type.
pub type SingleValueParser<P> = MapParser<P, SingleValue>;

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
#[doc(hidden)]
pub fn map<P, T, F>(parser: P, mapper: F) -> MapParser<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> T,
{
    MapParser {
        inner: parser,
        mapper,
    }
}

/// Return a parser that matches the same strings as `parser`, but after
/// performing conversion just discards the values and returns `()`.
#[allow(dead_code)]
pub fn skip<P>(parser: P) -> SkipParser<P>
where
    P: Parser,
{
    SkipParser {
        inner: parser,
        mapper: Skip,
    }
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
    SingleValueParser {
        inner: parser,
        mapper: SingleValue,
    }
}
