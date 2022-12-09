//! Parser that matches a particular exact string.

use crate::{error::Result, ParseError, ParseIter, Parser};

pub struct ExactParseIter {
    end: usize,
}

impl Parser for str {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = ExactParseIter;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<ExactParseIter> {
        if source[start..].starts_with(self) {
            Ok(ExactParseIter {
                end: start + self.len(),
            })
        } else {
            Err(ParseError::new_expected(source, start, self))
        }
    }
}

impl Parser for char {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = ExactParseIter;

    fn parse_iter<'parse>(
        &'parse self,
        source: &'parse str,
        start: usize,
    ) -> Result<ExactParseIter> {
        if source[start..].starts_with(*self) {
            Ok(ExactParseIter {
                end: start + self.len_utf8(),
            })
        } else {
            Err(ParseError::new_expected(source, start, &self.to_string()))
        }
    }
}

impl ParseIter for ExactParseIter {
    type RawOutput = ();
    fn match_end(&self) -> usize {
        self.end
    }
    fn backtrack(&mut self) -> bool {
        false
    }
    fn into_raw_output(self) {}
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[test]
    fn test_string_lifetime() {
        // A string that is locally scoped can serve as a constant for a
        // parser. No reason why not. The parser lifetime is limited to the
        // lifetime of the string.
        let x = format!("{} {}!", "hello", "world");
        let p: &str = &x;
        assert_parse_eq(&p, "hello world!", ());
    }
}
