//! The parser used by the `string()` function.

use crate::{ParseContext, ParseIter, Parser, Reported, Result};

#[derive(Clone, Copy)]
pub struct StringParser<P> {
    pub(crate) parser: P,
}

pub struct StringParseIter<'parse, P>
where
    P: Parser + 'parse,
{
    source: &'parse str,
    start: usize,
    iter: P::Iter<'parse>,
}

impl<P> Parser for StringParser<P>
where
    P: Parser,
{
    type Output = String;
    type RawOutput = (String,);
    type Iter<'parse> = StringParseIter<'parse, P>
    where
        P: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        let iter = self.parser.parse_iter(context, start)?;
        Ok(StringParseIter {
            source: context.source(),
            start,
            iter,
        })
    }
}

impl<'parse, P> ParseIter<'parse> for StringParseIter<'parse, P>
where
    P: Parser,
{
    type RawOutput = (String,);

    fn match_end(&self) -> usize {
        self.iter.match_end()
    }

    fn backtrack(&mut self, context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        self.iter.backtrack(context)
    }

    fn convert(&self) -> (String,) {
        let end = self.iter.match_end();
        let value = self.source[self.start..end].to_string();
        (value,)
    }
}

/// Parse using `parser`, but instead of converting the matched text to a
/// Rust value, simply return it as a `String`.
///
/// By default, `parser!(alpha+)` returns a `Vec<char>`, and sometimes that
/// is handy in AoC, but often it's better to have it return a `String`.
/// That can be done with `parser!(string(alpha+))`.
pub fn string<P: Parser>(parser: P) -> StringParser<P> {
    crate::parsers::StringParser { parser }
}
