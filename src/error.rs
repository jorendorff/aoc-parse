use thiserror::Error;

#[derive(Clone, Debug)]
pub struct ParseError {
    pub source: String,
    pub location: usize,
    reason: ParseErrorReason,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TODO print the line of `source` containing `location`\n{}",
            self.reason
        )
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Debug, Error)]
enum ParseErrorReason {
    #[error("extra unparsed text after match")]
    Extra,
    #[error("expected {0:?}")]
    Expected(String),
}

impl ParseError {
    pub fn new_extra(source: &str, location: usize) -> Self {
        ParseError {
            source: source.to_string(),
            location,
            reason: ParseErrorReason::Extra,
        }
    }

    pub fn new_expected(source: &str, location: usize, expected: &str) -> Self {
        ParseError {
            source: source.to_string(),
            location,
            reason: ParseErrorReason::Expected(expected.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;
