//! Parsers that parse lines or groups of lines: `line(p)`, `lines(p)`.

use std::marker::PhantomData;

use crate::{
    parsers::{star, EmptyParser, RepeatParser},
    types::ParserOutput,
    ParseContext, ParseError, ParseIter, Parser, Reported, Result,
};

/// This is implemented for `Line` and `Section`, the two region types.
pub trait Region: Copy + Clone {
    /// True if `start` is an offset within `source` that's the start of this
    /// type of region.
    ///
    /// # Panics
    ///
    /// This can panic if `start` is not a character boundary in `source`.
    fn check_at_start(context: &mut ParseContext, start: usize) -> Result<(), Reported>;

    /// If a suitable end is found for this region (`'\n'` or `/\Z/` for a line, `/^\n/`
    /// or `/\Z/` for a section) then return a pair of
    ///
    /// -   the end of the interior of the region, for the purpose of parsing the
    ///     interior; and
    /// -   the end of the delimiter, for the purpose of reporting how much data
    ///     we consumed on a successful parse.
    fn find_end(context: &mut ParseContext, start: usize) -> Result<(usize, usize), Reported>;

    /// Report an error to `context` indicating that we found a region and
    /// matched the text of the region to the expected subpattern, but the
    /// match doesn't cover the entire region.
    fn report_incomplete_match(context: &mut ParseContext, end: usize) -> Reported;
}

/// A line is a sequence of zero or more non-newline characters, starting
/// either at the beginning of the input or immediately after a newline;
/// followed by a single newline.
#[derive(Debug, Clone, Copy)]
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
        match source[start..].find('\n') {
            Some(offset) => Ok((start + offset, start + offset + 1)),
            None if start != source.len() => Ok((source.len(), source.len())),
            None => Err(context.error_expected(source.len(), "line")),
        }
    }

    fn report_incomplete_match(context: &mut ParseContext, end: usize) -> Reported {
        context.report(ParseError::new_line_extra(context.source(), end))
    }
}

/// A "section" is a sequence of zero or more nonblank lines, starting either
/// at the beginning of the input or immediately after a newline; followed by
/// either a blank line or the end of input.
#[derive(Debug, Clone, Copy)]
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
            None if start < source.len() => Ok((source.len(), source.len())),
            // no end-of-section delimiter found
            None => Err(context.error_expected(source.len(), "section")),
        }
    }

    fn report_incomplete_match(context: &mut ParseContext, end: usize) -> Reported {
        context.report(ParseError::new_section_extra(context.source(), end))
    }
}

/// Match but don't convert; just return the ParseIter on success. Expects all
/// of `source` to be matched, otherwise it's an error.
fn match_fully<'parse, R, P>(
    context: &mut ParseContext<'parse>,
    parser: &'parse P,
) -> Result<P::Iter<'parse>, Reported>
where
    R: Region,
    P: Parser,
{
    let source = context.source();
    let mut iter = parser.parse_iter(context, 0)?;
    while iter.match_end() != source.len() {
        R::report_incomplete_match(context, iter.match_end());
        iter.backtrack(context)?;
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

        let iter = context.with_slice(start, inner_end, |inner_context| {
            match_fully::<R, P>(inner_context, &self.parser)
        })?;
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

    fn convert(&self) -> Self::RawOutput {
        let v = self.iter.convert().into_user_type();
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

#[cfg(test)]
mod tests {
    use super::{line, section};
    use crate::prelude::u32;
    use crate::testing::*;

    #[test]
    fn test_newline_handling() {
        let p = line("hello world");
        assert_parse_eq(p, "hello world\n", ());
        assert_parse_eq(p, "hello world", ());
        assert_no_parse(p, "hello world\n\n");

        let p = sequence(line("dog"), line("cat"));
        assert_no_parse(p, "dog\n");
        assert_no_parse(p, "dogcat");
        assert_no_parse(p, "dogcat\n");
        assert_parse_eq(p, "dog\ncat", ((), ()));
        assert_parse_eq(p, "dog\ncat\n", ((), ()));

        let p = section(plus(line(u32)));
        assert_no_parse(p, "15\n16\n\n\n");
        assert_parse_eq(p, "15\n16\n\n", vec![15, 16]);
        assert_parse_eq(p, "15\n16\n", vec![15, 16]);
        assert_parse_eq(p, "15\n16", vec![15, 16]);

        let p = sequence(section(line("sec1")), section(line("sec2")));
        assert_parse_eq(p, "sec1\n\nsec2\n\n", ((), ()));
        assert_parse_eq(p, "sec1\n\nsec2\n", ((), ()));
        assert_parse_eq(p, "sec1\n\nsec2", ((), ()));
        assert_no_parse(p, "sec1\nsec2\n\n");
        assert_no_parse(p, "sec1\nsec2\n");
        assert_no_parse(p, "sec1\nsec2");
        assert_no_parse(p, "sec1sec2\n\n");
    }
}
