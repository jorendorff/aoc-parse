use crate::matches::Match;
use crate::{ParseError, Result};

#[derive(Clone, Debug)]
pub struct Parser {
    parser_source: String,
    body: Box<ParserBody>,
}

#[derive(Clone, Debug)]
enum ParserBody {
    /// Parser that matches only one string.
    Exact(String),
    /// Match several patterns in sequence. Use `Sequence([])` for the empty pattern.
    Sequence(Vec<Parser>),
    /// Match any of several patterns. Use `OneOf([])` for a parser that never matches.
    OneOf(Vec<Parser>),
    /// Parse a repeating pattern.
    Repeat(Box<Repeat>),
}

#[derive(Clone, Debug)]
struct Repeat {
    pattern: Parser,
    min: usize,
    max: Option<usize>,
    sep: Parser,
    sep_is_terminator: bool,
}

impl Repeat {
    fn check_repeat_count(&self, count: usize) -> bool {
        let even_matches = count / 2;
        let expected_parity = !self.sep_is_terminator as usize;
        (count == 0 || count % 2 == expected_parity)
            && self.min <= even_matches
            && match self.max {
                None => true,
                Some(max) => even_matches <= max,
            }
    }
}

/// A parser in action. Some parsers can match in several different ways (for
/// example, in `foo* bar` backtracking is accomplished by `foo*` first
/// matching as much as possible, then backing off one match at a time), so
/// this is an iterator.
///
/// This doesn't return data from `next_parse` but instead waits until you're
/// sure you have a complete, successful parse, and are thus ready to destroy
/// the iterator. This is necessary because of Rust's ownership model; if we
/// returned the results every time we'd have to do O(n^2) cloning of long
/// vectors when backtracking. (Backtracking probably has terrible performance
/// anyway.)
trait ParseIter {
    fn next_parse(&mut self) -> Option<Result<usize>>;

    /// Consume this iterator to extract data. This would take `self` by value,
    /// except that's not compatible with trait objects.
    fn take_data(&mut self) -> Match;
}

struct OnceParseIter {
    location: Option<usize>,
    result: Option<Match>,
}

impl ParseIter for OnceParseIter {
    fn next_parse(&mut self) -> Option<Result<usize>> {
        self.location.take().map(|loc| Ok(loc))
    }

    fn take_data(&mut self) -> Match {
        self.result.take().unwrap()
    }
}

struct EmptyParseIter {
    error: ParseError,
}

impl ParseIter for EmptyParseIter {
    fn next_parse(&mut self) -> Option<Result<usize>> {
        Some(Err(self.error.clone()))
    }

    fn take_data(&mut self) -> Match {
        panic!("can't happen")
    }
}

struct OneOfParseIter<'p> {
    source: &'p str,
    start: usize,
    iter: Option<Box<dyn ParseIter + 'p>>,
    parsers: std::slice::Iter<'p, Parser>,
}

impl<'p> OneOfParseIter<'p> {
    fn new(source: &'p str, start: usize, parsers: &'p [Parser]) -> Self {
        OneOfParseIter {
            source,
            start,
            iter: None,
            parsers: parsers.iter(),
        }
    }
}

impl<'p> ParseIter for OneOfParseIter<'p> {
    fn next_parse(&mut self) -> Option<Result<usize>> {
        let mut foremost_error: Option<ParseError> = None;
        loop {
            if let Some(iter) = self.iter.as_mut() {
                match iter.next_parse() {
                    None => self.iter = None,
                    Some(Err(err)) => {
                        if Some(err.location) > foremost_error.as_ref().map(|err| err.location) {
                            foremost_error = Some(err);
                        }
                        self.iter = None
                    }
                    Some(Ok(end)) => return Some(Ok(end)),
                }
            } else if let Some(next_parser) = self.parsers.next() {
                self.iter = Some(next_parser.parse_iter(self.source, self.start));
            } else {
                return foremost_error.map(Err);
            }
        }
    }

    fn take_data(&mut self) -> Match {
        self.iter.take().unwrap().take_data()
    }
}

struct RepeatParseIter<'p> {
    source: &'p str,
    params: &'p Repeat,
    iters: Vec<Box<dyn ParseIter + 'p>>,
    starts: Vec<usize>,
}

impl<'p> RepeatParseIter<'p> {
    fn new(source: &'p str, start: usize, params: &'p Repeat) -> Self {
        RepeatParseIter {
            source,
            params,
            iters: vec![params.pattern.parse_iter(source, start)],
            starts: vec![start],
        }
    }
}

