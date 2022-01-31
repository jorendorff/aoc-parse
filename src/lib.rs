//! A parser library designed for Advent of Code.

mod error;
mod matches;
mod parser;

pub use error::{ParseError, Result};
pub use parser::{alt, empty, exact, plus, repeat, sep_by, seq, star, Parser};
