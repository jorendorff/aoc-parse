//! A parser library designed for Advent of Code.

mod error;
pub mod functions;
pub mod macros;
mod parser;
mod types;

pub use error::{ParseError, Result};
pub use parser::{empty, exact, plus, repeat, star, ParseIter, Parser};

pub mod prelude {
    pub use crate::functions::lines;
    pub use crate::parser::{
        bool, i128, i128_bin, i128_hex, i16, i16_bin, i16_hex, i32, i32_bin, i32_hex, i64, i64_bin,
        i64_hex, i8, i8_bin, i8_hex, isize, isize_bin, isize_hex, sep_by, u128, u128_bin, u128_hex,
        u16, u16_bin, u16_hex, u32, u32_bin, u32_hex, u64, u64_bin, u64_hex, u8, u8_bin, u8_hex,
        usize, usize_bin, usize_hex, Parser,
    };
}
