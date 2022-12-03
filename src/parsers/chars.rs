use crate::{ParseError, ParseIter, Parser, Result};

#[allow(non_camel_case_types)]
pub struct CharParser {
    noun: &'static str,
    predicate: fn(char) -> bool,
}

pub enum CharParseIter<'parse, 'source> {
    Before {
        parser: &'parse CharParser,
        source: &'source str,
        start: usize,
    },
    Success(char),
    Error,
}

impl<'parse, 'source> Parser<'parse, 'source> for CharParser {
    type Output = char;
    type RawOutput = (char,);
    type Iter = CharParseIter<'parse, 'source>;

    fn parse_iter(&'parse self, source: &'source str, start: usize) -> Self::Iter {
        CharParseIter::Before {
            parser: self,
            source,
            start,
        }
    }
}

impl<'parse, 'source> ParseIter for CharParseIter<'parse, 'source> {
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
