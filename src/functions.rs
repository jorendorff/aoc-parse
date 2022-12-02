//! Trait for functions in the `parser!` language.
//!
//! This module supports function overloading and should support user-defined
//! functions at some point, maybe.

#![allow(non_camel_case_types)]

use crate::{
    parsers::{self, EmptyParser, LineParser, RepeatParser},
    Parser,
};

pub trait ParserFunction<Args> {
    type Output;

    fn call_parser_function(&self, args: Args) -> Self::Output;
}

// `lines` needs to be something other than a plain Rust function, because the
// goal is to have it support a variable number of arguments.
pub struct lines;

// Just take one argument for now.
impl<'parse, 'source, T> ParserFunction<(T,)> for lines
where
    T: Parser<'parse, 'source>,
{
    type Output = RepeatParser<LineParser<T>, EmptyParser>;

    fn call_parser_function(&self, (line_parser,): (T,)) -> Self::Output {
        parsers::lines(line_parser)
    }
}

pub struct repeat_sep;

impl<'parse, 'source, T, U> ParserFunction<(T, U)> for repeat_sep
where
    T: Parser<'parse, 'source>,
    U: Parser<'parse, 'source>,
{
    type Output = RepeatParser<T, U>;

    fn call_parser_function(&self, (parser, sep): (T, U)) -> Self::Output {
        parsers::repeat(parser, sep, 0, None, false)
    }
}

pub struct line;

// impl ParserFunction<()> for line {
//     type Output = RegexParser<String, Never>;
//
//     fn call_parser_function(&self, (): ()) -> Self::Output {
//         RegexParser {
//             regex: crate::parsers::line_regex,
//             parse_fn: |s| Ok(s.to_string()),
//         }
//     }
// }

impl<'parse, 'source, T> ParserFunction<(T,)> for line
where
    T: Parser<'parse, 'source>,
{
    type Output = LineParser<T>;

    fn call_parser_function(&self, (parser,): (T,)) -> Self::Output {
        parsers::line(parser)
    }
}

// TODO: try implementing the trait for plain `fn` types.
