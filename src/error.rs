use thiserror::Error;

#[derive(Clone)]
pub struct ParseError {
    pub source: String,
    pub location: usize,
    reason: ParseErrorReason,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = &self.reason;
        let source = &self.source;
        let p = self.location.min(source.len());
        let line_start = match source[..p].rfind('\n') {
            Some(i) => i + 1,
            None => 0,
        };
        let line_num = source[..line_start].chars().filter(|c| *c == '\n').count() + 1;
        let column_num = p - line_start + 1;
        write!(f, "{reason} at line {line_num} column {column_num}")
    }
}

// aoc_runner prints the Debug form of errors, instead of the human-readable
// Display form. Work around this by adding the Display form to ParseError's
// Debug output.
impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParseError")
            .field("source", &self.source)
            .field("location", &self.location)
            .field("reason", &self.reason)
            .field("summary", &format!("{self}"))
            .finish()
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Debug, Error)]
enum ParseErrorReason {
    #[error("extra unparsed text after match")]
    Extra,
    #[error("line() can't match here because this is not at the start of a line")]
    NotAtLineStart,
    #[error("line(pattern) matched part of the line, but not all of it")]
    LineExtra,
    #[error("expected {0:?}")]
    Expected(String),
    #[error("failed to parse {input:?} as type {type_name}: {message}")]
    FromStrFailed {
        input: String,
        type_name: &'static str,
        message: String,
    },
    #[error("nothing matches this pattern")]
    CannotMatch,
}

impl ParseError {
    pub fn new_extra(source: &str, location: usize) -> Self {
        ParseError {
            source: source.to_string(),
            location,
            reason: ParseErrorReason::Extra,
        }
    }

    pub fn new_bad_line_start(source: &str, location: usize) -> Self {
        ParseError {
            source: source.to_string(),
            location,
            reason: ParseErrorReason::NotAtLineStart,
        }
    }

    pub fn new_line_extra(source: &str, location: usize) -> Self {
        ParseError {
            source: source.to_string(),
            location,
            reason: ParseErrorReason::LineExtra,
        }
    }

    pub fn new_expected(source: &str, location: usize, expected: &str) -> Self {
        ParseError {
            source: source.to_string(),
            location,
            reason: ParseErrorReason::Expected(expected.to_string()),
        }
    }

    pub fn new_from_str_failed(
        source: &str,
        start: usize,
        end: usize,
        type_name: &'static str,
        message: String,
    ) -> Self {
        ParseError {
            source: source.to_string(),
            location: start,
            reason: ParseErrorReason::FromStrFailed {
                input: source[start..end].to_string(),
                type_name,
                message,
            },
        }
    }

    pub fn new_cannot_match(source: &str, location: usize) -> Self {
        ParseError {
            source: source.to_string(),
            location,
            reason: ParseErrorReason::CannotMatch,
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, ParseError>;
