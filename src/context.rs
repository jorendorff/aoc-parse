//! Mainly error tracking for the overall parse.

use std::{any::Any, collections::HashMap};

use crate::ParseError;

/// Error type for when an error has been reported to ParseContext.
///
/// It's OK to discard this kind of error and return success. See
/// `ParseContext` for an example.
pub struct Reported;

/// Contains the source text we're parsing and tracks errors.
///
/// We track errors in the ParseContext, not Results, because often a parser
/// produces both a successful match *and* the error that will later prove to
/// be the best error message for the overall parse attempt.
///
/// # The `:wq` example
///
/// For example, consider `parser!(line(u32)+)` parsing the following input:
///
/// ```text
/// 1
/// 2
/// 3
/// 4:wq
/// 5
/// ```
///
/// Clearly, someone accidentally typed `:wq` on line 4. But what will happen
/// here is that `line(u32)+` successfully matches the first 3 lines. If we
/// don't track the error we encountered when trying to parse `5:wq` as a
/// `u32`, but content ourselves with the successful match `line(u32)+`
/// produces, the best error message we can ultimately produce is something
/// like "found extra text after a successful match at line 4, column 1".
///
/// Here's how we now handle these success-failures:
///
/// -   `u32` returns success, matching `4`.
///
/// -   `line(u32)` reports an error to the context (at line 4 column 2) and
///      returns `Reported`, because `u32` didn't match the entire line.
///
/// -   `line(u32)+` then *discards* the `Reported` error, backtracks,
///     and returns a successful match for the first 3 lines.
///
/// -   The top-level parser sees that `line(u32)+` didn't match the entire
///     input, and reports an error to the context at line 4 column 1.
///     But we already have a previous error where we had got further,
///     so this error is ignored.
///
/// -   The error at line 4 column 2 is taken from the context and returned to
///     the user.
///
/// # Rationale: Design alternatives
///
/// To implement this without `ParseContext`, we could have implemented a
/// `TigerResult<T, E>` type that can be `Ok(T)`, `Err(E)`, or `OkBut(T, E)`,
/// the last containing *both* a success value *and* an excuse explaining why
/// we did not succeed even more. The forwardmost error would be propagated
/// there instead of being stored in the context. We would use `TigerResult<T,
/// ParseError>` instead of `Result<T, Reported>` everywhere. Both ways have
/// advantages. Both are pretty weird for Rust. The way of the context is
/// something I've wanted to explore in Rust; and it lets us keep using the `?`
/// try operator.
pub struct ParseContext<'parse> {
    source: &'parse str,
    foremost_error: Option<ParseError>,
    rule_sets: HashMap<usize, &'parse [Box<dyn Any>]>,
}

impl<'parse> ParseContext<'parse> {
    /// Create a `ParseContext` to parse the given input.
    pub fn new(source: &'parse str) -> Self {
        ParseContext {
            source,
            foremost_error: None,
            rule_sets: HashMap::new(),
        }
    }

    /// The text being parsed.
    pub fn source(&self) -> &'parse str {
        self.source
    }

    /// Extract the error. Use this only after receiving `Reported` from an
    /// operation on the context.
    ///
    /// # Panics
    ///
    /// If no error has been reported on this context.
    pub fn into_reported_error(self) -> ParseError {
        self.foremost_error
            .expect("a parse error should have been reported")
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
    pub fn report(&mut self, err: ParseError) -> Reported {
        if Some(err.location) > self.foremost_error.as_ref().map(|err| err.location) {
            self.foremost_error = Some(err);
        }
        Reported
    }

    /// Record a `foo expected` error.
    pub fn error_expected(&mut self, start: usize, expected: &str) -> Reported {
        self.report(ParseError::new_expected(self.source(), start, expected))
    }

    /// Record an error when `FromStr::from_str` fails.
    pub fn error_from_str_failed(
        &mut self,
        start: usize,
        end: usize,
        type_name: &'static str,
        message: String,
    ) -> Reported {
        self.report(ParseError::new_from_str_failed(
            self.source(),
            start,
            end,
            type_name,
            message,
        ))
    }

    /// Record an "extra unparsed text after match" error.
    pub fn error_extra(&mut self, location: usize) -> Reported {
        self.report(ParseError::new_extra(self.source(), location))
    }

    pub(crate) fn register_rule_set(
        &mut self,
        rule_set_id: usize,
        rule_parsers: &'parse [Box<dyn Any>],
    ) {
        self.rule_sets.insert(rule_set_id, rule_parsers);
    }

    pub(crate) fn fetch_parser_for_rule(
        &self,
        rule_set_id: usize,
        index: usize,
    ) -> &'parse dyn Any {
        let rule_parsers: &'parse [Box<dyn Any>] = self
            .rule_sets
            .get(&rule_set_id)
            .expect("internal error: rule set not registered");
        &*rule_parsers[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::{lines, sections, u64};
    use crate::testing::*;

    #[test]
    fn test_repeat_region_errors() {
        let p = lines(u64);

        let example1 = "\
            194832\n\
            2094235\n\
            374274\n\
            4297534:wq\n\
            59842843\n\
        ";

        assert_parse_error(
            &p,
            example1,
            "matched part of the line, but not all of it at line 4 column 8",
        );

        let p = sections(lines(u64));

        let example2 = "\
            1\n\
            \n\
            3\n\
            4\n\
            5\n\
            6:wq\n\
            \n\
            8\n\
            9\n\
        ";

        assert_parse_error(
            &p,
            example2,
            "matched part of the line, but not all of it at line 6 column 2",
        );
    }
}
