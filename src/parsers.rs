use crate::Parser;

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

// Used by the `parser!()` macro to implement the `?` quantifier.
#[doc(hidden)]
pub fn opt<T>(
    pattern: impl Parser<Output = T> + 'static,
) -> impl Parser<Output = Option<T>, RawOutput = (Option<T>,)> {
    either(pattern, empty()).map(|e: Either<T, ()>| match e {
        Either::Left(left) => Some(left),
        Either::Right(()) => None,
    })
}

type ParenthesizedParser<P> = MapParser<P, fn(<P as Parser>::Output) -> <P as Parser>::Output>;

// Make sure that RawOutput is exactly `(T,)`.
//
// Used by the `parser!()` macro to implement grouping parentheses.
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
#[doc(hidden)]
pub fn parenthesize<P>(pattern: P) -> ParenthesizedParser<P>
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

    #[test]
    fn test_parse_hex() {
        assert_no_parse(&i32_hex, "+");
        assert_no_parse(&i32_hex, "-");
        assert_no_parse(&i32_hex, "+ 4");
        assert_no_parse(&i32_hex, "+ 4");
        assert_parse_eq(&i32_hex, "7BCDEF01", 0x7bcdef01);
        assert_parse_eq(&i32_hex, "7fffffff", i32::MAX);
        assert_no_parse(&i32_hex, "80000000");
        assert_parse_eq(&i32_hex, "-80000000", i32::MIN);
        assert_no_parse(&i32_hex, "-80000001");

        let p = sequence(i32_hex, i32_hex);
        assert_no_parse(&p, "12");
        assert_no_parse(&p, "01230123ABCDABCD");
        assert_parse_eq(&p, "-1+1", (-1, 1));

        assert_no_parse(&u32_hex, "-1");
        assert_no_parse(&u32_hex, "+d3d32e2e");
        assert_parse_eq(&u32_hex, "ffffffff", u32::MAX);
        assert_parse_eq(&u32_hex, "ffffffff", u32::MAX);
        assert_parse_eq(
            &u32_hex,
            "0000000000000000000000000000000000000000000000000000000000000000ffffffff",
            u32::MAX,
        );
    }
}