impl<'p> ParseIter for RepeatParseIter<'p> {
    fn next_parse(&mut self) -> Option<Result<usize>> {
        // TODO: When considering creating a new iterator, if we have already
        // matched `max` times, don't bother; no matches can come of it.
        let mut foremost_error: Option<ParseError> = None;
        let mut got_iter = true;
        loop {
            if got_iter {
                let last_iter = if let Some(iter) = self.iters.last_mut() {
                    iter
                } else {
                    // No more iterators. We exhausted all possibilities.
                    return foremost_error.map(Err);
                };

                // Get the next match.
                match last_iter.next_parse() {
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
                        let next_pat = if self.iters.len() % 2 == 0 {
                            &self.params.pattern
                        } else {
                            &self.params.sep
                        };
                        self.iters.push(next_pat.parse_iter(self.source, point));
                    }
                }
            } else {
                // The current top-of-stack iterators is exhausted and needs to be discarded.
                self.iters.pop();
                let end = self.starts.pop().unwrap();
                got_iter = true;

                // Repeats are "greedy", so we need to yield the longest match
                // first. This means returning only "on the way out" (a
                // postorder walk of the tree of possible parses).
                if self.params.check_repeat_count(self.iters.len()) {
                    return Some(Ok(end));
                }
            }
        }
    }

    fn take_data(&mut self) -> Match {
        Match::Array(
            self.iters
                .split_off(0)
                .into_iter()
                .step_by(2)
                .map(|mut iter| iter.take_data())
                .collect(),
        )
    }
}

struct SequenceParseIter<'p> {
    is_at_start: bool,
    source: &'p str,
    start: usize,
    parsers: &'p [Parser],
    iters: Vec<Box<dyn ParseIter + 'p>>,
}

impl<'p> SequenceParseIter<'p> {
    fn new(source: &'p str, start: usize, parsers: &'p [Parser]) -> Self {
        SequenceParseIter {
            is_at_start: true,
            source,
            start,
            parsers,
            iters: vec![],
        }
    }
}

impl<'p> ParseIter for SequenceParseIter<'p> {
    fn next_parse(&mut self) -> Option<Result<usize>> {
        let mut foremost_error: Option<ParseError> = None;
        let mut pump_existing_iter = !self.is_at_start;
        self.is_at_start = false;
        let mut position = self.start;
        loop {
            if pump_existing_iter {
                if let Some(iter) = self.iters.last_mut() {
                    match iter.next_parse() {
                        None => {
                            self.iters.pop();
                        }
                        Some(Err(err)) => {
                            if Some(err.location) > foremost_error.as_ref().map(|err| err.location)
                            {
                                foremost_error = Some(err);
                            }
                            self.iters.pop();
                        }
                        Some(Ok(end)) => {
                            position = end;
                            pump_existing_iter = false;
                        }
                    }
                } else {
                    // iters is empty. Failure.
                    return foremost_error.map(|err| Err(err));
                }
            } else if self.iters.len() < self.parsers.len() {
                let i = self.iters.len();
                let iter = self.parsers[i].parse_iter(self.source, position);
                self.iters.push(iter);
                pump_existing_iter = true;
            } else {
                // We have a complete set of matches.
                return Some(Ok(position));
            }
        }
    }

    fn take_data(&mut self) -> Match {
        Match::Tuple(
            self.iters
                .split_off(0)
                .into_iter()
                .map(|mut iter| iter.take_data())
                .collect(),
        )
    }
}

impl Parser {
    pub fn can_parse<T>(&self) -> Result<()> {
        todo!();
    }

    fn parse_iter<'p>(&'p self, s: &'p str, start: usize) -> Box<dyn ParseIter + 'p> {
        match &*self.body {
            ParserBody::Exact(expected) => {
                if s[start..].starts_with(expected) {
                    Box::new(OnceParseIter {
                        location: Some(start + expected.len()),
                        result: Some(Match::Exact(expected.clone())),
                    })
                } else {
                    Box::new(EmptyParseIter {
                        error: ParseError::new_expected(s, start, expected),
                    })
                }
            }
            ParserBody::Sequence(parsers) => Box::new(SequenceParseIter::new(s, start, parsers)),
            ParserBody::OneOf(arms) => Box::new(OneOfParseIter::new(s, start, arms)),
            ParserBody::Repeat(rep) => Box::new(RepeatParseIter::new(s, start, rep)),
        }
    }

    pub fn parse(&self, s: &str) -> Result<Match> {
        let mut it = self.parse_iter(s, 0);
        let mut best_end: Option<usize> = None;
        while let Some(parse) = it.next_parse() {
            let end = parse?;
            if end == s.len() {
                return Ok(it.take_data());
            } else {
                best_end = best_end.max(Some(end));
            }
        }
        if let Some(end) = best_end {
            Err(ParseError::new_extra(s, end))
        } else {
            panic!("parse iterator broke the contract: no matches and no error");
        }
    }

