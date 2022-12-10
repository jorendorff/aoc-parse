use thiserror::Error;

#[derive(Clone, Debug, Error)]
enum ParseErrorReason {
    #[error("extra unparsed text after match")]
    Extra,
    #[error("line() can't match here because this is not at the start of a line")]
    NotAtLineStart,
    #[error("section() can't match here because this is not at the start of a section")]
    NotAtSectionStart,
    #[error("line(pattern) matched part of the line, but not all of it")]
    LineExtra,
    #[error("section(pattern) matched part of the section, but not all of it")]
    SectionExtra,
    #[error("expected {0:?}")]
    Expected(String),
    #[error("failed to parse {input:?} as type {type_name}: {message}")]
    FromStrFailed {
        input: String,
        type_name: &'static str,
        message: String,
    },
}

/// An error happened while trying to parse puzzle input or convert the matched
/// characters to a Rust value.
#[derive(Clone)]
pub struct ParseError {
    /// The puzzle input we were trying to parse.
    pub source: String,

    /// The byte offset into `source` where the elves detected a problem and
    /// could not go any further. This is guaranteed to be a char boundary in
    /// `source`.
    pub location: usize,

    reason: ParseErrorReason,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = &self.reason;
        let source = &self.source;
        if self.location == source.len() {
            write!(f, "{reason} at end of input")
        } else {
            let p = self.location.min(source.len());
            let line_start = match source[..p].rfind('\n') {
                Some(i) => i + 1,
                None => 0,
            };
            let line_num = source[..line_start].chars().filter(|c| *c == '\n').count() + 1;
            let column_num = source[line_start..p].chars().count() + 1;
            write!(f, "{reason} at line {line_num} column {column_num}")
        }
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

impl ParseError {
    fn new(source: &str, location: usize, reason: ParseErrorReason) -> Self {
        assert!(source.is_char_boundary(location));
        ParseError {
            source: source.to_string(),
            location,
            reason,
        }
    }

    pub(crate) fn new_extra(source: &str, location: usize) -> Self {
        Self::new(source, location, ParseErrorReason::Extra)
    }

    pub(crate) fn new_bad_line_start(source: &str, location: usize) -> Self {
        Self::new(source, location, ParseErrorReason::NotAtLineStart)
    }

    pub(crate) fn new_bad_section_start(source: &str, location: usize) -> Self {
        Self::new(source, location, ParseErrorReason::NotAtSectionStart)
    }

    pub(crate) fn new_line_extra(source: &str, location: usize) -> Self {
        Self::new(source, location, ParseErrorReason::LineExtra)
    }

    pub(crate) fn new_section_extra(source: &str, location: usize) -> Self {
        Self::new(source, location, ParseErrorReason::SectionExtra)
    }

    pub(crate) fn new_expected(source: &str, location: usize, expected: &str) -> Self {
        Self::new(
            source,
            location,
            ParseErrorReason::Expected(expected.to_string()),
        )
    }

    pub(crate) fn new_from_str_failed(
        source: &str,
        start: usize,
        end: usize,
        type_name: &'static str,
        message: String,
    ) -> Self {
        Self::new(
            source,
            start,
            ParseErrorReason::FromStrFailed {
                input: source[start..end].to_string(),
                type_name,
                message,
            },
        )
    }

    /// This is used when a subparser is used on a slice of the original
    /// string. If the subparse fails, the error location is a position within
    /// the slice. This can be used, passing the start offset of the slice, to
    /// convert that to a position within the original string.
    pub(crate) fn adjust_location(mut self, full_source: &str, offset: usize) -> Self {
        self.source = full_source.to_string();
        self.location += offset;
        self
    }
}

pub(crate) type Result<T, E = ParseError> = std::result::Result<T, E>;
