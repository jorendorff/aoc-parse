//! A parser library designed for Advent of Code.

mod error;
pub mod macros;
mod parser;

pub use error::{ParseError, Result};
pub use parser::{empty, exact, plus, repeat, star, Parser};

pub mod prelude {
    pub use crate::parser::{lines, sep_by, Parser};
}
