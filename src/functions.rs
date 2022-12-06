//! Trait for functions in the `parser!` language.
//!
//! This module supports function overloading and should support user-defined
//! functions at some point, maybe.

#![allow(non_camel_case_types)]

use crate::{
    parsers::{self, EmptyParser, LineParser, RepeatParser, SectionParser, StringParser},
    Parser,
};

/// Temporary feature used by the `parser!()` macro to implement function call syntax.
/// This trait will probably be dropped in favor of just using regular Rust functions.
pub trait ParserFunction<Args> {
    /// The return type.
    type Output;

    /// Call the function with the given `args`.
    fn call_parser_function(&self, args: Args) -> Self::Output;
}

/// <code>line(<var>pattern</var>)</code> matches a single line of text that
/// matches *pattern*, and the newline at the end of the line.
///
/// This is like <code>^<var>pattern</var>\n</code> in regular expressions,
/// except <code>line(<var>pattern</var>)</code> will only ever match exactly
/// one line of text, even if *pattern* could match more newlines.
///
/// `line(string(any_char+))` matches a line of text, strips off the newline
/// character, and returns the rest as a `String`.
///
/// `line("")` matches a blank line.
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

/// <code>lines(<var>pattern</var>)</code> matches any number of lines of text
/// matching *pattern*. Each line must be terminated by a newline, `'\n'`.
///
/// Equivalent to <code>line(<var>pattern</var>)*</code>.
///
/// ```
/// # use aoc_parse::{parser, prelude::*};
/// let p = parser!(lines(repeat_sep(digit, " ")));
/// assert_eq!(
///     p.parse("1 2 3\n4 5 6\n").unwrap(),
///     vec![vec![1, 2, 3], vec![4, 5, 6]],
/// );
/// ```
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

/// <code>section(<var>pattern</var>)</code> matches zero or more nonblank
/// lines, followed by either a blank line or the end of input. The nonblank
/// lines must match *pattern*.
///
/// `section()` consumes the blank line. *pattern* should not expect to see it.
///
/// It's common for an AoC puzzle input to have several lines of data, then a
/// blank line, and then a different kind of data. You can parse this with
/// <code>section(<var>p1</var>) section(<var>p2</var>)</code>.
///
/// `section(lines(u64))` matches a section that's a list of numbers, one per
/// line.
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

/// <code>sections(<var>pattern</var>)</code> - Matches any number of sections
/// matching *pattern*. Equivalent to <code>section(<var>pattern</var>)*</code>
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

/// <code>repeat_sep(<var>pattern</var>, <var>separator</var>)</code> matches
/// the given *pattern* any number of times, separated by the *separator*. For
/// example, `parser!(repeat_sep(i32, ","))` matches a list of comma-separated
/// integers.
///
/// This converts only the bits that match *pattern* to Rust values, producing
/// a `Vec`. Any parts of the string matched by *separator* are not converted.
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

/// <code>string(<var>pattern</var>)</code> - Matches the given *pattern*, but
/// instead of converting it to some value, simply return the matched
/// characters as a `String`.
///
/// By default, `alpha+` returns a `Vec<char>`, and sometimes that is handy in
/// AoC, but often it's better to have it return a `String`.
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
