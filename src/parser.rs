use crate::{ParseError, Result};

pub trait Parser<'parse, 'source> {
    type Output;
    type Iter: ParseIter<Output = Self::Output>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> Self::Iter;

    fn parse(&'parse self, s: &'source str) -> Result<Self::Output> {
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

    fn map<T, F>(self, mapper: F) -> MapParser<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> T,
    {
        MapParser {
            parser: self,
            mapper,
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
pub trait ParseIter {
    type Output;

    fn next_parse(&mut self) -> Option<Result<usize>>;

    /// Consume this iterator to extract data. This would take `self` by value,
    /// except that's not compatible with trait objects.
    fn take_data(&mut self) -> Self::Output;
}

// --- Parser that successfully matches the empty string

pub struct EmptyParser;

impl<'parse, 'source> Parser<'parse, 'source> for EmptyParser {
    type Output = ();
    type Iter = EmptyParseIter;

    fn parse_iter(&'parse self, _source: &'source str, start: usize) -> EmptyParseIter {
        EmptyParseIter {
            used: false,
            location: start,
        }
    }
}

pub struct EmptyParseIter {
    used: bool,
    location: usize,
}

impl ParseIter for EmptyParseIter {
    type Output = ();

    fn next_parse(&mut self) -> Option<Result<usize>> {
        if self.used {
            None
        } else {
            self.used = true;
            Some(Ok(self.location))
        }
    }

    fn take_data(&mut self) {}
}

// --- Parser that never matches anything

#[derive(Debug)]
pub enum Never {}

pub struct NeverParser;

pub struct NeverParseIter<'source> {
    source: &'source str,
    start: usize,
}

impl<'parse, 'source> Parser<'parse, 'source> for NeverParser {
    type Output = Never;
    type Iter = NeverParseIter<'source>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> NeverParseIter<'source> {
        NeverParseIter { source, start }
    }
}

impl<'source> ParseIter for NeverParseIter<'source> {
    type Output = Never;

    fn next_parse(&mut self) -> Option<Result<usize>> {
        Some(Err(ParseError::new_cannot_match(self.source, self.start)))
    }

    fn take_data(&mut self) -> Never {
        unreachable!("never matches");
    }
}

// --- Parser that matches a particular exact string

pub struct ExactParser {
    s: String,
}

pub struct ExactParseIter<'parse, 'source> {
    expected: &'parse str,
    input: &'source str,
    start: usize,
    done: bool,
}

impl<'parse, 'source> Parser<'parse, 'source> for ExactParser {
    type Output = ();
    type Iter = ExactParseIter<'parse, 'source>;

    fn parse_iter(
        &'parse self,
        source: &'source str,
        start: usize,
    ) -> ExactParseIter<'parse, 'source> {
        ExactParseIter {
            expected: &self.s,
            input: source,
            start,
            done: false,
        }
    }
}

impl<'parse, 'source> ParseIter for ExactParseIter<'parse, 'source> {
    type Output = ();

    fn next_parse(&mut self) -> Option<Result<usize>> {
        if self.done {
            None
        } else if self.input[self.start..].starts_with(self.expected) {
            self.done = true;
            Some(Ok(self.start + self.expected.len()))
        } else {
            Some(Err(ParseError::new_expected(
                self.input,
                self.start,
                self.expected,
            )))
        }
    }

    fn take_data(&mut self) {}
}

// --- Matching patterns in sequence

pub struct SequenceParser<Head, Tail> {
    head: Head,
    tail: Tail,
}

pub struct SequenceParseIter<'parse, 'source, Head, Tail>
where
    Head: Parser<'parse, 'source>,
    Tail: Parser<'parse, 'source>,
{
    parsers: &'parse SequenceParser<Head, Tail>,
    is_at_start: bool,
    source: &'source str,
    start: usize,
    head_iter: Option<Head::Iter>,
    tail_iter: Option<Tail::Iter>,
}

impl<'parse, 'source, Head, Tail> Parser<'parse, 'source> for SequenceParser<Head, Tail>
where
    Head: Parser<'parse, 'source> + 'parse,
    Tail: Parser<'parse, 'source> + 'parse,
{
    type Output = (Head::Output, Tail::Output);
    type Iter = SequenceParseIter<'parse, 'source, Head, Tail>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> Self::Iter {
        SequenceParseIter {
            parsers: self,
            is_at_start: true,
            source,
            start,
            head_iter: None,
            tail_iter: None,
        }
    }
}

impl<'parse, 'source, Head, Tail> ParseIter for SequenceParseIter<'parse, 'source, Head, Tail>
where
    Head: Parser<'parse, 'source>,
    Tail: Parser<'parse, 'source>,
{
    type Output = (Head::Output, Tail::Output);

    fn next_parse(&mut self) -> Option<Result<usize>> {
        let mut foremost_error: Option<ParseError> = None;
        loop {
            if let Some(tail_iter) = &mut self.tail_iter {
                match tail_iter.next_parse() {
                    None => {}
                    Some(Err(err)) => {
                        if foremost_error.as_ref().map(|err| err.location) < Some(err.location) {
                            foremost_error = Some(err);
                        }
                    }

                    Some(Ok(tail_end)) => return Some(Ok(tail_end)),
                }
                self.tail_iter = None;
            } else if let Some(head_iter) = &mut self.head_iter {
                match head_iter.next_parse() {
                    None => {}
                    Some(Err(err)) => {
                        if foremost_error.as_ref().map(|err| err.location) < Some(err.location) {
                            foremost_error = Some(err);
                        }
                    }
                    Some(Ok(head_end)) => {
                        self.tail_iter = Some(self.parsers.tail.parse_iter(self.source, head_end));
                        continue;
                    }
                }
                self.head_iter = None;
                return foremost_error.map(Err);
            } else if self.is_at_start {
                self.is_at_start = false;
                self.head_iter = Some(self.parsers.head.parse_iter(self.source, self.start));
            } else {
                return None;
            }
        }
    }

    fn take_data(&mut self) -> (Head::Output, Tail::Output) {
        let head = self.head_iter.as_mut().unwrap().take_data();
        let tail = self.tail_iter.as_mut().unwrap().take_data();
        (head, tail)
    }
}

// --- Alternation

#[derive(Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

pub struct EitherParser<A, B> {
    left: A,
    right: B,
}

pub struct EitherParseIter<'parse, 'source, A, B>
where
    A: Parser<'parse, 'source>,
    B: Parser<'parse, 'source>,
{
    source: &'source str,
    start: usize,
    parsers: &'parse EitherParser<A, B>,
    iter: Either<A::Iter, B::Iter>,
}

impl<'parse, 'source, A, B> Parser<'parse, 'source> for EitherParser<A, B>
where
    A: Parser<'parse, 'source> + 'parse,
    B: Parser<'parse, 'source> + 'parse,
{
    type Iter = EitherParseIter<'parse, 'source, A, B>;

    type Output = Either<A::Output, B::Output>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> Self::Iter {
        EitherParseIter {
            source,
            start,
            parsers: self,
            iter: Either::Left(self.left.parse_iter(source, start)),
        }
    }
}

impl<'parse, 'source, A, B> ParseIter for EitherParseIter<'parse, 'source, A, B>
where
    A: Parser<'parse, 'source>,
    B: Parser<'parse, 'source>,
{
    type Output = Either<A::Output, B::Output>;

    fn next_parse(&mut self) -> Option<Result<usize>> {
        let mut foremost_error: Option<ParseError> = None;
        loop {
            match &mut self.iter {
                Either::Left(iter) => {
                    match iter.next_parse() {
                        None => {}
                        Some(Err(err)) => {
                            if Some(err.location) > foremost_error.as_ref().map(|err| err.location)
                            {
                                foremost_error = Some(err);
                            }
                        }
                        Some(Ok(end)) => return Some(Ok(end)),
                    }
                    self.iter =
                        Either::Right(self.parsers.right.parse_iter(self.source, self.start));
                }
                Either::Right(iter) => {
                    match iter.next_parse() {
                        None => {}
                        Some(Err(err)) => {
                            if Some(err.location) > foremost_error.as_ref().map(|err| err.location)
                            {
                                foremost_error = Some(err);
                            }
                        }
                        Some(Ok(end)) => return Some(Ok(end)),
                    }
                    return foremost_error.map(Err);
                }
            }
        }
    }

    fn take_data(&mut self) -> Either<A::Output, B::Output> {
        match &mut self.iter {
            Either::Left(iter) => Either::Left(iter.take_data()),
            Either::Right(iter) => Either::Right(iter.take_data()),
        }
    }
}

// --- Parsing a repeated pattern

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
    type Output = Vec<Pattern::Output>;

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

    fn take_data(&mut self) -> Vec<Pattern::Output> {
        self.starts.truncate(0);
        self.sep_iters.truncate(0);
        self.pattern_iters
            .split_off(0)
            .iter_mut()
            .map(|iter| iter.take_data())
            .collect()
    }
}

// --- Mapping parser

pub struct MapParser<P, F> {
    parser: P,
    mapper: F,
}

pub struct MapParseIter<'parse, 'source, P, F>
where
    P: Parser<'parse, 'source>,
{
    iter: P::Iter,
    mapper: &'parse F,
}

impl<'parse, 'source, P, F, T> Parser<'parse, 'source> for MapParser<P, F>
where
    P: Parser<'parse, 'source>,
    F: Fn(P::Output) -> T,
    F: 'parse,
{
    type Output = T;
    type Iter = MapParseIter<'parse, 'source, P, F>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> Self::Iter {
        MapParseIter {
            iter: self.parser.parse_iter(source, start),
            mapper: &self.mapper,
        }
    }
}

impl<'parse, 'source, P, F, T> ParseIter for MapParseIter<'parse, 'source, P, F>
where
    P: Parser<'parse, 'source>,
    F: Fn(P::Output) -> T,
{
    type Output = T;

    fn next_parse(&mut self) -> Option<Result<usize>> {
        self.iter.next_parse()
    }

    fn take_data(&mut self) -> Self::Output {
        (self.mapper)(self.iter.take_data())
    }
}

// --- Wrappers

pub fn empty() -> EmptyParser {
    EmptyParser
}

pub fn exact(s: &str) -> ExactParser {
    ExactParser { s: s.to_string() }
}

pub fn sequence<Head, Tail>(head: Head, tail: Tail) -> SequenceParser<Head, Tail> {
    SequenceParser { head, tail }
}

// pub fn sequence(parsers: impl ParserTuple) -> impl for<'parse, 'source> Parser<'parse, 'source> {
//     parsers.product()
// }

pub fn either<A, B>(
    left: impl for<'parse, 'source> Parser<'parse, 'source, Output = A> + 'static,
    right: impl for<'parse, 'source> Parser<'parse, 'source, Output = B> + 'static,
) -> impl for<'parse, 'source> Parser<'parse, 'source, Output = Either<A, B>> {
    EitherParser { left, right }
}

// pub fn one_of(parsers: impl ParserTuple) -> impl for<'parse, 'source> Parser<'parse, 'source> {
//     parsers.sum()
// }

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

pub fn opt<T>(
    pattern: impl for<'parse, 'source> Parser<'parse, 'source, Output = T> + 'static,
) -> impl for<'parse, 'source> Parser<'parse, 'source, Output = Option<T>> {
    either(pattern, empty()).map(|e| match e {
        Either::Left(left) => Some(left),
        Either::Right(()) => None,
    })
}

// Kleene *
pub fn star<Pattern>(pattern: Pattern) -> RepeatParser<Pattern, EmptyParser> {
    repeat(pattern, empty(), 0, None, false)
}

// Kleene +
pub fn plus<Pattern>(pattern: Pattern) -> RepeatParser<Pattern, EmptyParser> {
    repeat(pattern, empty(), 1, None, false)
}

pub fn sep_by<Pattern, Sep>(pattern: Pattern, sep: Sep) -> RepeatParser<Pattern, Sep> {
    repeat(pattern, sep, 0, None, false)
}

pub fn lines<Pattern>(pattern: Pattern) -> RepeatParser<Pattern, ExactParser> {
    repeat(pattern, exact("\n"), 0, None, true)
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    #[track_caller]
    fn assert_parse<'s, P>(parser: &'s P, s: &'s str)
    where
        P: Parser<'s, 's>,
    {
        if let Err(err) = parser.parse(s) {
            panic!("parse failed: {}", err);
        }
    }

    #[track_caller]
    fn assert_parse_eq<'s, P, E>(parser: &'s P, s: &'s str, expected: E)
    where
        P: Parser<'s, 's>,
        P::Output: PartialEq<E> + Debug,
        E: Debug,
    {
        match parser.parse(s) {
            Err(err) => panic!("parse failed: {}", err),
            Ok(val) => assert_eq!(val, expected),
        }
    }

    #[track_caller]
    fn assert_no_parse<'s, P>(parser: &'s P, s: &'s str)
    where
        P: Parser<'s, 's>,
        P::Output: Debug,
    {
        if let Ok(m) = parser.parse(s) {
            panic!("expected no match, got: {:?}", m);
        }
    }

    #[test]
    fn test_parse() {
        let p = empty();
        assert_parse_eq(&p, "", ());
        assert_no_parse(&p, "x");

        let p = exact("ok");
        assert_parse(&p, "ok");
        assert_no_parse(&p, "");
        assert_no_parse(&p, "o");
        assert_no_parse(&p, "nok");

        let p = sequence(exact("ok"), exact("go"));
        assert_parse(&p, "okgo");
        assert_no_parse(&p, "ok");
        assert_no_parse(&p, "go");
        assert_no_parse(&p, "");

        let p = either(empty(), exact("ok"));
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

        let p = plus(exact("a"));
        assert_no_parse(&p, "");
        assert_parse(&p, "a");
        assert_parse(&p, "aa");
    }
}