    #[doc(hidden)]
    pub fn with_source(mut self, parser_source: &'static str) -> Self {
        self.parser_source = parser_source.to_string();
        self
    }
}

pub fn empty() -> Parser {
    sequence([])
}

pub fn exact(s: &str) -> Parser {
    Parser {
        parser_source: format!("{:?}", s),
        body: Box::new(ParserBody::Exact(s.to_string())),
    }
}

pub fn sequence(parsers: impl IntoIterator<Item = Parser>) -> Parser {
    let parsers = parsers.into_iter().collect::<Vec<Parser>>();
    let parser_source = parsers
        .iter()
        .map(|p| &p.parser_source as &str)
        .collect::<Vec<&str>>()
        .join(" ");
    Parser {
        parser_source,
        body: Box::new(ParserBody::Sequence(parsers)),
    }
}

pub fn one_of(parsers: impl IntoIterator<Item = Parser>) -> Parser {
    let parsers = parsers.into_iter().collect::<Vec<Parser>>();
    let parser_source = "{\n".to_string()
        + &parsers
            .iter()
            .map(|p| p.parser_source.clone() + ",\n")
            .collect::<Vec<String>>()
            .join("")
        + "}\n";
    Parser {
        parser_source,
        body: Box::new(ParserBody::OneOf(parsers)),
    }
}

pub fn repeat(
    parser_source: String,
    pattern: Parser,
    sep: Parser,
    min: usize,
    max: Option<usize>,
    sep_is_terminator: bool,
) -> Parser {
    Parser {
        parser_source,
        body: Box::new(ParserBody::Repeat(Box::new(Repeat {
            pattern,
            min,
            max,
            sep,
            sep_is_terminator,
        }))),
    }
}

pub fn opt(pattern: Parser) -> Parser {
    one_of([pattern, empty()])
}

// Kleene *
pub fn star(pattern: Parser) -> Parser {
    repeat(
        pattern.parser_source.clone() + "*",
        pattern,
        empty(),
        0,
        None,
        false,
    )
}

// Kleene +
pub fn plus(pattern: Parser) -> Parser {
    repeat(
        pattern.parser_source.clone() + "+",
        pattern,
        empty(),
        1,
        None,
        false,
    )
}

pub fn sep_by(pattern: Parser, sep: Parser) -> Parser {
    repeat(
        format!("sep_by({}, {})", pattern.parser_source, sep.parser_source),
        pattern,
        sep,
        0,
        None,
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn assert_parse(parser: &Parser, s: &str) {
        if let Err(err) = parser.parse(s) {
            panic!("parse failed: {}", err);
        }
    }

    #[track_caller]
    fn assert_no_parse(parser: &Parser, s: &str) {
        if let Ok(m) = parser.parse(s) {
            panic!("expected no match, got: {:?}", m);
        }
    }

    #[test]
    fn test_parse() {
        let p = empty();
        assert_parse(&p, "");
        assert_no_parse(&p, "x");

        let p = exact("ok");
        assert_parse(&p, "ok");
        assert_no_parse(&p, "");
        assert_no_parse(&p, "o");
        assert_no_parse(&p, "nok");

        let p = sequence([exact("ok"), exact("go")]);
        assert_parse(&p, "okgo");
        assert_no_parse(&p, "ok");
        assert_no_parse(&p, "go");
        assert_no_parse(&p, "");

        let p = one_of([empty(), exact("ok")]);
        assert_parse(&p, "");
        assert_parse(&p, "ok");
        assert_no_parse(&p, "okc");
        assert_no_parse(&p, "okok");

        let p = star(exact("a"));
        assert_parse(&p, "");
        assert_parse(&p, "a");
        assert_parse(&p, "aa");
        assert_parse(&p, "aaa");
        assert_no_parse(&p, "b");
        assert_no_parse(&p, "ab");
        assert_no_parse(&p, "ba");

        let p = sep_by(exact("cow"), exact(","));
        assert_parse(&p, "");
        assert_parse(&p, "cow");
        assert_parse(&p, "cow,cow");
        assert_parse(&p, "cow,cow,cow");
        assert_no_parse(&p, "cowcow");
        assert_no_parse(&p, "cow,");
        assert_no_parse(&p, "cow,,cow");
        assert_no_parse(&p, "cow,cow,");
        assert_no_parse(&p, ",");
    }
}
