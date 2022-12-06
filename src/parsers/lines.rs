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
    type Iter<'parse> = RegionParseIter<'parse, R, P>
    where
        R: 'parse,
        P: 'parse;

    fn parse_iter<'parse>(&'parse self, source: &'parse str, start: usize) -> Self::Iter<'parse> {
        RegionParseIter::Init {
            parser: self,
            source,
            start,
        }
    }
}

pub enum RegionParseIter<'parse, R, P>
where
    R: Region + 'parse,
    P: Parser + 'parse,
{
    Init {
        parser: &'parse RegionParser<R, P>,
        source: &'parse str,
        start: usize,
    },
    Matched {
        iter: P::Iter<'parse>,
    },
    Done,
}

impl<'parse, R, P> ParseIter for RegionParseIter<'parse, R, P>
where
    R: Region,
    P: Parser,
{
    type RawOutput = (P::Output,);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        match *self {
            RegionParseIter::Init {
                parser,
                source,
                start,
            } => {
                if !R::is_at_start(source, start) {
                    *self = RegionParseIter::Done;
                    return Some(Err(R::not_at_start_err(source, start)));
                }
                let (inner_end, outer_end) = match R::find_end(source, start) {
                    Some(pair) => pair,
                    None => {
                        *self = RegionParseIter::Done;
                        return Some(Err(ParseError::new_expected(source, source.len(), "\n")));
                    }
                };
                let mut iter = parser.parser.parse_iter(&source[start..inner_end], 0);
                let mut farthest = 0;
                loop {
                    match iter.next_parse() {
                        None => {
                            *self = RegionParseIter::Done;
                            return Some(Err(R::extra_err(source, start + farthest)));
                        }
                        Some(Err(mut err)) => {
                            *self = RegionParseIter::Done;
                            err.adjust_location(start);
                            return Some(Err(err));
                        }
                        Some(Ok(len)) if start + len == inner_end => {
                            *self = RegionParseIter::Matched { iter };
                            return Some(Ok(outer_end));
                        }
                        Some(Ok(len)) => farthest = farthest.max(len),
                    }
                }
            }
            _ => {
                *self = RegionParseIter::Done;
                None
            }
        }
    }

    fn take_data(&mut self) -> Self::RawOutput {
        match self {
            RegionParseIter::Matched { iter } => {
                let v = iter.take_data().into_user_type();
                *self = RegionParseIter::Done;
                (v,)
            }
            _ => panic!("internal error: take_data called in invalid state"),
        }
    }
}

pub type LineParser<P> = RegionParser<Line, P>;
pub type SectionParser<P> = RegionParser<Section, P>;

// Used to implement `aoc_parse::functions::line`.
#[doc(hidden)]
pub fn line<P>(parser: P) -> LineParser<P> {
    LineParser {
        parser,
        phantom: PhantomData,
    }
}

// Used to implement `aoc_parse::functions::lines`.
#[doc(hidden)]
pub fn lines<P>(parser: P) -> RepeatParser<LineParser<P>, EmptyParser> {
    star(line(parser))
}

// Used to implement `aoc_parse::functions::section`.
#[doc(hidden)]
pub fn section<P>(parser: P) -> SectionParser<P> {
    SectionParser {
        parser,
        phantom: PhantomData,
    }
}

// Used to implement `aoc_parse::functions::sections`.
#[doc(hidden)]
pub fn sections<P>(parser: P) -> RepeatParser<SectionParser<P>, EmptyParser> {
    star(section(parser))
}
