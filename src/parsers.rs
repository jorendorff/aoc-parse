use crate::{ParseError, ParserOutput, Result};

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
    fn parse_iter<'parse>(&'parse self, source: &'parse str, start: usize) -> Self::Iter<'parse>;

    /// Like `parse` but produce the output in its [raw form][Self::RawOutput].
    fn parse_raw(&self, s: &str) -> Result<Self::RawOutput> {
        let mut it = self.parse_iter(s, 0);
        let mut best_end: Option<usize> = None;
        while let Some(parse) = it.next_parse() {
            let end = parse?;
            if end == s.len() {
                return Ok(it.take_data());
            } else {
                best_end = best_end.max(Some(end));
            }
        }
        if let Some(end) = best_end {
            Err(ParseError::new_extra(s, end))
        } else {
            panic!("parse iterator broke the contract: no matches and no error");
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

    fn parse_iter<'parse>(&'parse self, source: &'parse str, start: usize) -> Self::Iter<'parse> {
        <P as Parser>::parse_iter(self, source, start)
    }
}

/// Parse the given puzzle input supplied by `#[aoc_generator]`.
///
/// This function is like `parser.parse(puzzle_input)` except that
/// `#[aoc_generator]` unfortunately [strips off trailing newlines][bad]. This
/// function therefore checks to see if the last line is missing its final `\n`
/// and, if so, re-adds it before parsing.
///
/// # Example
///
/// ```no_run
/// use aoc_runner_derive::*;
/// use aoc_parse::{parser, prelude::*};
///
/// #[aoc_generator(day1)]
/// fn parse_input(text: &str) -> anyhow::Result<Vec<Vec<u64>>> {
///     let p = parser!(repeat_sep(lines(u64), "\n"));
///     aoc_parse(text, p)
/// }
/// ```
///
/// [bad]: https://github.com/gobanos/aoc-runner/blob/master/src/lib.rs#L17
pub fn aoc_parse<P, E>(puzzle_input: &str, parser: P) -> std::result::Result<P::Output, E>
where
    P: Parser,
    E: From<ParseError>,
{
    let mut p = puzzle_input.to_string();
    if !p.ends_with('\n') {
        p.push('\n');
    }
    Ok(parser.parse(&p)?)
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

    /// Try parsing the input.
    ///
    /// The first time this is called, it should return either `Some(Ok(end))`
    /// or `Some(Err(err))` indicating that parsing either succeeded or failed.
    ///
    /// Subsequently, it should return either `Some(Ok(end))` or `Some(None)`
    /// to indicate that there either is or isn't another, less preferable
    /// match.
    fn next_parse(&mut self) -> Option<Result<usize>>;

    /// Consume this iterator to extract data. This is called only after a
    /// successful `next_parse` call that returns `Some(Ok(offset))`.
    ///
    /// This would take `self` by value, except that's not compatible with
    /// trait objects. (`Box<Self>` is, so this could change someday.)
    fn take_data(&mut self) -> Self::RawOutput;
}

mod chars;
mod either;
mod empty;
mod exact;
mod lines;
mod map;
mod primitive;
mod regex;
mod repeat;
mod sequence;
mod string;

pub use self::regex::RegexParser;
pub use chars::{alnum, alpha, any_char, digit, digit_bin, digit_hex, lower, upper};
pub use either::{alt, either, AltParser, Either, EitherParser};
pub use empty::{empty, EmptyParser};
pub use exact::{exact, ExactParser};
pub use lines::{line, lines, section, sections, LineParser, SectionParser};
pub use map::{MapParser, MapRawParser};
pub use primitive::{
    bool, i128, i128_bin, i128_hex, i16, i16_bin, i16_hex, i32, i32_bin, i32_hex, i64, i64_bin,
    i64_hex, i8, i8_bin, i8_hex, isize, isize_bin, isize_hex, u128, u128_bin, u128_hex, u16,
    u16_bin, u16_hex, u32, u32_bin, u32_hex, u64, u64_bin, u64_hex, u8, u8_bin, u8_hex, usize,
    usize_bin, usize_hex,
};
pub use repeat::{plus, repeat, repeat_sep, star, RepeatParser};
pub use sequence::{sequence, SequenceParser};
pub use string::StringParser;

// --- Wrappers

pub fn opt<T>(
    pattern: impl Parser<Output = T> + 'static,
) -> impl Parser<Output = Option<T>, RawOutput = (Option<T>,)> {
    either(pattern, empty()).map(|e: Either<T, ()>| match e {
        Either::Left(left) => Some(left),
        Either::Right(()) => None,
    })
}

// Make sure that RawOutput is exactly `(T,)`.
//
// Parenthesizing an expression makes a semantic difference to prevent it from
// disappearing in concatenation.
//
// Example 1: In `parser!("hello " (x: i32) => x)` the raw output type of
// `"hello "` is `()` and it disappears when concatenated with `(x: i32)`. Now
// if we label `"hello"` `parser!((a: "hello ") (x: i32) => (a, x))` we have to
// make sure that doesn't happen so that we can build a pattern that matches
// both `a` and `x`.
//
// Example 2: `parser!((i32 " " i32) " " (i32))` should have the output type
// `((i32, i32), i32)`; but conatenating the three top-level RawOutput types,
// `(i32, i32)` `()` and `(i32,)`, would produce the flat `(i32, i32, i32)`
// instead.
//
// It turns out all we need is to ensure the `RawOutput` type of the
// parenthesized parser is a singleton tuple type.
pub fn parenthesize<P>(pattern: P) -> MapParser<P, fn(P::Output) -> P::Output>
where
    P: Parser,
{
    pattern.map(|val| val)
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    #[track_caller]
    fn assert_parse<P>(parser: &P, s: &str)
    where
        P: Parser,
    {
        if let Err(err) = parser.parse(s) {
            panic!("parse failed: {}", err);
        }
    }

    #[track_caller]
    fn assert_parse_eq<P, E>(parser: &P, s: &str, expected: E)
    where
        P: Parser,
        P::Output: PartialEq<E> + Debug,
        E: Debug,
    {
        match parser.parse(s) {
            Err(err) => panic!("parse failed: {}", err),
            Ok(val) => assert_eq!(val, expected),
        }
    }

    #[track_caller]
    fn assert_no_parse<P>(parser: &P, s: &str)
    where
        P: Parser,
        P::Output: Debug,
    {
        if let Ok(m) = parser.parse(s) {
            panic!("expected no match, got: {:?}", m);
        }
    }

    #[test]
    fn test_parse() {
        let p = empty();
        assert_parse_eq(&p, "", ());
        assert_no_parse(&p, "x");

        let p = exact("ok");
        assert_parse(&p, "ok");
        assert_no_parse(&p, "");
        assert_no_parse(&p, "o");
        assert_no_parse(&p, "nok");

        let p = sequence(exact("ok"), exact("go"));
        assert_parse(&p, "okgo");
        assert_no_parse(&p, "ok");
        assert_no_parse(&p, "go");
        assert_no_parse(&p, "");

        let p = either(empty(), exact("ok"));
        assert_parse(&p, "");
        assert_parse(&p, "ok");
        assert_no_parse(&p, "okc");
        assert_no_parse(&p, "okok");

        let p = star(exact("a"));
        assert_parse(&p, "");
        assert_parse(&p, "a");
        assert_parse(&p, "aa");
        assert_parse(&p, "aaa");
        assert_no_parse(&p, "b");
        assert_no_parse(&p, "ab");
        assert_no_parse(&p, "ba");

        let p = repeat_sep(exact("cow"), exact(","));
        assert_parse(&p, "");
        assert_parse(&p, "cow");
        assert_parse(&p, "cow,cow");
        assert_parse(&p, "cow,cow,cow");
        assert_no_parse(&p, "cowcow");
        assert_no_parse(&p, "cow,");
        assert_no_parse(&p, "cow,,cow");
        assert_no_parse(&p, "cow,cow,");
        assert_no_parse(&p, ",");

        let p = plus(exact("a"));
        assert_no_parse(&p, "");
        assert_parse(&p, "a");
        assert_parse(&p, "aa");

        let p = repeat_sep(usize, exact(","));
        assert_parse_eq(&p, "11417,0,0,334", vec![11417usize, 0, 0, 334]);

        assert_no_parse(&u8, "256");

        assert_parse_eq(&u8, "255", 255u8);
        assert_parse_eq(&sequence(exact("#"), u32), "#100", 100u32);
        assert_parse_eq(
            &sequence(exact("forward "), u64).map(|a| a),
            "forward 1234",
            1234u64,
        );
    }
}
