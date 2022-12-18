//! Parser that matches a particular exact string.

use crate::{ParseContext, ParseIter, Parser, Reported, Result};

pub struct ExactParseIter {
    end: usize,
}

impl Parser for str {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = ExactParseIter;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<ExactParseIter, Reported> {
        if context.source()[start..].starts_with(self) {
            Ok(ExactParseIter {
                end: start + self.len(),
            })
        } else {
            Err(context.error_expected(start, &format!("{self:?}")))
        }
    }
}

impl Parser for char {
    type Output = ();
    type RawOutput = ();
    type Iter<'parse> = ExactParseIter;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<ExactParseIter, Reported> {
        if context.source()[start..].starts_with(*self) {
            Ok(ExactParseIter {
                end: start + self.len_utf8(),
            })
        } else {
            Err(context.error_expected(start, &format!("{self:?}")))
        }
    }
}

impl<'parse> ParseIter<'parse> for ExactParseIter {
    type RawOutput = ();
    fn match_end(&self) -> usize {
        self.end
    }
    fn backtrack(&mut self, _context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        Err(Reported)
    }
    fn convert(&self) {}
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

    #[test]
    fn test_exact_char_errors() {
        let p = '\n';
        assert_parse_error(&p, "q", r#"expected '\n' at"#);
        assert_parse_error(&p, "", r#"expected '\n' at end"#);
    }
}
