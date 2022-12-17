//! Implementation of the `parser!` macro.
//!
//! # Syntax errors
//!
//! The doc-tests below address cases where the pattern is invalid and
//! compilation should fail.
//!
//! ```compile_fail
//! # use aoc_parse::parser;
//! let p = parser!(label:);
//! //      ^ERROR: missing pattern after `label:`
//! ```
//!
//! ```compile_fail
//! # use aoc_parse::parser;
//! let p = parser!(nothing: => nothing);
//! //      ^ERROR: missing pattern after `nothing:`
//! ```
//!
//! ```compile_fail
//! # use aoc_parse::{parser, prelude::*};
//! let p = parser!("double label " a:b:u32);
//! //      ^ERROR: missing pattern between `a:` and `b:`
//! ```
//!
//! ```compile_fail
//! # use aoc_parse::{parser, prelude::*};
//! let p = parser!(alpha*?);
//! //      ^ERROR: non-greedy quantifier `*?` is not supported
//! ```
//!
//! ```compile_fail
//! # use aoc_parse::{parser, prelude::*};
//! let p = parser!(? "hello world");
//! //      ^ERROR: quantifier `?` has to come after something
//! ```
//!
//! This one is not something we can detect at macro-expand time,
//! but the ambiguity in `ident ( )` between function call and concatenation
//! is always resolved in favor of a function call, regardless of whether `ident`
//! actually names a function. We let Rust typeck flag the error if the user
//! intended concatenation.
//!
//! ```compile_fail
//! # use aoc_parse::{parser, prelude::*};
//! const PROMPT: &str = "> ";
//! let p = parser!(PROMPT (alpha '-')*);
//! //      ^ERROR: call expression requires function
//! ```

pub use crate::parsers::{alt, empty, lines, map, opt, pair, plus, sequence, single_value, star};

/// Macro that creates a parser for a given pattern.
///
/// See [the top-level documentation][lib] for more about how to write patterns.
///
/// Here's a formal syntax for patterns:
///
/// ```text
/// pattern ::= expr
///
/// expr ::= seq
///   | seq "=>" rust_expr      -- custom conversion
///
/// seq ::= lterm
///   | seq lterm               -- concatenated subpatterns
///
/// lterm ::= term
///   | ident ":" term          -- labeled subpattern
///
/// term ::= prim
///   | term "*"                -- optional repeating
///   | term "+"                -- repeating
///   | term "?"                -- optional
///
/// prim ::= "(" expr ")"
///   | ident "(" expr,* ")"    -- function call
///   | ident                   -- named parser (when not followed by `(`)
///   | literal                 -- exact char or string
///   | "{" expr,* "}"          -- one-of syntax
///
/// ident ::= a Rust identifier
/// expr ::= a Rust expression
/// literal ::= a Rust literal
/// ```
///
/// [lib]: crate#patterns
#[macro_export]
macro_rules! parser {
    ($($pattern:tt)*) => { $crate::aoc_parse_helper!( $( $pattern )* ) }
}

