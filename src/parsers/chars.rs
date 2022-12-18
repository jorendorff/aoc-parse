use crate::parsers::MapParser;
use crate::{ParseContext, ParseIter, Parser, Reported, Result};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct CharParser {
    noun: &'static str,
    predicate: fn(char) -> bool,
}

pub struct CharParseIter {
    c: char,
    end: usize,
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
