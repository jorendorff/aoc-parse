//! Structured data, the output of successful parsing.

use num_bigint::BigUint;

/// The output of a successful parse.
#[derive(Debug)]
pub enum Match {
    Exact(String),

    UInt(BigUint),

    /// Match for a repeat expression (`foo*`).
    Array(Vec<Match>),

    /// Match for multiple expressions in sequence (`foo bar baz`).
    Tuple(Vec<Match>),

    Label(String, Box<Match>),

    /// Match for a `=>` expression.
    ///
    /// Represents a promise to map this data through a Rust closure on
    /// success. We don't run this code until a complete successful parse of
    /// the whole string; that way parse failure and backtracking is never
    /// observable.
    Map(Box<Match>, usize),

    /// Match for an `as` expression.
    ///
    /// Represents a promise to convert the data to a particular Rust type. The
    /// actual conversion happens after parsing is finished. For example, if
    /// the pattern is `(digit+ as u8) (garbage: regex(".*"))`, and the target
    /// string is `1234`, parsing succeeds and the cast then fails because
    /// `1234` is too big to fit in a `u8`. We do not then go back and
    /// successfully parse the u8 `123` followed by a garbage `4`.
    Cast(Box<Match>, usize),
}
