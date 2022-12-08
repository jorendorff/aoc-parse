//! Parsers that parse lines or groups of lines: `line(p)`, `lines(p)`.

use std::marker::PhantomData;

use crate::{
    error::Result,
    parsers::{star, EmptyParser, RepeatParser},
    types::ParserOutput,
    ParseError, ParseIter, Parser,
};

/// This is implemented for `Line` and `Section`, the two region types.
pub trait Region {
    /// True if `start` is an offset within `source` that's the start of this
    /// type of region. Caller promises that `start` is at least in bounds and
    /// a character boundary in `source`.
    fn is_at_start(source: &str, start: usize) -> bool;

    /// If a suitable end is found for this region (`'\n'` for a line, `"\n\n"`
    /// or `/\n\Z/` for a section) then return a pair of
    ///
    /// -   the end of the interior of the region, for the purpose of parsing the
    ///     interior; and
    /// -   the end of the delimiter, for the purpose of reporting how much data
    ///     we consumed on a successful parse.
    fn find_end(source: &str, start: usize) -> Option<(usize, usize)>;

    fn not_at_start_err(source: &str, start: usize) -> ParseError;

    fn extra_err(source: &str, end: usize) -> ParseError;
}

/// A line is a sequence of zero or more non-newline characters, starting
/// either at the beginning of the input or immediately after a newline;
/// followed by a single newline.
pub struct Line;

impl Region for Line {
    fn is_at_start(source: &str, start: usize) -> bool {
        start == 0 || source[..start].ends_with('\n')
    }

    fn find_end(source: &str, start: usize) -> Option<(usize, usize)> {
        source[start..]
            .find('\n')
            .map(|offset| (start + offset, start + offset + 1))
    }

    fn not_at_start_err(source: &str, start: usize) -> ParseError {
        ParseError::new_bad_line_start(source, start)
    }

    fn extra_err(source: &str, end: usize) -> ParseError {
        ParseError::new_line_extra(source, end)
    }
}

/// A "section" is a sequence of zero or more nonblank lines, starting either
/// at the beginning of the input or immediately after a newline; followed by
/// either a blank line or the end of input.
pub struct Section;

impl Region for Section {
    fn is_at_start(source: &str, start: usize) -> bool {
        start == 0 || &source[..start] == "\n" || source[..start].ends_with("\n\n")
    }

    fn find_end(source: &str, start: usize) -> Option<(usize, usize)> {
        // FIXME BUG: unclear what this should do when looking at an empty
        // section at end of input. presumably not repeat forever.
        match source[start..].find("\n\n") {
            // ending at a blank line
            Some(index) => Some((start + index + 1, start + index + 2)),
            // ending at the end of `source`
            None if start < source.len() && source.ends_with('\n') => {
                Some((source.len(), source.len()))
            }
            // no end-of-section delimiter found
            None => None,
        }
    }

    fn not_at_start_err(source: &str, start: usize) -> ParseError {
        ParseError::new_bad_section_start(source, start)
    }

    fn extra_err(source: &str, end: usize) -> ParseError {
        ParseError::new_section_extra(source, end)
    }
}

/// Match but don't convert; just return the ParseIter on success. Expects all
/// of `source` to be matched, otherwise it's an error.
fn match_fully<'parse, R, P>(parser: &'parse P, source: &'parse str) -> Result<P::Iter<'parse>>
where
    R: Region,
    P: Parser,
{
    let mut iter = parser.parse_iter(source, 0)?;
    let mut farthest = 0;
    loop {
        match iter.next_parse() {
            None => {
                return Err(R::extra_err(source, farthest));
            }
            Some(Err(err)) => return Err(err),
            Some(Ok(len)) if len == source.len() => {
                return Ok(iter);
            }
            Some(Ok(len)) => farthest = farthest.max(len),
        }
    }
}

#[derive(Copy, Clone)]
pub struct RegionParser<R: Region, P> {
    parser: P,
    phantom: PhantomData<fn() -> R>,
}

impl<R, P> Parser for RegionParser<R, P>
where
    R: Region,
    P: Parser,
{
    type RawOutput = (P::Output,);
    type Output = P::Output;
    type Iter<'parse> = RegionParseIter<'parse, P>
    where
        R: 'parse,
        P: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        if !R::is_at_start(source, start) {
            return Err(R::not_at_start_err(source, start));
        }
        let (inner_end, outer_end) = match R::find_end(source, start) {
            Some(pair) => pair,
            None => {
                return Err(ParseError::new_expected(source, source.len(), "\n"));
            }
        };

        let iter = match_fully::<R, P>(&self.parser, &source[start..inner_end])
            .map_err(|err| err.adjust_location(start))?;

        Ok(RegionParseIter {
            iter,
            outer_end,
            done: false,
        })
    }
}

pub struct RegionParseIter<'parse, P>
where
    P: Parser + 'parse,
{
    iter: P::Iter<'parse>,
    outer_end: usize,
    done: bool,
}

impl<'parse, P> ParseIter for RegionParseIter<'parse, P>
where
    P: Parser,
{
    type RawOutput = (P::Output,);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        if self.done {
            None
        } else {
            self.done = true;
            Some(Ok(self.outer_end))
        }
    }

    fn take_data(&mut self) -> Self::RawOutput {
        let v = self.iter.take_data().into_user_type();
        (v,)
    }
}

pub type LineParser<P> = RegionParser<Line, P>;
pub type SectionParser<P> = RegionParser<Section, P>;

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
pub fn line<P>(parser: P) -> LineParser<P> {
    LineParser {
        parser,
        phantom: PhantomData,
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
pub fn lines<P>(parser: P) -> RepeatParser<LineParser<P>, EmptyParser> {
    star(line(parser))
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
pub fn section<P>(parser: P) -> SectionParser<P> {
    SectionParser {
        parser,
        phantom: PhantomData,
    }
}

/// <code>sections(<var>pattern</var>)</code> matches any number of sections
/// matching *pattern*. Equivalent to
/// <code>section(<var>pattern</var>)*</code>.
pub fn sections<P>(parser: P) -> RepeatParser<SectionParser<P>, EmptyParser> {
    star(section(parser))
}
