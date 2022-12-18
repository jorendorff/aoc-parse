//! Parsers using Regex.

use std::{
    any::{self, Any},
    fmt::Display,
};

use regex::Regex;

use crate::{ParseContext, ParseIter, Parser, Reported, Result};

/// This parser matches using a regex, then converts the value to a Rust value
/// using the given `parse_fn`.
///
/// Each time this parser is used, either it finds a regex match and `parse_fn`
/// succeeds, or it fails to match. For example, if `regex` is `/\w+/` and the
/// input is "hello world", if the RegexParser is invoked at start offset 0, it
/// either matches `"hello"` or doesn't match at all. It never falls back on
/// `"hell"` or `"h"`, even though those are both valid (shorter) regex
/// matches.
///
/// This means `sequence(/[a-z]+/, 'a')` will not match `"cba"` (or any other
/// string) because the regex matches all lowercase letters, leaving nothing
/// for the next pattern `'a'` to match.
pub struct RegexParser<T, E> {
    pub(crate) regex: fn() -> &'static Regex,
    pub(crate) parse_fn: fn(&str) -> Result<T, E>,
}

// Manual Clone impl because `#[derive(Clone)]` is buggy in this case.
impl<T, E> Clone for RegexParser<T, E> {
    fn clone(&self) -> Self {
        RegexParser {
            regex: self.regex,
            parse_fn: self.parse_fn,
        }
    }
}

impl<T, E> Copy for RegexParser<T, E> {}

pub struct RegexParseIter<T> {
    end: usize,
    value: T,
}

impl<T, E> Parser for RegexParser<T, E>
where
    T: Any + Clone,
    E: Display,
{
    type Output = T;
    type RawOutput = (T,);
    type Iter<'parse> = RegexParseIter<T>
    where
        E: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        match (self.regex)().find(&context.source()[start..]) {
            None => Err(context.error_expected(start, any::type_name::<T>())),
            Some(m) => match (self.parse_fn)(m.as_str()) {
                Ok(value) => Ok(RegexParseIter {
                    end: start + m.end(),
                    value,
                }),
                Err(err) => Err(context.error_from_str_failed(
                    start,
                    start + m.end(),
                    any::type_name::<T>(),
                    format!("{err}"),
                )),
            },
        }
    }
}

impl<'parse, T> ParseIter<'parse> for RegexParseIter<T>
where
    T: Clone,
{
    type RawOutput = (T,);
    fn match_end(&self) -> usize {
        self.end
    }
    fn backtrack(&mut self, _context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        Err(Reported)
    }
    fn convert(&self) -> (T,) {
        (self.value.clone(),)
    }
}
