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

pub struct RepeatParseIter<'parse, Pattern, Sep>
where
    Pattern: Parser + 'parse,
    Sep: Parser + 'parse,
{
    source: &'parse str,
    params: &'parse RepeatParser<Pattern, Sep>,
    pattern_iters: Vec<Pattern::Iter<'parse>>,
    sep_iters: Vec<Sep::Iter<'parse>>,
    starts: Vec<usize>,
}

impl<Pattern, Sep> Parser for RepeatParser<Pattern, Sep>
where
    Pattern: Parser,
    Sep: Parser,
{
    type Output = Vec<Pattern::Output>;
    type RawOutput = (Vec<Pattern::Output>,);
    type Iter<'parse> = RepeatParseIter<'parse, Pattern, Sep>
    where
        Pattern: 'parse,
        Sep: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        Ok(RepeatParseIter {
            source,
            params: self,
            pattern_iters: vec![],
            sep_iters: vec![],
            starts: vec![start],
        })
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

impl<'parse, Pattern, Sep> ParseIter for RepeatParseIter<'parse, Pattern, Sep>
where
    Pattern: Parser,
    Sep: Parser,
{
    type RawOutput = (Vec<Pattern::Output>,);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        // TODO: When considering creating a new iterator, if we have already
        // matched `max` times, don't bother; no matches can come of it.
        enum Mode {
            Forward,
            NewIter,
            Exhausted,
            Backtrack,
        }

        let mut mode = if self.pattern_iters.is_empty() && self.starts.len() == 1 {
            Mode::NewIter
        } else {
            Mode::Forward
        };

        let mut foremost_error: Option<ParseError> = None;
        loop {
            match mode {
                Mode::Forward => {
                    assert_eq!(self.pattern_iters.len(), (self.starts.len() + 1) / 2);
                    assert_eq!(self.sep_iters.len(), self.starts.len() / 2);

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
                            mode = Mode::Exhausted;
                        }
                        Some(Err(err)) => {
                            ParseError::keep_best(&mut foremost_error, err);
                            mode = Mode::Exhausted;
                        }
                        Some(Ok(point)) => {
                            // Got a match! But don't return it to the user yet.
                            // Repeats are "greedy"; we press on to see if we can
                            // match again! If we just matched `pattern`, try
                            // `sep`; if we just matched `sep`, try `pattern`.
                            self.starts.push(point);
                            mode = Mode::NewIter;
                        }
                    }
                }
                Mode::NewIter => {
                    let point = self.starts.last().copied().unwrap();
                    if self.pattern_iters.is_empty() || self.starts.len() % 2 == 1 {
                        match self.params.pattern.parse_iter(self.source, point) {
                            Err(err) => {
                                ParseError::keep_best(&mut foremost_error, err);
                                mode = Mode::Backtrack;
                            }
                            Ok(iter) => {
                                self.pattern_iters.push(iter);
                                mode = Mode::Forward;
                            }
                        }
                    } else {
                        match self.params.sep.parse_iter(self.source, point) {
                            Err(err) => {
                                ParseError::keep_best(&mut foremost_error, err);
                                mode = Mode::Backtrack;
                            }
                            Ok(iter) => {
                                self.sep_iters.push(iter);
                                mode = Mode::Forward;
                            }
                        }
                    }
                }
                Mode::Exhausted => {
                    // The current top-of-stack iterator is exhausted and needs to
                    // be discarded.
                    assert_eq!(self.pattern_iters.len(), (self.starts.len() + 1) / 2);
                    assert_eq!(self.sep_iters.len(), self.starts.len() / 2);

                    if self.starts.len() % 2 == 1 {
                        self.pattern_iters.pop();
                    } else {
                        self.sep_iters.pop();
                    }
                    mode = Mode::Backtrack;
                }

                Mode::Backtrack => {
                    // Repeats are "greedy", so we need to yield the longest match
                    // first. This means returning only "on the way out" (a
                    // postorder walk of the tree of possible parses).
                    let end = self.starts.pop().unwrap();
                    if self.params.check_repeat_count(self.starts.len()) {
                        return Some(Ok(end));
                    }
                    mode = Mode::Forward;
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

// Used by the `parser!()` macro to implement the `*` quantifier.
#[doc(hidden)]
pub fn star<Pattern>(pattern: Pattern) -> RepeatParser<Pattern, EmptyParser> {
    repeat(pattern, empty(), 0, None, false)
}

// Used by the `parser!()` macro to implement the `+` quantifier.
#[doc(hidden)]
pub fn plus<Pattern>(pattern: Pattern) -> RepeatParser<Pattern, EmptyParser> {
    repeat(pattern, empty(), 1, None, false)
}

/// <code>repeat_sep(<var>pattern</var>, <var>separator</var>)</code> matches
/// the given *pattern* any number of times, separated by the *separator*. For
/// example, `parser!(repeat_sep(i32, ","))` matches a list of comma-separated
/// integers.
///
/// This converts only the bits that match *pattern* to Rust values, producing
/// a `Vec`. Any parts of the string matched by *separator* are not converted.
pub fn repeat_sep<Pattern, Sep>(pattern: Pattern, sep: Sep) -> RepeatParser<Pattern, Sep> {
    repeat(pattern, sep, 0, None, false)
}
