//! A parser library designed for Advent of Code.

mod error;
pub mod functions;
pub mod macros;
mod parser;

pub use error::{ParseError, Result};
pub use parser::{empty, exact, plus, repeat, star, ParseIter, Parser};

pub mod prelude {
    pub use crate::functions::lines;
    pub use crate::parser::{sep_by, Parser};
}
