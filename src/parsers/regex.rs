//! Parsers using Regex.

use std::{
    any::{self, Any},
    fmt::Display,
};

use regex::Regex;

use crate::{error::Result, ParseError, ParseIter, Parser};

pub struct RegexParser<T, E> {
    pub(crate) regex: fn() -> &'static Regex,
    pub(crate) parse_fn: fn(&str) -> std::result::Result<T, E>,
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

pub enum RegexParseIter<'parse, 'source, T, E> {
    Init {
        source: &'source str,
        start: usize,
        parser: &'parse RegexParser<T, E>,
    },
    Done {
        value: Option<T>,
    },
}

impl<'parse, 'source, T, E> Parser<'parse, 'source> for RegexParser<T, E>
where
    T: 'static,
    E: Display + 'parse,
{
    type Output = T;
    type RawOutput = (T,);
    type Iter = RegexParseIter<'parse, 'source, T, E>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> Self::Iter {
        RegexParseIter::Init {
            source,
            start,
            parser: self,
        }
    }
}

impl<'parse, 'source, T, E> ParseIter for RegexParseIter<'parse, 'source, T, E>
where
    T: Any,
    E: Display,
{
    type RawOutput = (T,);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        match *self {
            RegexParseIter::Init {
                source,
                start,
                parser,
            } => match (parser.regex)().find(&source[start..]) {
                Some(m) => match (parser.parse_fn)(m.as_str()) {
                    Ok(value) => {
                        *self = RegexParseIter::Done { value: Some(value) };
                        Some(Ok(start + m.end()))
                    }
                    Err(err) => {
                        *self = RegexParseIter::Done { value: None };
                        Some(Err(ParseError::new_from_str_failed(
                            source,
                            start,
                            start + m.end(),
                            any::type_name::<T>(),
                            format!("{err}"),
                        )))
                    }
                },
                None => {
                    *self = RegexParseIter::Done { value: None };
                    Some(Err(ParseError::new_expected(
                        source,
                        start,
                        any::type_name::<T>(),
                    )))
                }
            },
            _ => None,
        }
    }

    fn take_data(&mut self) -> Self::RawOutput {
        let v = match self {
            RegexParseIter::Done { value } => value.take().unwrap(),
            _ => unreachable!("matching failed"),
        };
        (v,)
    }
}
