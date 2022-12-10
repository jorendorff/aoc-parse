//! Parsers that parse lines or groups of lines: `line(p)`, `lines(p)`.

use std::marker::PhantomData;

use crate::{
    parsers::{star, EmptyParser, RepeatParser},
    types::ParserOutput,
    ParseContext, ParseError, ParseIter, Parser, Reported, Result,
};

/// This is implemented for `Line` and `Section`, the two region types.
pub trait Region {
    /// True if `start` is an offset within `source` that's the start of this
    /// type of region. Caller promises that `start` is at least in bounds and
    /// a character boundary in `source`.
    fn check_at_start(context: &mut ParseContext, start: usize) -> Result<(), Reported>;

    /// If a suitable end is found for this region (`'\n'` for a line, `"\n\n"`
    /// or `/\n\Z/` for a section) then return a pair of
    ///
    /// -   the end of the interior of the region, for the purpose of parsing the
    ///     interior; and
    /// -   the end of the delimiter, for the purpose of reporting how much data
    ///     we consumed on a successful parse.
    fn find_end(context: &mut ParseContext, start: usize) -> Result<(usize, usize), Reported>;

    fn report_incomplete_match(context: &mut ParseContext, end: usize) -> Reported;
}

/// A line is a sequence of zero or more non-newline characters, starting
/// either at the beginning of the input or immediately after a newline;
/// followed by a single newline.
pub struct Line;

impl Region for Line {
    fn check_at_start(context: &mut ParseContext, start: usize) -> Result<(), Reported> {
        let source = context.source();
        if start == 0 || source[..start].ends_with('\n') {
            Ok(())
        } else {
            Err(context.report(ParseError::new_bad_line_start(source, start)))
        }
    }

    fn find_end(context: &mut ParseContext, start: usize) -> Result<(usize, usize), Reported> {
        let source = context.source();
        source[start..]
            .find('\n')
            .map(|offset| (start + offset, start + offset + 1))
            .ok_or_else(|| context.error_expected(source.len(), "\n"))
    }

    fn report_incomplete_match(context: &mut ParseContext, end: usize) -> Reported {
        context.report(ParseError::new_line_extra(context.source(), end))
    }
}

/// A "section" is a sequence of zero or more nonblank lines, starting either
/// at the beginning of the input or immediately after a newline; followed by
/// either a blank line or the end of input.
pub struct Section;

impl Region for Section {
    fn check_at_start(context: &mut ParseContext, start: usize) -> Result<(), Reported> {
        let source = context.source();
        if start == 0 || &source[..start] == "\n" || source[..start].ends_with("\n\n") {
            Ok(())
        } else {
            Err(context.report(ParseError::new_bad_section_start(source, start)))
        }
    }

    fn find_end(context: &mut ParseContext, start: usize) -> Result<(usize, usize), Reported> {
        // FIXME BUG: unclear what this should do when looking at an empty
        // section at end of input. presumably not repeat forever. (why does
        // this not always hang forever if you try to use `sections`?)
        let source = context.source();
        match source[start..].find("\n\n") {
            // ending at a blank line
            Some(index) => Ok((start + index + 1, start + index + 2)),
            // ending at the end of `source`
            None if start < source.len() && source.ends_with('\n') => {
                Ok((source.len(), source.len()))
            }
            // no end-of-section delimiter found
            None => Err(context.error_expected(source.len(), "\n")),
        }
    }

    fn report_incomplete_match(context: &mut ParseContext, end: usize) -> Reported {
        context.report(ParseError::new_section_extra(context.source(), end))
    }
}

/// Match but don't convert; just return the ParseIter on success. Expects all
/// of `source` to be matched, otherwise it's an error.
fn match_fully<'parse, R, P>(parser: &'parse P, source: &'parse str) -> Result<P::Iter<'parse>>
where
    R: Region,
    P: Parser,
{
    let mut context = ParseContext::new(source);
    let mut iter = match parser.parse_iter(&mut context, 0) {
        Ok(iter) => iter,
        Err(Reported) => return Err(context.into_reported_error()),
    };
    while iter.match_end() != source.len() {
        R::report_incomplete_match(&mut context, iter.match_end());
        if iter.backtrack(&mut context).is_err() {
            return Err(context.into_reported_error());
        }
    }
    Ok(iter)
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
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        R::check_at_start(context, start)?;
        let (inner_end, outer_end) = R::find_end(context, start)?;

        let iter = match_fully::<R, P>(&self.parser, &context.source()[start..inner_end])
            .map_err(|err| context.report(err.adjust_location(start)))?;

        Ok(RegionParseIter { iter, outer_end })
    }
}

pub struct RegionParseIter<'parse, P>
where
    P: Parser + 'parse,
{
    iter: P::Iter<'parse>,
    outer_end: usize,
}

impl<'parse, P> ParseIter<'parse> for RegionParseIter<'parse, P>
where
    P: Parser,
{
    type RawOutput = (P::Output,);

    fn match_end(&self) -> usize {
        self.outer_end
    }

    fn backtrack(&mut self, _context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        Err(Reported)
    }

    fn into_raw_output(self) -> Self::RawOutput {
        let v = self.iter.into_raw_output().into_user_type();
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
