//! Parsing a repeated pattern.

use crate::{
    parsers::{empty, EmptyParser},
    types::ParserOutput,
    ParseContext, ParseIter, Parser, Result,
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
    params: &'parse RepeatParser<Pattern, Sep>,
    start: usize,
    pattern_iters: Vec<Pattern::Iter<'parse>>,
    sep_iters: Vec<Sep::Iter<'parse>>,
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
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>> {
        let mut iter = RepeatParseIter {
            params: self,
            start,
            pattern_iters: vec![],
            sep_iters: vec![],
        };
        if !iter.next(context, Mode::Advance) {
            return Err(context.error_extra(iter.end()));
        }
        Ok(iter)
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

// Internal state of the next() method.
enum Mode {
    BacktrackTopIter,
    Advance,
    Exhausted,
    YieldThenBacktrack,
}

impl<'parse, Pattern, Sep> RepeatParseIter<'parse, Pattern, Sep>
where
    Pattern: Parser,
    Sep: Parser,
{
    fn num_matches(&self) -> usize {
        self.pattern_iters.len() + self.sep_iters.len()
    }

    // True if we've matched as many separators as patterns, so pattern is next.
    fn is_pattern_next(&self) -> bool {
        self.pattern_iters.len() == self.sep_iters.len()
    }

    /// End position of what's been matched so far.
    fn end(&self) -> usize {
        if self.num_matches() == 0 {
            self.start
        } else if self.is_pattern_next() {
            self.sep_iters.last().unwrap().match_end()
        } else {
            self.pattern_iters.last().unwrap().match_end()
        }
    }

    /// Precondition: Either there are no iters or we just successfully
    /// backtracked the foremost iter.
    fn advance(&mut self, context: &mut ParseContext<'parse>) -> Result<()> {
        // TODO: When considering creating a new iterator, if we have already
        // matched `max` times, don't bother; no matches can come of it.
        let mut advanced = false;
        loop {
            assert_eq!(self.pattern_iters.len(), (self.num_matches() + 1) / 2);
            assert_eq!(self.sep_iters.len(), self.num_matches() / 2);

            if self.is_pattern_next() {
                let start = self.end();
                match self.params.pattern.parse_iter(context, start) {
                    Err(err) => {
                        if !advanced {
                            return Err(err);
                        } else {
                            return Ok(());
                        }
                    }
                    Ok(iter) => {
                        self.pattern_iters.push(iter);
                        advanced = true;
                    }
                }
            }

            let start = self.end();
            match self.params.sep.parse_iter(context, start) {
                Err(err) => {
                    if !advanced {
                        return Err(err);
                    } else {
                        return Ok(());
                    }
                }
                Ok(iter) => {
                    self.sep_iters.push(iter);
                    advanced = true;
                }
            }
        }
    }

    fn next(&mut self, context: &mut ParseContext<'parse>, mut mode: Mode) -> bool {
        loop {
            match mode {
                Mode::BacktrackTopIter => {
                    // Need to call backtrack() on the top iter. If that
                    // succeeds, advance again.
                    assert_eq!(self.pattern_iters.len(), (self.num_matches() + 1) / 2);
                    assert_eq!(self.sep_iters.len(), self.num_matches() / 2);

                    if self.num_matches() == 0 {
                        // No more iterators. We exhausted all possibilities.
                        return false;
                    }
                    let ok = if !self.is_pattern_next() {
                        self.pattern_iters.last_mut().unwrap().backtrack(context)
                    } else {
                        self.sep_iters.last_mut().unwrap().backtrack(context)
                    };

                    mode = if ok {
                        // Got a match! But don't return it to the user yet.
                        // Repeats are "greedy"; we press on to see if we can
                        // match again! If we just matched `pattern`, try
                        // `sep`; if we just matched `sep`, try `pattern`.
                        Mode::Advance
                    } else {
                        Mode::Exhausted
                    }
                }
                Mode::Advance => {
                    // Scan forward, hoping to find matches and create new
                    // iterators.
                    let _ = self.advance(context);
                    mode = Mode::YieldThenBacktrack;
                }
                Mode::Exhausted => {
                    // We just called backtrace() on the top iter, and it
                    // failed. It's exhausted and needs to be discarded.
                    assert_eq!(self.pattern_iters.len(), (self.num_matches() + 1) / 2);
                    assert_eq!(self.sep_iters.len(), self.num_matches() / 2);

                    if self.is_pattern_next() {
                        self.sep_iters.pop();
                    } else {
                        self.pattern_iters.pop();
                    }
                    mode = Mode::YieldThenBacktrack;
                }

                Mode::YieldThenBacktrack => {
                    // We just either popped an exhausted iterator, or failed
                    // to create one. If the current status is an overall
                    // match, yield that. Then transition to BacktrackTopIter
                    // mode.
                    //
                    // (Repeats are "greedy", so we need to yield the longest match
                    // first. This means returning only "on the way out", a
                    // postorder walk of the tree of possible parses.)
                    if self.params.check_repeat_count(self.num_matches()) {
                        return true;
                    }
                    mode = Mode::BacktrackTopIter;
                }
            }
        }
    }
}

impl<'parse, Pattern, Sep> ParseIter<'parse> for RepeatParseIter<'parse, Pattern, Sep>
where
    Pattern: Parser,
    Sep: Parser,
{
    type RawOutput = (Vec<Pattern::Output>,);

    fn match_end(&self) -> usize {
        self.end()
    }

    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> bool {
        self.next(context, Mode::BacktrackTopIter)
    }

    fn into_raw_output(self) -> (Vec<Pattern::Output>,) {
        let v = self
            .pattern_iters
            .into_iter()
            .map(|iter| iter.into_raw_output().into_user_type())
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
