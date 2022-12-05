//! Parsing a repeated pattern.

use crate::{
    error::Result,
    parsers::{empty, EmptyParser},
    types::ParserOutput,
    ParseError, ParseIter, Parser,
};

#[derive(Clone, Copy)]
pub struct RepeatParser<Pattern, Sep> {
    pattern: Pattern,
    min: usize,
    max: Option<usize>,
    sep: Sep,
    sep_is_terminator: bool,
}

pub struct RepeatParseIter<'parse, 'source, Pattern, Sep>
where
    Pattern: Parser<'parse, 'source>,
    Sep: Parser<'parse, 'source>,
{
    source: &'source str,
    params: &'parse RepeatParser<Pattern, Sep>,
    pattern_iters: Vec<Pattern::Iter>,
    sep_iters: Vec<Sep::Iter>,
    starts: Vec<usize>,
}

impl<'parse, 'source, Pattern, Sep> Parser<'parse, 'source> for RepeatParser<Pattern, Sep>
where
    Pattern: Parser<'parse, 'source> + 'parse,
    Sep: Parser<'parse, 'source> + 'parse,
{
    type Output = Vec<Pattern::Output>;
    type RawOutput = (Vec<Pattern::Output>,);
    type Iter = RepeatParseIter<'parse, 'source, Pattern, Sep>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> Self::Iter {
        RepeatParseIter {
            source,
            params: self,
            pattern_iters: vec![self.pattern.parse_iter(source, start)],
            sep_iters: vec![],
            starts: vec![start],
        }
    }
}

impl<Pattern, Sep> RepeatParser<Pattern, Sep> {
    fn check_repeat_count(&self, count: usize) -> bool {
        let expected_parity = !self.sep_is_terminator as usize;
        let nmatches = (count + expected_parity) / 2;
        (count == 0 || count % 2 == expected_parity)
            && self.min <= nmatches
            && match self.max {
                None => true,
                Some(max) => nmatches <= max,
            }
    }
}

impl<'parse, 'source, Pattern, Sep> ParseIter for RepeatParseIter<'parse, 'source, Pattern, Sep>
where
    Pattern: Parser<'parse, 'source> + 'parse,
    Sep: Parser<'parse, 'source> + 'parse,
{
    type RawOutput = (Vec<Pattern::Output>,);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        // TODO: When considering creating a new iterator, if we have already
        // matched `max` times, don't bother; no matches can come of it.
        let mut foremost_error: Option<ParseError> = None;
        let mut got_iter = true;
        loop {
            assert_eq!(self.pattern_iters.len(), (self.starts.len() + 1) / 2);
            assert_eq!(self.sep_iters.len(), self.starts.len() / 2);
            if got_iter {
                let next_parse = if self.starts.is_empty() {
                    // No more iterators. We exhausted all possibilities.
                    return foremost_error.map(Err);
                } else if self.starts.len() % 2 == 1 {
                    self.pattern_iters.last_mut().unwrap().next_parse()
                } else {
                    self.sep_iters.last_mut().unwrap().next_parse()
                };

                // Get the next match.
                match next_parse {
                    None => {
                        got_iter = false;
                    }
                    Some(Err(err)) => {
                        got_iter = false;
                        if Some(err.location) > foremost_error.as_ref().map(|err| err.location) {
                            foremost_error = Some(err);
                        }
                    }
                    Some(Ok(point)) => {
                        // Got a match! But don't return it to the user yet.
                        // Repeats are "greedy"; we press on to see if we can
                        // match again! If we just matched `pattern`, try
                        // `sep`; if we just matched `sep`, try `pattern`.
                        self.starts.push(point);
                        if self.starts.len() % 2 == 1 {
                            self.pattern_iters
                                .push(self.params.pattern.parse_iter(self.source, point));
                        } else {
                            self.sep_iters
                                .push(self.params.sep.parse_iter(self.source, point));
                        }
                    }
                }
            } else {
                // The current top-of-stack iterator is exhausted and needs to
                // be discarded.
                if self.starts.len() % 2 == 1 {
                    self.pattern_iters.pop();
                } else {
                    self.sep_iters.pop();
                }
                let end = self.starts.pop().unwrap();
                got_iter = true;

                // Repeats are "greedy", so we need to yield the longest match
                // first. This means returning only "on the way out" (a
                // postorder walk of the tree of possible parses).
                if self.params.check_repeat_count(self.starts.len()) {
                    return Some(Ok(end));
                }
            }
        }
    }

    fn take_data(&mut self) -> (Vec<Pattern::Output>,) {
        self.starts.truncate(0);
        self.sep_iters.truncate(0);
        let v = self
            .pattern_iters
            .split_off(0)
            .iter_mut()
            .map(|iter| iter.take_data().into_user_type())
            .collect();
        (v,)
    }
}

pub fn repeat<Pattern, Sep>(
    pattern: Pattern,
    sep: Sep,
    min: usize,
    max: Option<usize>,
    sep_is_terminator: bool,
) -> RepeatParser<Pattern, Sep> {
    RepeatParser {
        pattern,
        min,
        max,
        sep,
        sep_is_terminator,
    }
}

// Kleene *
pub fn star<Pattern>(pattern: Pattern) -> RepeatParser<Pattern, EmptyParser> {
    repeat(pattern, empty(), 0, None, false)
}

// Kleene +
pub fn plus<Pattern>(pattern: Pattern) -> RepeatParser<Pattern, EmptyParser> {
    repeat(pattern, empty(), 1, None, false)
}

pub fn repeat_sep<Pattern, Sep>(pattern: Pattern, sep: Sep) -> RepeatParser<Pattern, Sep> {
    repeat(pattern, sep, 0, None, false)
}
