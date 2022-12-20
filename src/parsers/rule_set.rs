use std::{any::Any, marker::PhantomData, pin::Pin};

use crate::{
    parsers::dynamic::{DynParseIter, DynParser},
    ParseContext, Parser, Reported,
};

/// Builder for constructing a parser based on a rule set.
///
/// Builder methods must be called in a strict order:
///
/// -   First `new()` to create the builder.
/// -   Then `.new_rule()` 0 or more times to create the rules.
/// -   Then `.assign_parser_for_rule()` the same number of times, in the same order,
///     to assign the actual parsers implementing each rule.
/// -   Lastly `.build()` to build the entry point to the rule set.
pub struct RuleSetBuilder {
    id: Pin<Box<u8>>,
    capacity: usize,
    rule_parsers: Vec<Box<dyn Any>>,
}

/// Parser for a rule in a rule set.
///
/// This is used by the `parser!` macro to implement `rule`.
#[doc(hidden)]
#[derive(Debug)]
pub struct RuleParser<T> {
    rule_set_id: usize,
    index: usize,
    phantom: PhantomData<fn() -> T>,
}

pub struct RuleSetParser<T> {
    id: Pin<Box<u8>>,
    rule_parsers: Vec<Box<dyn Any>>,
    entry_parser: DynParser<'static, T>,
}

impl RuleSetBuilder {
    /// Create a builder to build a new parser based on a rule set.
    ///
    /// This is used by the `parser!` macro to implement `rule`.
    #[doc(hidden)]
    pub fn new() -> Self {
        RuleSetBuilder {
            id: Box::pin(0),
            capacity: 0,
            rule_parsers: vec![],
        }
    }

    /// Create a `Copy` parser as a placeholder for a rule in a rule set.
    ///
    /// This is used by the `parser!` macro to implement `rule`.
    #[doc(hidden)]
    pub fn new_rule<T>(&mut self) -> RuleParser<T> {
        let index = self.capacity;
        self.capacity += 1;
        RuleParser {
            rule_set_id: self.id(),
            index,
            phantom: PhantomData,
        }
    }

    /// Set the parser to be used when the given parser `nt` is invoked.
    ///
    /// This is used by the `parser!` macro to implement `rule`.
    #[doc(hidden)]
    pub fn assign_parser_for_rule<T, P>(&mut self, nt: &RuleParser<T>, parser: P)
    where
        T: 'static,
        P: Parser<Output = T> + 'static,
    {
        assert_eq!(nt.rule_set_id, self.id());
        assert_eq!(nt.index, self.rule_parsers.len());

        // We are double-boxing the parsers at the moment. Look away
        self.rule_parsers.push(Box::new(DynParser::new(parser)));
    }

    /// Build the rule-set-based parser.
    ///
    /// This is used by the `parser!` macro to implement `rule`.
    #[doc(hidden)]
    pub fn build<P>(self, parser: P) -> RuleSetParser<P::Output>
    where
        P: Parser + 'static,
    {
        RuleSetParser {
            id: self.id,
            rule_parsers: self.rule_parsers,
            entry_parser: DynParser::new(parser),
        }
    }

    fn id(&self) -> usize {
        &self.id as &u8 as *const u8 as usize
    }
}

// Explicit impl because `#[derive(Clone)]` fails to do the right thing in this
// case.
impl<T> Clone for RuleParser<T> {
    fn clone(&self) -> Self {
        RuleParser {
            rule_set_id: self.rule_set_id,
            index: self.index,
            phantom: self.phantom,
        }
    }
}

impl<T> Copy for RuleParser<T> {}

impl<T> Parser for RuleParser<T>
where
    T: 'static,
{
    type Output = T;
    type RawOutput = (T,);
    type Iter<'parse> = DynParseIter<'parse, T> where T: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        let parser_as_any: &'parse dyn Any =
            context.fetch_parser_for_rule(self.rule_set_id, self.index);
        let parser = parser_as_any
            .downcast_ref::<DynParser<T>>()
            .expect("internal error: downcast failed");
        parser.parse_iter(context, start)
    }
}

impl<T> RuleSetParser<T> {
    fn id(&self) -> usize {
        &self.id as &u8 as *const u8 as usize
    }
}

impl<T> Parser for RuleSetParser<T> {
    type Output = T;
    type RawOutput = (T,);
    type Iter<'parse> = DynParseIter<'parse, T> where T: 'parse;

    fn parse_iter<'parse>(
        &'parse self,
        context: &mut ParseContext<'parse>,
        start: usize,
    ) -> Result<Self::Iter<'parse>, Reported> {
        context.register_rule_set(self.id(), &self.rule_parsers);
        self.entry_parser.parse_iter(context, start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::{alt, map, pair, repeat_sep, u32};
    use crate::testing::*;

    #[test]
    fn test_rule_set() {
        #[derive(Debug, PartialEq)]
        enum Value {
            Int(u32),
            List(Vec<Value>),
        }

        let value_parser = {
            let mut builder = RuleSetBuilder::new();
            let value: RuleParser<Value> = builder.new_rule();
            let values: RuleParser<Vec<Value>> = builder.new_rule();

            builder.assign_parser_for_rule(
                &value,
                alt(
                    map(u32, Value::Int),
                    alt(
                        map("[]", |()| Value::List(vec![])),
                        map(pair("[", pair(values, "]")), |(_, (vs, _))| Value::List(vs)),
                    ),
                ),
            );
            builder.assign_parser_for_rule(&values, repeat_sep(value, ","));
            builder.build(value)
        };

        assert_parse_eq(&value_parser, "92183", Value::Int(92183));
        assert_parse_eq(
            &value_parser,
            "[3,[7,88]]",
            Value::List(vec![
                Value::Int(3),
                Value::List(vec![Value::Int(7), Value::Int(88)]),
            ]),
        );
    }
}
