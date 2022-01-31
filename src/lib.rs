//! A parser library designed for Advent of Code.

mod error;
pub mod macros;
mod matches;
mod parser;

pub use error::{ParseError, Result};
pub use parser::{empty, exact, one_of, plus, repeat, sequence, star, Parser};
