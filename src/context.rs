//! Mainly error tracking for the overall parse.

use crate::ParseError;

/// Contains the source text we're parsing and tracks errors.
///
/// We track errors in the ParseContext, not Results, because often a parser
/// produces both a successful match *and* the error that will later prove to
/// be the best error message for the overall parse attempt.
///
/// For example, consider `parser!(line(u32)+)` parsing the input
/// `1\n2\n3\n4\n5:wq\n6\n7\n`. The actual problem is that someone accidentally
/// typed `:wq` on line 5. But what will happen here is that `line(u32)+`
/// successfully parses the first 4 lines. If we don't track the error we
/// encountered when trying to parse `5:wq` as a `u32`, but content ourselves
/// with the successful match `line(u32)+` produces, the best error message we
/// can ultimately produce is something like "found extra text after a
/// successful match at line 5, column 1".
///
pub struct ParseContext<'parse> {
    source: &'parse str,
    foremost_error: Option<ParseError>,
}

impl<'parse> ParseContext<'parse> {
    /// Create a `ParseContext` to parse the given input.
    pub fn new(source: &'parse str) -> Self {
        ParseContext {
            source,
            foremost_error: None,
        }
    }

    /// The text being parsed.
    pub fn source(&self) -> &'parse str {
        self.source
    }

    /// Record an error.
    ///
    /// Currently a ParseContext only tracks the foremost error. That is, if
    /// `err.location` is farther forward than any other error we've
    /// encountered, we store it. Otherwise discard it.
    ///
    /// Nontrivial patterns try several different things. If anything succeeds,
    /// we get a match. We only fail if every branch leads to failure. This
    /// means that by the time matching fails, we have an abundance of
    /// different error messages. Generally the error we want is the one where
    /// we progressed as far as possible through the input string before
    /// failing.
    pub fn report(&mut self, err: ParseError) -> ParseError {
        if Some(err.location) > self.foremost_error.as_ref().map(|err| err.location) {
            self.foremost_error = Some(err.clone());
        }
        err
    }

    /// Record a `foo expected` error.
    pub fn error_expected(&mut self, start: usize, expected: &str) -> ParseError {
        self.report(ParseError::new_expected(self.source(), start, expected))
    }

    /// Record an error when `FromStr::from_str` fails.
    pub fn error_from_str_failed(
        &mut self,
        start: usize,
        end: usize,
        type_name: &'static str,
        message: String,
    ) -> ParseError {
        self.report(ParseError::new_from_str_failed(
            self.source(),
            start,
            end,
            type_name,
            message,
        ))
    }

    /// Record an "extra unparsed text after match" error.
    pub fn error_extra(&mut self, location: usize) -> ParseError {
        self.report(ParseError::new_extra(self.source(), location))
    }
}
