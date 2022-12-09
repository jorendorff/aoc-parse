//! Parsers using Regex.

use std::{
    any::{self, Any},
    fmt::Display,
};

use regex::Regex;

use crate::{error::Result, ParseError, ParseIter, Parser};

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
    T: Any,
    E: Display,
{
    type Output = T;
    type RawOutput = (T,);
    type Iter<'parse> = RegexParseIter<T>
    where
        E: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        match (self.regex)().find(&source[start..]) {
            None => Err(ParseError::new_expected(
                source,
                start,
                any::type_name::<T>(),
            )),
            Some(m) => match (self.parse_fn)(m.as_str()) {
                Ok(value) => Ok(RegexParseIter {
                    end: start + m.end(),
                    value,
                }),
                Err(err) => Err(ParseError::new_from_str_failed(
                    source,
                    start,
                    start + m.end(),
                    any::type_name::<T>(),
                    format!("{err}"),
                )),
            },
        }
    }
}

impl<T> ParseIter for RegexParseIter<T> {
    type RawOutput = (T,);
    fn match_end(&self) -> usize {
        self.end
    }
    fn backtrack(&mut self) -> bool {
        false
    }
    fn into_raw_output(self) -> (T,) {
        (self.value,)
    }
}
