//! Parsers that parse lines or groups of lines: `line(p)`, `lines(p)`.

use crate::{
    error::Result,
    parsers::{star, EmptyParser, RepeatParser},
    types::ParserOutput,
    ParseError, ParseIter, Parser,
};

#[derive(Copy, Clone)]
pub struct LineParser<P> {
    parser: P,
}

impl<'parse, 'source, P> Parser<'parse, 'source> for LineParser<P>
where
    P: Parser<'parse, 'source> + 'parse,
{
    type RawOutput = (P::Output,);
    type Output = P::Output;
    type Iter = LineParseIter<'parse, 'source, P>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> Self::Iter {
        LineParseIter::Init {
            parser: self,
            source,
            start,
        }
    }
}

pub enum LineParseIter<'parse, 'source, P>
where
    P: Parser<'parse, 'source>,
{
    Init {
        parser: &'parse LineParser<P>,
        source: &'source str,
        start: usize,
    },
    Matched {
        iter: P::Iter,
    },
    Done,
}

fn is_at_line_start(source: &str, start: usize) -> bool {
    start == 0 || source[..start].ends_with('\n')
}

impl<'parse, 'source, P> ParseIter for LineParseIter<'parse, 'source, P>
where
    P: Parser<'parse, 'source>,
{
    type RawOutput = (P::Output,);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        match *self {
            LineParseIter::Init {
                parser,
                source,
                start,
            } => {
                if !is_at_line_start(source, start) {
                    *self = LineParseIter::Done;
                    return Some(Err(ParseError::new_bad_line_start(source, start)));
                }
                let end = match source[start..].find('\n') {
                    Some(i) => start + i,
                    None => {
                        *self = LineParseIter::Done;
                        return Some(Err(ParseError::new_expected(source, source.len(), "\n")));
                    }
                };
                let mut iter = parser.parser.parse_iter(&source[start..end], 0);
                let mut farthest = 0;
                loop {
                    match iter.next_parse() {
                        None => {
                            *self = LineParseIter::Done;
                            return Some(Err(ParseError::new_line_extra(source, start + farthest)));
                        }
                        Some(Err(err)) => {
                            *self = LineParseIter::Done;
                            return Some(Err(err));
                        }
                        Some(Ok(len)) if start + len == end => {
                            *self = LineParseIter::Matched { iter };
                            return Some(Ok(end + 1));
                        }
                        Some(Ok(len)) => farthest = farthest.max(len),
                    }
                }
            }
            _ => {
                *self = LineParseIter::Done;
                None
            }
        }
    }

    fn take_data(&mut self) -> Self::RawOutput {
        match self {
            LineParseIter::Matched { iter } => {
                let v = iter.take_data().into_user_type();
                *self = LineParseIter::Done;
                (v,)
            }
            _ => panic!("internal error: take_data called in invalid state"),
        }
    }
}

pub fn line<P>(parser: P) -> LineParser<P> {
    LineParser { parser }
}

pub fn lines<P>(parser: P) -> RepeatParser<LineParser<P>, EmptyParser> {
    star(line(parser))
}
