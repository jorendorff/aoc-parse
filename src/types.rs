use crate::{
    parser::{self, EitherParser, EmptyParser, NeverParser, SequenceParser},
    Parser,
};

// --- TupleConcat

pub trait TupleConcat<Tail> {
    type Output: ParserOutput;

    fn concat(self, tail: Tail) -> Self::Output;
}

macro_rules! impl_tuple_concat_helper {
    (( $($left:ident ,)* ) ,) => {
        impl< $($left ,)* > TupleConcat<()> for ( $($left ,)* ) {
            type Output = Self;

            fn concat(self, _tail: ()) -> Self {
                self
            }
        }
    };
    (( $($left:ident ,)* ) , $first:ident $($rest:ident)*) => {
        #[allow(non_snake_case)]
        impl<$($left ,)* $first, $($rest ,)*> TupleConcat<($first, $($rest ,)*)> for ( $($left ,)* ) {
            type Output = ($($left ,)* $first, $($rest ,)*);
            fn concat(self, ($first, $($rest ,)*): ($first, $($rest ,)*)) -> Self::Output {
                let ( $($left ,)* ) = self;
                ($($left ,)* $first, $($rest ,)*)
            }
        }
        impl_tuple_concat_helper!( ( $($left ,)* ) , $($rest)* );
    };
}

macro_rules! impl_tuple_concat {
    ( , $($right:ident)* ) => {
        impl_tuple_concat_helper!((), $($right)*);
    };
    ($first:ident $($rest:ident)* , $($right:ident)*) => {
        impl_tuple_concat_helper!( ( $first, $($rest ,)* ) , $($right)* );
        impl_tuple_concat!( $($rest)* , $($right)*);
    };
}

impl_tuple_concat!(A0 A1 A2 A3, B0 B1 B2 B3);

// --- ParserOutput

// there could be a special non-tuple ParserOutput type for character output such that Vec<char> ends up as String instead;
// or, parsers could have a toggle that says whether they produce "data" or the exact characters parsed
//
// another special ParserOutput type might be Never

pub trait ParserOutput {
    type UserType;
    type OptionalType; // trying to make Option<()> be bool instead

    fn into_user_type(self) -> Self::UserType;
}

impl ParserOutput for () {
    type UserType = Self;
    type OptionalType = bool;

    fn into_user_type(self) {}
}

impl<A> ParserOutput for (A,) {
    type UserType = A;
    type OptionalType = Option<A>;

    fn into_user_type(self) -> A {
        self.0
    }
}

macro_rules! impl_parser_output {
    ( $( $t:ident )* ) => {
        impl < $($t,)* > ParserOutput for ( $($t,)* ) {
            type UserType = Self;
            type OptionalType = Option<Self>;

            fn into_user_type(self) -> Self {
                self
            }
        }
    }
}

impl_parser_output!(A B);
impl_parser_output!(A B C);
impl_parser_output!(A B C D);
impl_parser_output!(A B C D E);
impl_parser_output!(A B C D E F);
impl_parser_output!(A B C D E F G);
impl_parser_output!(A B C D E F G H);

#[derive(Debug)]
pub enum Never {}

impl ParserOutput for Never {
    type UserType = Self;
    type OptionalType = Option<Self>;

    fn into_user_type(self) -> Self {
        self
    }
}

// --- ParserTuple

trait ParserTuple<'parse, 'source> {
    type SeqParser;
    type AltParser;

    fn seq(self) -> Self::SeqParser;
    fn alt(self) -> Self::AltParser;
}

impl<'parse, 'source> ParserTuple<'parse, 'source> for () {
    type SeqParser = EmptyParser;
    type AltParser = NeverParser;

    fn seq(self) -> EmptyParser {
        EmptyParser
    }
    fn alt(self) -> NeverParser {
        NeverParser
    }
}

impl<'parse, 'source, A> ParserTuple<'parse, 'source> for (A,)
where
    A: Parser<'parse, 'source>,
{
    type SeqParser = A;
    type AltParser = A;

    fn seq(self) -> A {
        self.0
    }
    fn alt(self) -> A {
        self.0
    }
}

impl<'parse, 'source, A, B> ParserTuple<'parse, 'source> for (A, B)
where
    A: Parser<'parse, 'source>,
    B: Parser<'parse, 'source>,
{
    type SeqParser = SequenceParser<A, B>;
    type AltParser = EitherParser<A, B>;

    fn seq(self) -> SequenceParser<A, B> {
        let (a, b) = self;
        parser::sequence(a, b)
    }

    fn alt(self) -> EitherParser<A, B> {
        todo!("what a disaster")
        //let (a, b) = self;
        //parser::either(a, b)
    }
}
