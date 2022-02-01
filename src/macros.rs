use crate::matches::Match;
use crate::parser::{self, Parser};
use crate::ParseError;

#[non_exhaustive]
pub enum ParserExpr {
    StringLiteral(&'static str),
    Map(
        Box<ParserExpr>,
        Box<dyn Fn(Match) -> Result<Match, ParseError>>,
    ),
    Label(String, Box<ParserExpr>),
    Sequence(Vec<ParserExpr>),
    OneOf(Vec<ParserExpr>),
    Star(Box<ParserExpr>),
    Plus(Box<ParserExpr>),
    Optional(Box<ParserExpr>),
    Identifier(String),
    Call(String, Vec<ParserExpr>),
}

impl ParserExpr {
    pub fn evaluate(self) -> Parser {
        match self {
            ParserExpr::StringLiteral(s) => parser::exact(s),
            ParserExpr::Map(_p, _mapper) => todo!("map syntax support"),
            ParserExpr::Label(_label, _parser) => todo!("label syntax support"),
            ParserExpr::Sequence(parsers) => parser::sequence(
                parsers
                    .into_iter()
                    .map(ParserExpr::evaluate)
                    .collect::<Vec<Parser>>(),
            ),
            ParserExpr::OneOf(parsers) => parser::one_of(
                parsers
                    .into_iter()
                    .map(ParserExpr::evaluate)
                    .collect::<Vec<Parser>>(),
            ),
            ParserExpr::Star(expr) => parser::star((*expr).evaluate()),
            ParserExpr::Plus(expr) => parser::plus((*expr).evaluate()),
            ParserExpr::Optional(expr) => parser::opt((*expr).evaluate()),
            ParserExpr::Identifier(_name) => todo!("named parser support"),
            ParserExpr::Call(name, mut arguments) => match &name as &str {
                "lines" => {
                    if arguments.len() != 1 {
                        panic!("lines() takes 1 argument, got {}", arguments.len());
                    }
                    parser::repeat(
                        "".to_string(),
                        arguments.pop().unwrap().evaluate(),
                        parser::exact("\n"),
                        0,
                        None,
                        true,
                    )
                }
                _ => todo!("unexpected function `{}`", name),
            },
        }
    }
}

pub fn literal(s: &'static str) -> ParserExpr {
    ParserExpr::StringLiteral(s)
}

pub fn sequence(exprs: Vec<ParserExpr>) -> ParserExpr {
    ParserExpr::Sequence(exprs)
}

pub fn one_of(exprs: Vec<ParserExpr>) -> ParserExpr {
    ParserExpr::OneOf(exprs)
}

pub fn star(expr: ParserExpr) -> ParserExpr {
    ParserExpr::Star(Box::new(expr))
}

pub fn identifier(ident: &'static str) -> ParserExpr {
    ParserExpr::Identifier(ident.to_string())
}

pub fn call(ident: &'static str, arguments: Vec<ParserExpr>) -> ParserExpr {
    ParserExpr::Call(ident.to_string(), arguments)
}

/// ```text
/// ident ::= a Rust identifier
/// expr ::= a Rust expression
/// literal ::= a Rust literal
///
/// parser ::= rule* expr
///
/// rule ::= "rule" ident "=" expr ";"
///
/// expr ::= cast
///   | seq "=>" expr               => Map($1, Box::new(|...fields of $1...| Ok($2)))
///   | ident ":" cast              => Label($1, $2)
///
/// cast ::= seq
///   | seq "as" ty                 => Map($1, |x| $2::try_from(x))
///
/// seq ::= term
///   | seq SP term                 => $1.seq($2)
///
/// term ::= prim
///   | prim "*"                    => Repeat($1, 0)
///   | prim "+"                    => Repeat($1, 1)
///   | prim "?"                    => Optional($1)
///   | ident NOSP "(" expr,* ")"   => Call($1, $2)
///
/// prim ::= "(" expr ")"
///   | ident                       => NamedRule($1)
///   | literal                     => Literal($1)
///   | "{" expr,* "}"              => OneOf($1)
/// ```
#[macro_export]
macro_rules! parser {
    (@seq [ $($stack:tt)* ] => $mapper:expr) => {
        todo!("map syntax")
    };
    (@seq [ $top:expr , $($stack:expr ,)* ] as $ty:ty) => {
        todo!("cast syntax")
    };
    (@seq [ $top:expr , $($stack:expr ,)* ] * $($tail:tt)*) => {
        $crate::parser!(@seq [ $crate::macros::star($top) , $($stack ,)* ] $($tail)*)
    };
    (@seq [ $top:expr , $($stack:expr ,)* ] + $($tail:tt)*) => {
        $crate::parser!(@seq [ $crate::macros::plus($top) , $($stack ,)* ] $($tail)*)
    };
    (@seq [ $top:expr , $($stack:tt)* ] ? $($tail:tt)*) => {
        $crate::parser!(@seq [ $crate::macros::opt($top) , $($stack)* ] $($tail)*)
    };
    // call syntax
    (@seq [ $($stack:expr ,)* ] $f:ident ( $($args:tt)* ) $($tail:tt)*) => {
        $crate::parser!(
            @seq
            [
                $crate::macros::call(stringify!($f), $crate::parser!(@list [$($args)*] [] []))
                ,
                $($stack ,)*
            ]
            $($tail)*
        )
    };
    (@seq [ $($stack:expr ,)* ] $x:tt $($tail:tt)*) => {
        $crate::parser!(@seq [ $crate::parser!(@prim $x) , $($stack ,)* ] $($tail)*)
    };
    (@seq [ $($parts:expr ,)* ] /* end of input */) => {
        $crate::macros::sequence($crate::parser!(@reverse [ $($parts ,)* ] []))
    };
    (@seq [ $($parts:expr ,)* ] $($tail:tt)*) => {
        ::core::compile_error!(stringify!(unrecognized syntax @ $($tail)*))
    };

    (@reverse [ ] [ $($exprs:expr ,)* ]) => {
        vec![$($exprs ,)*]
    };
    (@reverse [ $head:expr , $($tail:expr ,)* ] [ $($exprs:expr ,)* ]) => {
        $crate::parser!(@reverse [ $($tail ,)* ] [ $head , $($exprs ,)* ])
    };

    (@prim $x:ident) => {
        $crate::macros::identifier(stringify!($x))
    };
    (@prim $x:literal) => {
        $crate::macros::literal($x)
    };
    (@prim ( $($nested:tt)* )) => {
        $crate::parser!(@seq [ ] $($nested)*)
    };
    (@prim { $($nested:tt)* }) => {
        $crate::macros::one_of($crate::parser!(@list [ $( $nested )* ] [] []))
    };

    (@list [ , $($tail:tt)* ] [ $($seq:tt)* ] [ $($exprs:expr ,)* ]) => {
        $crate::parser!(
            @list
            [ $( $tail )* ]
            [ ]
            [ $( $exprs , )* $crate::parser!(@seq [ ] $( $seq )*) , ]
        )
    };
    (@list [ $next:tt $($tail:tt)* ] [ $($seq:tt)* ] [ $($exprs:expr ,)* ]) => {
        $crate::parser!(
            @list
            [ $( $tail )* ]
            [ $( $seq )* $next ]
            [ $( $exprs , )* ]
        )
    };
    (@list [ /*end of input*/ ] [ ] [ $($exprs:expr ,)* ]) => {
        vec![ $($exprs ,)* ]
    };
    (@list [ /*end of input*/ ] [ $($seq:tt)+ ] [ $($exprs:expr ,)* ]) => {
        vec![ $($exprs ,)* $crate::parser!(@seq [ ] $($seq)+), ]
    };

    (@ $($tail:tt)*) => {
        ::core::compile_error!(stringify!(unrecognized syntax @ $($tail)*))
    };

    (rule $name:ident = $($body:tt)* ; $($tail:tt)*) => {
        todo!("rule syntax")
    };
    ($label:ident : $($tail:tt)*) => {
        todo!("label syntax")
    };
    ($($tail:tt)*) => {
        $crate::parser!(@seq [ ] $($tail)*).evaluate().with_source(stringify!( $( $tail )* ))
    };
}
