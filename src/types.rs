//! Traits for types that are used as the `RawOutput` type of parsers.

// There are finitely many flavors of RawOutput, necessarily, because we need
// to implement Concat for all combinations, an O(n^2) matrix.
//
// The RawOutput flavors are:
//
// -   `()` - The match doesn't produce any interesting output. Used for exact strings.
//
// -   `(T,)` - Singleton tuple. Used for almost everything, including optional parsers
//     `(Option<T>,)` and repeating parsers `(Vec<T>,)`.
//
// -   `(A, B, ...)` - Tuple. Produced by concatenation.
//
// I'm planning to add one more, a "text" type that I'll use for `alpha` and such,
// and the special feature is that when it concatenates with other text, you get text
// rather than a tuple.
//
//     parser!(u64 any_char u64)      --> Output=(u64, char, u64)
//     parser!(alpha+ digit*)         --> Output=String
//
// This would be rather magical. It would be necessary to cast if you *don't* want the
// automatic conversion, maybe using the syntax `(alpha as char)+`. Or just parentheses.
// Still thinking about just how I would want it to work.

/// A type that can be used as a parser's `RawOutput` type.
///
/// A parser's raw output type is used by parser combinators; end users
/// typically shouldn't ever think about it. But the RawOutput type determines
/// the user-friendly `Output` type (it's the `UserType` defined here).
pub trait ParserOutput {
    type UserType;

    fn into_user_type(self) -> Self::UserType;
}

impl ParserOutput for () {
    type UserType = Self;

    fn into_user_type(self) {}
}

impl<A> ParserOutput for (A,) {
    type UserType = A;

    fn into_user_type(self) -> A {
        self.0
    }
}

macro_rules! impl_parser_output {
    ( $( $t:ident )* ) => {
        impl < $($t,)* > ParserOutput for ( $($t,)* ) {
            type UserType = Self;

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

/// Trait used to concatenate two RawOutput types. SequenceParser uses this.
///
/// For every pair of `RawOutput` types `A` and `B`, `A` must implement
/// `RawOutputConcat<B>`.
pub trait RawOutputConcat<Tail> {
    type Output: ParserOutput;

    fn concat(self, tail: Tail) -> Self::Output;
}

macro_rules! impl_tuple_concat_item {
    ( ( $( $t1:ident , )* ) ( $( $t2:ident , )* ) ) => {
        impl< $( $t1 , )* $( $t2 , )* > RawOutputConcat<( $( $t2 , )* )> for ( $( $t1 , )* ) {
            type Output = ( $( $t1 , )* $( $t2 , )* );

            #[allow(non_snake_case)]
            fn concat(self, ( $( $t2 , )* ) : ( $( $t2 , )* )) -> Self::Output {
                let ( $( $t1 , )* ) = self;
                let combined = ( $( $t1 , )* $( $t2 , )* );
                combined
            }
        }
    };
}

macro_rules! impl_tuple_concat_helper {
    ( [] ( $( $t1:ident , )* ) ( $( $t2:ident , )* ) ) => {
        impl_tuple_concat_item!( ( $( $t1 , )* ) ( $( $t2 , )* ) );
    };
    ( [ $next:ident $( $rest:ident )* ] ( $( $t1:ident , )* ) ( $( $t2:ident , )* ) ) => {
        impl_tuple_concat_item!( ( $( $t1 , )* ) ( $( $t2 , )* ) );
        impl_tuple_concat_helper!( [ $( $rest )* ] ( $( $t1 , )* ) ( $( $t2 , )* $next , ) );
    };
}

macro_rules! impl_tuple_concat {
    ( [ ] ( $( $t1:ident , )* ) ) => {
        impl_tuple_concat_helper!( [] ( $( $t1 ,)* ) ( ) );
    };
    ( [ $next:ident $( $rest:ident )* ] ( $( $t1:ident , )* ) ) => {
        impl_tuple_concat_helper!( [ $next $( $rest )* ] ( $( $t1 ,)* ) () );
        impl_tuple_concat!( [ $($rest)* ] ( $( $t1 , )* $next , ) );
    };
}

impl_tuple_concat!([A B C D E F G H] ());
