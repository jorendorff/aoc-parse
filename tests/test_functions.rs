use std::fmt::Debug;

use aoc_parse::{parser, prelude::*};

#[track_caller]
fn assert_parse_eq<P, E>(parser: P, s: &str, expected: E)
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
fn assert_no_parse<P>(parser: P, s: &str)
where
    P: Parser,
    P::Output: Debug,
{
    if let Ok(m) = parser.parse(s) {
        panic!("expected no match, got: {:?}", m);
    }
}

#[test]
fn test_custom_parser_combinator() {
    // Users can actually kind of write their own combinators from scratch,
    // just using the public API. Make sure such functions are callable.

    fn triplicate<P>(parser: P) -> impl Parser<Output = [P::Output; 3]> + Copy
    where
        P: Parser + Copy,
    {
        parser!(v0:parser v1:parser v2:parser => [v0, v1, v2])
    }

    let p = parser!(triplicate(line(u64)));
    assert_parse_eq(p, "34\n41\n76\n", [34, 41, 76]);
}

#[test]
fn test_function_no_arguments() {
    fn login() -> impl Parser<RawOutput = (String,), Output = String> + Copy {
        parser!(string(alpha alnum*))
    }

    let p = parser!(
        line("Username: " login())  // <--- can call function with no arguments
        line("Password: " string(any_char+))
    );

    assert_parse_eq(
        p,
        "Username: root\nPassword: hunter2\n",
        ("root".to_string(), "hunter2".to_string()),
    );
}

#[test]
fn test_function_int_argument() {
    // Full Rust expression syntax conflicts with parser syntax, but
    // integer literals and identifiers, at least, work fine.
    fn player(n: usize) -> &'static str {
        ["scary", "sporty", "baby", "ginger", "posh"][n]
    }

    // pass a numeric literal to a function
    assert_parse_eq(parser!(player(2) ": " u64), "baby: 338", 338);
    assert_parse_eq(
        parser!(repeat_sep_n(u32, '/', 3)),
        "12/21/2017",
        vec![12, 21, 2017],
    );

    // pass a variable to a function
    let active = 4;
    assert_parse_eq(parser!(player(active) ": " u64), "posh: 0", 0);
    let num_fields = 3;
    assert_parse_eq(
        parser!(repeat_sep_n(u32, '/', num_fields)),
        "12/21/2017",
        vec![12, 21, 2017],
    );
}

#[test]
fn test_repeat_variants() {
    assert_no_parse(parser!(repeat_n(digit, 5)), "1234");
    assert_parse_eq(parser!(repeat_n(digit, 5)), "12345", vec![1, 2, 3, 4, 5]);
    assert_no_parse(parser!(repeat_n(digit, 5)), "123456");

    assert_parse_eq(
        parser!(repeat_n(digit, 5) digit+),
        "1234567",
        (vec![1, 2, 3, 4, 5], vec![6, 7]),
    );
    assert_parse_eq(
        parser!(string(repeat_n(digit, 5)) string(digit+)),
        "1234567",
        ("12345".to_string(), "67".to_string()),
    );

    let min5 = parser!(repeat_min(digit, 5));
    assert_no_parse(min5, "1234");
    assert_parse_eq(min5, "12345", vec![1, 2, 3, 4, 5]);
    assert_parse_eq(min5, "123456", vec![1, 2, 3, 4, 5, 6]);
    assert_parse_eq(parser!(repeat_min(digit, 0)), "", vec![]);
    assert_parse_eq(parser!(repeat_min(digit, 0)), "12345", vec![1, 2, 3, 4, 5]);

    let max5 = parser!(repeat_max(digit, 5));
    assert_parse_eq(max5, "1234", vec![1, 2, 3, 4]);
    assert_parse_eq(max5, "12345", vec![1, 2, 3, 4, 5]);
    assert_no_parse(max5, "123456");
    assert_parse_eq(parser!(repeat_max(digit, 0)), "", vec![]);
    assert_no_parse(parser!(repeat_max(digit, 0)), "1");
    assert_parse_eq(parser!(repeat_max(digit, 1)), "", vec![]);

    let min5max7 = parser!(repeat_min_max(digit, 5, 7));
    assert_no_parse(min5max7, "1234");
    assert_parse_eq(min5max7, "12345", vec![1, 2, 3, 4, 5]);
    assert_parse_eq(min5max7, "1234567", vec![1, 2, 3, 4, 5, 6, 7]);
    assert_no_parse(min5max7, "12345678");
}

#[test]
#[should_panic]
fn test_invalid_repeat_min_max() {
    // This panics because the minimum is greater than the maximum.
    let _p = parser!(repeat_min_max(digit, 5, 4));
}