#[macro_export]
#[doc(hidden)]
macro_rules! aoc_parse_helper {
    // aoc_parse_helper!(@seq [expr] [stack] [patterns])
    //
    // Submacro to transform a pattern matching `expr` to a Rust Parser
    // expression.
    //
    // Gradually parses the tokens in `expr`, producing `stack` (in reverse)
    // and `patterns` (in reverse for no good reason), then at the end converts
    // those output-stacks into a Rust parser-expression using `@reverse` and
    // `@reverse_pats`.
    //
    // `stack` is a list of Rust expressions, parsers for the elements of the
    // `expr`. `patterns` is a list of patterns that match the output of the
    // overall SequenceParser we will build from the bits in `stack`.
    //
    // BUG: Because of the simplistic way this macro-parses the input, it
    // doesn't reject some bad syntax like `foo?(x)` or `foo??` or `foo++`.

    // Mapper at the end of a pattern that is not labeled, `expr ::= label => rust_expr`.
    (@seq [ => $mapper:expr ] [ $($stack:tt)* ] [ $($pats:tt ,)* ]) => {
        $crate::macros::map(
            $crate::aoc_parse_helper!(@reverse_map [ $($stack)* ] []),
            | ( $crate::aoc_parse_helper!(@reverse_pats [ $($pats ,)* ] []) ) | $mapper ,
        )
    };

    // Reject unsupported non-greedy regex syntax.
    (@seq [ * ? $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("non-greedy quantifier `*?` is not supported")
    };

    // Reject unsupported non-greedy regex syntax.
    (@seq [ + ? $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("non-greedy quantifier `+?` is not supported")
    };

    // Detect Kleene * and apply it to the preceding term.
    (@seq [ * $($tail:tt)* ] [ $top:expr , $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(@seq [ $($tail)* ] [ $crate::macros::star($top) , $($stack ,)* ] [ $($pats ,)* ])
    };

    // Detect Kleene + and apply it to the preceding term.
    (@seq [ + $($tail:tt)* ] [ $top:expr , $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(@seq [ $($tail)* ] [ $crate::macros::plus($top) , $($stack ,)* ] [ $($pats ,)* ])
    };

    // Detect optional `?` and apply it to the preceding term.
    (@seq [ ? $($tail:tt)* ] [ $top:expr , $($stack:tt)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(@seq [ $($tail)* ] [ $crate::macros::opt($top) , $($stack)* ] [ $($pats ,)* ])
    };

    // A quantifier at the beginning of input (nothing on the stack) is an errror.
    (@seq [ * $($tail:tt)* ] [ ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("quantifier `*` has to come after something, not at the start of an expression.")
    };
    (@seq [ + $($tail:tt)* ] [ ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("quantifier `+` has to come after something, not at the start of an expression.")
    };
    (@seq [ ? $($tail:tt)* ] [ ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("quantifier `?` has to come after something, not at the start of an expression.")
    };

    // Reject incorrect label syntax.
    (@seq [ $label:ident : => $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        core::compile_error!(
            core::concat!("missing pattern after `", core::stringify!($label), ":`")
        );
    };
    (@seq [ $label:ident : ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        core::compile_error!(
            core::concat!("missing pattern after `", core::stringify!($label), ":`")
        );
    };
    (@seq [ $label1:ident : $label2:ident : $( $tail:tt )* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        core::compile_error!(
            core::concat!(
                "missing pattern between `", core::stringify!($label1), ":` and `",
                    core::stringify!($label2), ":`"
            )
        );
    };

    // Function call
    (@seq [ $f:ident ( $($args:tt)* ) $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(
            @seq
            [ $($tail)* ]
            [
                $crate::aoc_parse_helper!(@args ( $f ) [ $( $args )* ] [] ())
                ,
                $($stack ,)*
            ]
            [ _ , $($pats ,)* ]
        )
    };

    // Labelled function call
    (@seq [ $label:ident : $f:ident ( $( $args:tt )* )  $( $tail:tt )* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(
            @seq
            [ $($tail)* ]
            [
                $crate::aoc_parse_helper!(@args ( $f ) [ $( $args )* ] [] ())
                ,
                $($stack ,)*
            ]
            [ $label , $($pats ,)* ]
        )
    };

    // any Rust literal (strings and chars are valid patterns; others may be
    // used as function arguments)
    (@seq [ $x:literal $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(
            @seq
            [ $($tail)* ]
            [
                $crate::aoc_parse_helper!(@prim $x) ,
                $($stack ,)*
            ]
            [ _, $($pats ,)* ]
        )
    };

    // Other labeled term
    (@seq [ $label:ident : $x:tt $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(
            @seq
            [ $($tail)* ]
            [
                $crate::aoc_parse_helper!(@prim $x) ,
                $($stack ,)*
            ]
            [ $label , $($pats ,)* ]
        )
    };

    // the first `tt` of any other `term`
    (@seq [ $x:tt $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(
            @seq
            [ $($tail)* ]
            [
                $crate::aoc_parse_helper!(@prim $x) ,
                $($stack ,)*
            ]
            [ _ , $($pats ,)* ]
        )
    };

    // end of input
    (@seq [ ] [ $($parts:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(@reverse [ $($parts ,)* ] [])
    };

    // anything not matched by this point is an error
    (@seq [ $($tail:tt)* ] [ $($parts:expr ,)* ] [ $($pats:tt ,)* ]) => {
        core::compile_error!(stringify!(unrecognized syntax @ $($tail)*))
    };

    // aoc_parse_helper!(@reverse [input expr stack] [output stack])
    //
    // Take the stack of parsers and produce a single sequence-parser.
    (@reverse [ ] [ ]) => {
        $crate::macros::empty()
    };
    (@reverse [ ] [ $out:expr ]) => {
        $out
    };
    (@reverse [ $head:expr , $($tail:expr ,)* ] [ ]) => {
        $crate::aoc_parse_helper!(@reverse [ $($tail ,)* ] [ $head ])
    };
    (@reverse [ $head:expr , $($tail:expr ,)* ] [ $out:expr ]) => {
        $crate::aoc_parse_helper!(@reverse [ $($tail ,)* ] [ $crate::macros::sequence($head, $out) ])
    };

    // aoc_parse_helper!(@reverse_map [input expr stack] [output expr])
    //
    // Take the stack of parsers and make a single parser that produces nested
    // pairs. For example, if the input is [d, c, b, a] this produces the
    // output `pair(a, pair(b, pair(c, d)))`.
    (@reverse_map [] []) => {
        $crate::macros::empty()
    };
    (@reverse_map [] [ $out:expr ]) => {
        $out
    };
    (@reverse_map [ $head:expr , $( $tail:expr , )* ] []) => {
        $crate::aoc_parse_helper!(@reverse_map [ $( $tail , )* ] [ $head ])
    };
    (@reverse_map [ $head:expr , $( $tail:expr , )* ] [ $out:expr ]) => {
        $crate::aoc_parse_helper!(
            @reverse_map
                [ $( $tail , )* ]
                [ $crate::macros::pair($head, $out) ]
        )
    };

    // aoc_parse_helper!(@reverse_pats [pattern stack] [output stack])
    //
    // Take the stack of Rust patterns and produce a single pattern.
    (@reverse_pats [] []) => {
        ()
    };
    (@reverse_pats [] [ $out:pat ]) => {
        $out
    };
    (@reverse_pats [ $head:pat , $( $tail:pat , )* ] []) => {
        $crate::aoc_parse_helper!(@reverse_pats [ $( $tail , )* ] [ $head ])
    };
    (@reverse_pats [ $head:pat , $( $tail:pat , )* ] [ $out:pat ]) => {
        $crate::aoc_parse_helper!(@reverse_pats [ $( $tail , )* ] [ ($head, $out) ])
    };

    // aoc_parse_helper!(@prim pattern)
    //
    // Transform a `prim` into a Rust Parser expression.
    (@prim $x:ident) => {
        $x
    };
    (@prim $x:literal) => {
        $x
    };
    (@prim ( $($nested:tt)* )) => {
        $crate::macros::single_value(
            $crate::aoc_parse_helper!(@seq [ $( $nested )* ] [ ] [ ])
        )
    };
    (@prim { $($nested:tt)* }) => {
        $crate::aoc_parse_helper!(@list [ $( $nested )* ] [ ] [ ])
    };

    // aoc_parse_helper!(@args fn_expr [unexamined input tokens] [current argument holding area] [transformed output argument exprs])
    //
    // Transform argument lists.

    // end of an argument in an argument list
    (@args ( $f:expr ) [ , $($tail:tt)* ] [ $($seq:tt)* ] ( $( $arg:expr , )* )) => {
        $crate::aoc_parse_helper!(
            @args
            ( $f )
            [ $( $tail )* ]
            [ ]
            (
                $( $arg , )*
                $crate::aoc_parse_helper!(@seq [ $( $seq )* ] [ ] [ ]) ,
            )
        )
    };

    // not the end of an arg; just move a token from the input to the holding area
    (@args ( $f:expr ) [ $next:tt $($tail:tt)* ] [ $($seq:tt)* ] ( $( $out:expr , )* )) => {
        $crate::aoc_parse_helper!(
            @args
            ( $f )
            [ $( $tail )* ]
            [ $( $seq )* $next ]
            ( $( $out , )* )
        )
    };

    // end of argument list, after trailing comma or empty
    (@args ( $f:expr ) [] [] ( $( $out:expr , )* )) => {
        $f ( $( $out , )* )
    };

    // end of argument list with no trailing comma: infer one
    (@args ( $f:expr ) [] [ $($seq:tt)+ ] ( $( $out:expr , )* )) => {
        $crate::aoc_parse_helper!(@args ( $f ) [,] [ $($seq)+ ] ( $( $out , )* ))
    };

    // aoc_parse_helper!(@list [unexamined input tokens] [current arm holding area] [transformed output arm parser expressions])
    //
    // The list of patterns in the body of an alternation.

    // end of first arm of an alternation
    (@list [ , $($tail:tt)* ] [ $($seq:tt)* ] [ ]) => {
        $crate::aoc_parse_helper!(
            @list
            [ $( $tail )* ]
            [ ]
            [ $crate::aoc_parse_helper!(@seq [ $( $seq )* ] [ ] [ ]) ]
        )
    };

    // end of a non-first arm of an alternation
    (@list [ , $($tail:tt)* ] [ $($seq:tt)* ] [ $out:expr ]) => {
        $crate::aoc_parse_helper!(
            @list
            [ $( $tail )* ]
            [ ]
            [ $crate::macros::alt($out, $crate::aoc_parse_helper!(@seq [ $( $seq )* ] [ ] [ ])) ]
        )
    };

    // not the end of an arm; just move a token from the input to the holding area
    (@list [ $next:tt $($tail:tt)* ] [ $($seq:tt)* ] [ $($out:expr)? ]) => {
        $crate::aoc_parse_helper!(
            @list
            [ $( $tail )* ]
            [ $( $seq )* $next ]
            [ $( $out )? ]
        )
    };

    // completely empty alternation; could technically be understood as never matching,
    // but it's not a useful thing to express, so reject.
    (@list [ ] [ ] [ ]) => {
        ::core::compile_error("no arms in alternation")
    };

    // end of alternation after comma
    (@list [ ] [ ] [ $out:expr ]) => {
        $out
    };

    // end of alternation with no comma: infer one
    (@list [ ] [ $($seq:tt)+ ] [ $( $out:expr )? ]) => {
        $crate::aoc_parse_helper!(@list [,] [ $($seq)+ ] [ $( $out )? ])
    };

    // aoc_parse_helper!(@...) - This is an internal error, shouldn't happen in the wild.
    (@ $($tail:tt)*) => {
        ::core::compile_error!(stringify!(unrecognized syntax @ $($tail)*))
    };

    // Hand anything else off to the @seq submacro.
    ($($tail:tt)*) => {
        $crate::macros::single_value(
            $crate::aoc_parse_helper!(@seq [ $($tail)* ] [ ] [ ])
        )
    };
}
