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

    // pass a variable to a function
    let active = 4;
    assert_parse_eq(parser!(player(active) ": " u64), "posh: 0", 0);
}
