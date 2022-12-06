use crate::parsers::MapParser;
use crate::{ParseError, ParseIter, Parser, Result};

#[allow(non_camel_case_types)]
pub struct CharParser {
    noun: &'static str,
    predicate: fn(char) -> bool,
}

pub enum CharParseIter<'parse> {
    Before {
        parser: &'parse CharParser,
        source: &'parse str,
        start: usize,
    },
    Success(char),
    Error,
}

impl<'parse> Parser<'parse> for CharParser {
    type Output = char;
    type RawOutput = (char,);
    type Iter = CharParseIter<'parse>;

    fn parse_iter<'source>(&'parse self, source: &'source str, start: usize) -> Self::Iter
    where
        'source: 'parse,
    {
        CharParseIter::Before {
            parser: self,
            source,
            start,
        }
    }
}

impl<'parse> ParseIter for CharParseIter<'parse> {
    type RawOutput = (char,);
    fn next_parse(&mut self) -> Option<Result<usize>> {
        if let CharParseIter::Before {
            parser,
            source,
            start,
        } = *self
        {
            match source[start..].chars().next() {
                Some(c) if (parser.predicate)(c) => {
                    *self = CharParseIter::Success(c);
                    Some(Ok(start + c.len_utf8()))
                }
                _ => {
                    *self = CharParseIter::Error;
                    Some(Err(ParseError::new_expected(source, start, parser.noun)))
                }
            }
        } else {
            None
        }
    }

    fn take_data(&mut self) -> (char,) {
        match self {
            CharParseIter::Success(c) => (*c,),
            _ => panic!("invalid state"),
        }
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
    parser: CharParser {
        noun: "decimal digit",
        predicate: |c| c.is_ascii_digit(),
    },
    mapper: |c| c.to_digit(10).unwrap() as usize,
};

/// Matches a binary digit `'0'` or `'1'`, and converts it to its integer value
/// `0` or `1`.
#[allow(non_upper_case_globals)]
pub const digit_bin: MapParser<CharParser, fn(char) -> usize> = MapParser {
    parser: CharParser {
        noun: "hexadecimal digit",
        predicate: |c| c.is_digit(2),
    },
    mapper: |c| c.to_digit(2).unwrap() as usize,
};

/// Matches a hexadecimal digit `'0'`-`'9'`, `'a'`-`'f'`, or `'A'`-`'F'`, and
/// converts it to its integer value `0`-`15`.
#[allow(non_upper_case_globals)]
pub const digit_hex: MapParser<CharParser, fn(char) -> usize> = MapParser {
    parser: CharParser {
        noun: "hexadecimal digit",
        predicate: |c| c.is_digit(16),
    },
    mapper: |c| c.to_digit(16).unwrap() as usize,
};
