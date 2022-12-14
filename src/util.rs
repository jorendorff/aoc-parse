//! Utility function for compatibility with aoc-runner.

use crate::{ParseError, Parser};

/// Parse the given puzzle input supplied by `#[aoc_generator]`.
///
/// This function is like `parser.parse(puzzle_input)` except that
/// `#[aoc_generator]` unfortunately [strips off trailing newlines][bad]. This
/// function therefore checks to see if the last line is missing its final `\n`
/// and, if so, re-adds it before parsing.
///
/// # Example
///
/// ```no_run
/// use aoc_runner_derive::*;
/// use aoc_parse::{parser, prelude::*};
///
/// #[aoc_generator(day1)]
/// fn parse_input(text: &str) -> anyhow::Result<Vec<Vec<u64>>> {
///     let p = parser!(repeat_sep(lines(u64), "\n"));
///     aoc_parse(text, p)
/// }
/// ```
///
/// [bad]: https://github.com/gobanos/aoc-runner/blob/master/src/lib.rs#L17
#[doc(hidden)]
pub fn aoc_parse<P, E>(puzzle_input: &str, parser: P) -> Result<P::Output, E>
where
    P: Parser,
    E: From<ParseError>,
{
    let mut p = puzzle_input.to_string();
    if !p.ends_with('\n') {
        p.push('\n');
    }
    Ok(parser.parse(&p)?)
}
