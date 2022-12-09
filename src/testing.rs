//! Prelude for unit tests that don't use the macro.
//!
//! The macro is tested from outside the crate; see `aoc-parse/tests/test_*.rs`.

use std::fmt::Debug;

pub use crate::parsers::{alt, empty, opt, plus, sequence, star};
pub use crate::{ParseError, ParseIter, Parser};

#[track_caller]
pub(crate) fn assert_parse_eq<P, E>(parser: &P, s: &str, expected: E)
where
    P: Parser + ?Sized,
    P::Output: PartialEq<E> + Debug,
    E: Debug,
{
    match parser.parse(s) {
        Err(err) => panic!("parse failed: {}", err),
        Ok(val) => assert_eq!(val, expected),
    }
}

#[track_caller]
pub(crate) fn assert_no_parse<P>(parser: &P, s: &str)
where
    P: Parser + ?Sized,
    P::Output: Debug,
{
    if let Ok(m) = parser.parse(s) {
        panic!("expected no match, got: {:?}", m);
    }
}

#[track_caller]
pub(crate) fn assert_parse_error<P>(parser: &P, s: &str, expected_message: &str)
where
    P: Parser,
    P::Output: Debug,
{
    match parser.parse(s) {
        Ok(m) => panic!("expected no match, got: {:?}", m),
        Err(err) => {
            let actual = err.to_string();
            if !actual.contains(expected_message) {
                panic!("expected error message containing {expected_message:?}, got {actual:?}");
            }
        }
    }
}
