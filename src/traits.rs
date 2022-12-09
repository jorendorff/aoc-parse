//! Core traits.

use crate::error::{ParseError, Result};
use crate::parsers::MapParser;
use crate::types::ParserOutput;

/// Trait implemented by all parsers.
///
/// This is implemented by the built-in parsers, like `i32`, as well as
/// user-defined parsers created with `parser!`.
///
/// To run a parser, pass some text to [the `parse` method][Parser::parse].
pub trait Parser {
    /// The type of value this parser produces from text.
    type Output;

    /// The type this parser produces internally before converting to the output type.
    ///
    /// Some combinators use the `RawOutput` to determine how types should combine.
    /// For example, if `A::RawOutput = ()`, then `A` produces no output;
    /// and if `B::RawOutput = (i32,)` then `B` produces an integer;
    /// `SequenceParser<A, B>::RawOutput` will then be `(i32,)`, the
    /// result of concatenating the two raw tuples, rather than `((), i32)`.
    ///
    /// However, `RawOutput` is very often a singleton tuple, and these are
    /// awkward for users, so we convert to the `Output` type before presenting a
    /// result to the user.
    type RawOutput: ParserOutput<UserType = Self::Output>;

    /// The type that implements matching, backtracking, and type conversion
    /// for this parser, an implementation detail.
    type Iter<'parse>: ParseIter<RawOutput = Self::RawOutput>
    where
        Self: 'parse;

    /// Fully parse the given source string `s` and return the resulting value.
    ///
    /// This is the main way of using a `Parser`.
    ///
    /// This succeeds only if this parser matches the entire input string. It's
    /// an error if any unmatched characters are left over at the end of `s`.
    fn parse(&self, s: &str) -> Result<Self::Output> {
        self.parse_raw(s).map(|v| v.into_user_type())
    }

    /// Produce a [parse iterator][ParseIter]. This is an internal implementation detail of
    /// the parser and shouldn't normally be called directly from application code.
    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<Self::Iter<'parse>>;

    /// Like `parse` but produce the output in its [raw form][Self::RawOutput].
    fn parse_raw(&self, s: &str) -> Result<Self::RawOutput> {
        let mut it = self.parse_iter(s, 0)?;
        let mut best_end: Option<usize> = None;
        loop {
            let end = it.match_end();
            if end == s.len() {
                return Ok(it.into_raw_output());
            }
            best_end = best_end.max(Some(end));
            if !it.backtrack() {
                return Err(ParseError::new_extra(s, best_end.unwrap()));
            }
        }
    }

    /// Produce a new parser that behaves like this parser but additionally
    /// applies the given closure when producing the value.
    ///
    /// ```
    /// use aoc_parse::{parser, prelude::*};
    /// let p = u32.map(|x| x * 1_000_001);
    /// assert_eq!(p.parse("123").unwrap(), 123_000_123);
    /// ```
    ///
    /// This is used to implement the `=>` feature of `parser!`.
    ///
    /// ```
    /// # use aoc_parse::{parser, prelude::*};
    /// let p = parser!(x: u32 => x * 1_000_001);
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
    fn map<T, F>(self, mapper: F) -> MapParser<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> T,
    {
        MapParser::new(self, mapper)
    }
}

/// A parser in action. Some parsers can match in several different ways (for
/// example, in `foo* bar` backtracking is accomplished by `foo*` first
/// matching as much as possible, then backing off one match at a time), so
/// this is an iterator.
///
/// This doesn't return a `RawOutput` value from `next_parse` but instead waits
/// until you're sure you have a complete, successful parse, and are thus ready
/// to destroy the iterator. This helps us avoid building values only to drop
/// them later when some downstream parser fails to match, so it makes
/// backtracking faster. It also means we don't call `.map` closures until
/// there is a successful overall match and the values are actually needed.
pub trait ParseIter {
    /// The type this iterator can produce on a successful match.
    type RawOutput;

    /// Position at the end of the current match.
    fn match_end(&self) -> usize;

    /// Reject the current match and find the next-most-preferable match.
    /// Returns true if another match was found, false if not.
    ///
    /// Once this returns `false`, no more method calls should be made.
    fn backtrack(&mut self) -> bool;

    /// Consume this iterator to extract data.
    fn into_raw_output(self) -> Self::RawOutput;
}

impl<'a, P> Parser for &'a P
where
    P: Parser,
{
    type Output = P::Output;
    type RawOutput = P::RawOutput;

    type Iter<'parse> = P::Iter<'parse>
    where
        P: 'parse,
        'a: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        <P as Parser>::parse_iter(self, source, start)
    }
}
