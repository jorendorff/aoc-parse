// --- TupleConcat

pub trait TupleConcat<Tail> {
    type Output: ParserOutput;

    fn concat(self, tail: Tail) -> Self::Output;
}

macro_rules! impl_tuple_concat_item {
    ( ( $( $t1:ident , )* ) ( $( $t2:ident , )* ) ) => {
        impl< $( $t1 , )* $( $t2 , )* > TupleConcat<( $( $t2 , )* )> for ( $( $t1 , )* ) {
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

// --- ParserOutput

// there could be a special non-tuple ParserOutput type for character output such that Vec<char> ends up as String instead;
// or, parsers could have a toggle that says whether they produce "data" or the exact characters parsed

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
