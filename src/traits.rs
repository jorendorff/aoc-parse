//! Core traits.

use crate::error::{ParseError, Result};
use crate::parsers::MapParser;
use crate::types::ParserOutput;

/// Contains the source text we're parsing and tracks errors.
///
/// We track errors in the ParseContext, not Results, because often a parser
/// produces both a successful match *and* the error that will later prove to
/// be the best error message for the overall parse attempt.
///
/// For example, consider `parser!(line(u32)+)` parsing the input
/// `1\n2\n3\n4\n5:wq\n6\n7\n`. The actual problem is that someone accidentally
/// typed `:wq` on line 5. But what will happen here is that `line(u32)+`
/// successfully parses the first 4 lines. If we don't track the error we
/// encountered when trying to parse `5:wq` as a `u32`, but content ourselves
/// with the successful match `line(u32)+` produces, the best error message we
/// can ultimately produce is something like "found extra text after a
/// successful match at line 5, column 1".
///
pub struct ParseContext<'parse> {
    source: &'parse str,
    foremost_error: Option<ParseError>,
}

impl<'parse> ParseContext<'parse> {
    /// Create a `ParseContext` to parse the given input.
    pub fn new(source: &'parse str) -> Self {
        ParseContext {
            source,
            foremost_error: None,
        }
    }

    /// The text being parsed.
    pub fn source(&self) -> &'parse str {
        self.source
    }

    /// Record an error.
    ///
    /// Currently a ParseContext only tracks the foremost error. That is, if
    /// `err.location` is farther forward than any other error we've
    /// encountered, we store it. Otherwise discard it.
    ///
    /// Nontrivial patterns try several different things. If anything succeeds,
    /// we get a match. We only fail if every branch leads to failure. This
    /// means that by the time matching fails, we have an abundance of
    /// different error messages. Generally the error we want is the one where
    /// we progressed as far as possible through the input string before
    /// failing.
    pub fn report(&mut self, err: ParseError) -> ParseError {
        if Some(err.location) > self.foremost_error.as_ref().map(|err| err.location) {
            self.foremost_error = Some(err.clone());
        }
        err
    }

    /// Record a `foo expected` error.
    pub fn error_expected(&mut self, start: usize, expected: &str) -> ParseError {
        self.report(ParseError::new_expected(self.source(), start, expected))
    }

    /// Record an error when `FromStr::from_str` fails.
    pub fn error_from_str_failed(
        &mut self,
        start: usize,
        end: usize,
        type_name: &'static str,
        message: String,
    ) -> ParseError {
        self.report(ParseError::new_from_str_failed(
            self.source(),
            start,
            end,
            type_name,
            message,
        ))
    }

    /// Record an "extra unparsed text after match" error.
    pub fn error_extra(&mut self, location: usize) -> ParseError {
        self.report(ParseError::new_extra(self.source(), location))
    }
}

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
    type Iter<'parse>: ParseIter<'parse, RawOutput = Self::RawOutput>
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
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>>;

    /// Like `parse` but produce the output in its [raw form][Self::RawOutput].
    fn parse_raw(&self, s: &str) -> Result<Self::RawOutput> {
        let mut ctx = ParseContext {
            source: s,
            foremost_error: None,
        };
        let mut it = self.parse_iter(&mut ctx, 0)?;
        let mut best_end: Option<usize> = None;
        loop {
            let end = it.match_end();
            if end == s.len() {
                return Ok(it.into_raw_output());
            }
            best_end = best_end.max(Some(end));
            if !it.backtrack(&mut ctx) {
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
pub trait ParseIter<'parse> {
    /// The type this iterator can produce on a successful match.
    type RawOutput;

    /// Position at the end of the current match.
    fn match_end(&self) -> usize;

    /// Reject the current match and find the next-most-preferable match.
    /// Returns true if another match was found, false if not.
    ///
    /// Once this returns `false`, no more method calls should be made.
    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> bool;

    /// Consume this iterator to extract data.
    fn into_raw_output(self) -> Self::RawOutput;
}

impl<'a, P> Parser for &'a P
where
    P: Parser + ?Sized,
{
    type Output = P::Output;
    type RawOutput = P::RawOutput;

    type Iter<'parse> = P::Iter<'parse>
    where
        P: 'parse,
        'a: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        <P as Parser>::parse_iter(self, context, start)
    }
}
