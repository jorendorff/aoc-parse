//! Trait for functions in the `parser!` language.
//!
//! This module supports function overloading and should support user-defined
//! functions at some point, maybe.

#![allow(non_camel_case_types)]

use crate::{
    parsers::{self, EmptyParser, LineParser, RepeatParser, SectionParser, StringParser},
    Parser,
};

pub trait ParserFunction<Args> {
    type Output;

    fn call_parser_function(&self, args: Args) -> Self::Output;
}

pub struct line;

impl<T> ParserFunction<(T,)> for line
where
    T: Parser,
{
    type Output = LineParser<T>;

    fn call_parser_function(&self, (line_parser,): (T,)) -> Self::Output {
        parsers::line(line_parser)
    }
}

pub struct lines;

impl<T> ParserFunction<(T,)> for lines
where
    T: Parser,
{
    type Output = RepeatParser<LineParser<T>, EmptyParser>;

    fn call_parser_function(&self, (line_parser,): (T,)) -> Self::Output {
        parsers::lines(line_parser)
    }
}

pub struct section;

impl<T> ParserFunction<(T,)> for section
where
    T: Parser,
{
    type Output = SectionParser<T>;

    fn call_parser_function(&self, (section_parser,): (T,)) -> Self::Output {
        parsers::section(section_parser)
    }
}

pub struct sections;

impl<T> ParserFunction<(T,)> for sections
where
    T: Parser,
{
    type Output = RepeatParser<SectionParser<T>, EmptyParser>;

    fn call_parser_function(&self, (section_parser,): (T,)) -> Self::Output {
        parsers::sections(section_parser)
    }
}

pub struct repeat_sep;

impl<T, U> ParserFunction<(T, U)> for repeat_sep
where
    T: Parser,
    U: Parser,
{
    type Output = RepeatParser<T, U>;

    fn call_parser_function(&self, (parser, sep): (T, U)) -> Self::Output {
        parsers::repeat_sep(parser, sep)
    }
}

pub struct string;

impl<P> ParserFunction<(P,)> for string
where
    P: Parser,
{
    type Output = StringParser<P>;

    fn call_parser_function(&self, (parser,): (P,)) -> Self::Output {
        StringParser { parser }
    }
}

// TODO: try implementing the trait for plain `fn` types.
