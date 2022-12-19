//! A type that behaves like `Box<dyn Parser<Output=T>>` (even though `Parser`
//! itself has other associated types that would make this impossible).

use crate::{
    parsers::{map::SingleValueParser, single_value},
    ParseIter, Parser, Reported, Result,
};

trait MyDynParserTrait<Out> {
    fn parse_iter<'parse>(
        &'parse self,
        context: &mut crate::ParseContext<'parse>,
        start: usize,
    ) -> Result<DynParseIter<'parse, Out>, Reported>;
}

/// Any parser that produces a value of type Out.
///
/// Unlike most parsers, this is not necessarily `Copy` or `Clone`.
pub struct DynParser<'parser, Out> {
    inner: Box<dyn MyDynParserTrait<Out> + 'parser>,
}

pub struct DynParseIter<'parse, T> {
    inner: Box<dyn ParseIter<'parse, RawOutput = (T,)> + 'parse>,
}

impl<P> MyDynParserTrait<P::Output> for SingleValueParser<P>
where
    P: Parser,
{
    fn parse_iter<'parse>(
        &'parse self,
        context: &mut crate::ParseContext<'parse>,
        start: usize,
    ) -> Result<DynParseIter<'parse, P::Output>, Reported> {
        let iter = <SingleValueParser<P> as Parser>::parse_iter(self, context, start)?;
        Ok(DynParseIter {
            inner: Box::new(iter),
        })
    }
}

impl<'parser, Out> DynParser<'parser, Out> {
    /// Wrap any parser in a `DynParser<'parser, T>`, parameterized only on the
    /// output type (not Iter or RawOutput).
    ///
    /// This isn't exposed as a documented API for a couple of reasons.
    ///
    /// -   I'm not sure we can't do better; this type doesn't pass through `Copy`,
    ///     `Clone`, `Send`, or `Sync`. It also hardcodes `Box`, so patterns like
    ///     `Arc<dyn ...>` won't work. (Of course for the Advent of Code use case
    ///     this doesn't matter too much.)
    ///
    /// -   This is meant as an implemention detail of rule-sets, a `parser!`
    ///     feature. There are several other similar functions, and so far all of
    ///     them are undocumented.
    ///
    #[doc(hidden)]
    #[allow(dead_code)]
    pub(crate) fn new<P>(parser: P) -> Self
    where
        P: Parser<Output = Out> + 'parser,
    {
        DynParser {
            inner: Box::new(single_value(parser)),
        }
    }
}

impl<'parser, Out> Parser for DynParser<'parser, Out> {
    type Output = Out;

    type RawOutput = (Out,);

    type Iter<'parse> = DynParseIter<'parse, Out>
    where
        Self: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut crate::ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        let iter = self.inner.parse_iter(context, start)?;
        Ok(iter)
    }
}

impl<'parse, T> ParseIter<'parse> for DynParseIter<'parse, T> {
    type RawOutput = (T,);

    fn match_end(&self) -> usize {
        self.inner.match_end()
    }

    fn backtrack(&mut self, context: &mut crate::ParseContext<'parse>) -> Result<(), Reported> {
        self.inner.backtrack(context)
    }

    fn convert(&self) -> Self::RawOutput {
        self.inner.convert()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::{i32, map, u64};
    use crate::testing::*;

    #[test]
    fn test_dynamic() {
        let mut p = DynParser::new(u64);
        assert_parse_eq(p, "1141413", 1141413);

        // We assign another parser, with a completely different underlying
        // implementation type, to the same variable, because the wrapper has
        // the same type.
        p = DynParser::new(map(i32, |x| x as u64));
        assert_parse_eq(p, "-1", 0xffff_ffff_ffff_ffff_u64);
    }
}
