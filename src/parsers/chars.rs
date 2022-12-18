use crate::{
    parsers::{BasicParseIter, MapParser},
    ParseContext, ParseIter, Parser, Reported, Result,
};

#[derive(Clone, Copy)]
pub struct CharParser {
    noun: &'static str,
    predicate: fn(char) -> bool,
}

pub struct CharParseIter {
    c: char,
    end: usize,
}

/// The type of parser returned by [`char_of()`].
#[derive(Clone, Copy)]
pub struct CharOfParser {
    options: &'static str,
}

impl Parser for CharParser {
    type Output = char;
    type RawOutput = (char,);
    type Iter<'parse> = CharParseIter;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        match context.source()[start..].chars().next() {
            Some(c) if (self.predicate)(c) => Ok(CharParseIter {
                c,
                end: start + c.len_utf8(),
            }),
            _ => Err(context.error_expected(start, self.noun)),
        }
    }
}

impl<'parse> ParseIter<'parse> for CharParseIter {
    type RawOutput = (char,);
    fn match_end(&self) -> usize {
        self.end
    }
    fn backtrack(&mut self, _context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        Err(Reported)
    }
    fn convert(&self) -> (char,) {
        (self.c,)
    }
}

impl Parser for CharOfParser {
    type Output = usize;
    type RawOutput = (usize,);
    type Iter<'parse> = BasicParseIter<usize>;
    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        if let Some(c) = context.source()[start..].chars().next() {
            // Note: `self.options.find(c)` would give the wrong answer here: we
            // want the number of characters before `c`, not bytes.
            for (i, x) in self.options.chars().enumerate() {
                if c == x {
                    return Ok(BasicParseIter {
                        value: i,
                        end: start + c.len_utf8(),
                    });
                }
            }
        }
        Err(context.error_expected(start, &format!("one of {:?}", self.options)))
    }
}

/// Matches any alphabetic character (see [`char::is_alphabetic`]). Returns a `char`.
#[allow(non_upper_case_globals)]
pub const alpha: CharParser = CharParser {
    noun: "letter",
    predicate: char::is_alphabetic,
};

/// Matches any alphabetic or numeric character (see
/// [`char::is_alphanumeric`]). Returns a `char`.
#[allow(non_upper_case_globals)]
pub const alnum: CharParser = CharParser {
    noun: "letter or digit",
    predicate: char::is_alphanumeric,
};

/// Matches any uppercase letter (see [`char::is_uppercase`]). Returns a `char`.
#[allow(non_upper_case_globals)]
pub const upper: CharParser = CharParser {
    noun: "uppercase letter",
    predicate: char::is_uppercase,
};

/// Matches any lowercase letter (see [`char::is_lowercase`]). Returns a `char`.
#[allow(non_upper_case_globals)]
pub const lower: CharParser = CharParser {
    noun: "lowercase letter",
    predicate: char::is_lowercase,
};

/// Matches any Unicode character. Returns a `char`.
#[allow(non_upper_case_globals)]
pub const any_char: CharParser = CharParser {
    noun: "any character",
    predicate: |_| true,
};

/// Matches any ASCII decimal digit `'0'`-`'9'` and converts it to its integer
/// value `0`-`9`.
#[allow(non_upper_case_globals)]
pub const digit: MapParser<CharParser, fn(char) -> usize> = MapParser {
    inner: CharParser {
        noun: "decimal digit",
        predicate: |c| c.is_ascii_digit(),
    },
    mapper: |c| c.to_digit(10).unwrap() as usize,
};

/// Matches a binary digit `'0'` or `'1'`, and converts it to its integer value
/// `0` or `1`.
#[allow(non_upper_case_globals)]
pub const digit_bin: MapParser<CharParser, fn(char) -> usize> = MapParser {
    inner: CharParser {
        noun: "hexadecimal digit",
        predicate: |c| c.is_digit(2),
    },
    mapper: |c| c.to_digit(2).unwrap() as usize,
};

/// Matches a hexadecimal digit `'0'`-`'9'`, `'a'`-`'f'`, or `'A'`-`'F'`, and
/// converts it to its integer value `0`-`15`.
#[allow(non_upper_case_globals, clippy::is_digit_ascii_radix)]
pub const digit_hex: MapParser<CharParser, fn(char) -> usize> = MapParser {
    inner: CharParser {
        noun: "hexadecimal digit",
        predicate: |c| c.is_digit(16),
    },
    mapper: |c| c.to_digit(16).unwrap() as usize,
};

/// Make a parser that matches any single character in `options` and produces
/// the index of that character in the list, so that `char_of("ABCD")` produces
/// a number in `0..4`.
pub fn char_of(options: &'static str) -> CharOfParser {
    CharOfParser { options }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[test]
    fn test_char_of() {
        assert_parse_eq(char_of("<=>"), "<", 0);
        assert_parse_eq(char_of("<=>"), ">", 2);
        assert_parse_error(char_of("<=>"), "", "expected one of \"<=>\"");
        assert_parse_error(
            char_of("<=>"),
            " <",
            "expected one of \"<=>\" at line 1 column 1",
        );
        assert_parse_eq(char_of("DCBA"), "D", 0);
        assert_parse_eq(char_of("DCBA"), "C", 1);
        assert_parse_error(char_of("DCBA"), "DC", "at line 1 column 2");

        // Nonsense parser, nonsense error message; but check that the behavior
        // is correct.
        assert_parse_error(char_of(""), "", "expected one of \"\"");

        // Operates on characters, not UTF-8 bytes. (So this returns 2, not 8;
        // but also, not 0 even though the first byte of the text happens to
        // match the first byte of `options`.)
        assert_parse_eq(char_of("😂😃🌍"), "🌍", 2);

        assert_parse_error(char_of("😂😃🌍"), "L", "expected one of \"😂😃🌍\"");
    }
}
