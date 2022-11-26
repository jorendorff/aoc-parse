//! Trait for functions in the `parser!` language.
//!
//! This module supports function overloading and should support user-defined
//! functions at some point, maybe.

#![allow(non_camel_case_types)]

use crate::{
    parser::{ExactParser, RepeatParser},
    Parser,
};

pub trait ParserFunction<Args> {
    type Output;

    fn call_parser_function(&self, args: Args) -> Self::Output;
}

// `lines` needs to be something other than a plain Rust function, because the
// goal is to have it support a variable number of arguments.
pub struct lines;

// Just take one argument for now.
impl<'parse, 'source, T: Parser<'parse, 'source>> ParserFunction<(T,)> for lines {
    type Output = RepeatParser<T, ExactParser>;

    fn call_parser_function(&self, (line_parser,): (T,)) -> Self::Output {
        crate::parser::lines(line_parser)
    }
}

// TODO: try implementing the trait for plain `fn` types.