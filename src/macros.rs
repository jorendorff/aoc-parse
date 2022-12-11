pub use crate::parsers::{alt, empty, lines, opt, parenthesize, plus, sequence, star};

/// Macro that creates a parser for a given pattern.
///
/// See [the top-level documentation][lib] for more about how to write patterns.
///
/// Here's a formal syntax for patterns:
///
/// ```text
/// pattern ::= expr
///
/// expr ::= label
///   | label "=>" rust_expr    -- custom conversion
///
/// label ::= cast
///   | ident ":" seq           -- labeled subpattern
///
/// seq ::= term
///   | seq term                -- concatenated subpatterns
///
/// term ::= prim
///   | term "*"                -- optional repeating
///   | term "+"                -- repeating
///   | term "?"                -- optional
///
/// prim ::= "(" expr ")"
///   | ident "(" expr,* ")"    -- function call
///   | ident                   -- named parser (when not followed by `(`)
///   | literal                 -- exact string
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
    // aoc_parse_helper!(@seq label [expr] [stack] [patterns])
    //
    // Submacro to transform a pattern matching `expr` to a Rust Parser
    // expression; except if `expr` was labeled, the label must already have
    // been stripped off and passed in as `label`.
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
    (@seq _ [ => $mapper:expr ] [ $($stack:tt)* ] [ $($pats:tt ,)* ]) => {
        $crate::Parser::map(
            $crate::aoc_parse_helper!(@seq _ [] [ $($stack)* ] [ $($pats ,)* ]) ,
            | ( $crate::aoc_parse_helper!(@reverse_pats [ $($pats ,)* ] []) ) | $mapper ,
        )
    };

    // Mapper at the end of a pattern that is labeled.
    //
    // Experimentally: In `(foo: t1 ", " t2 => f1(foo)) => f2(foo)` the pair is
    // available as `foo` in the inner mapper. Then the result of the inner
    // mapper is named `foo` in the outer mapper.
    (@seq $label:ident [ => $mapper:expr ] [ $($stack:tt)* ] [ $($pats:tt ,)* ]) => {
        $crate::Parser::map(
            $crate::aoc_parse_helper!(@seq $label [] [ $($stack)* ] [ $($pats ,)* ]) ,
            | $label @ ( $crate::aoc_parse_helper!(@reverse_pats [ $($pats ,)* ] []) ) | $mapper ,
        )
    };

    // Recognize `as` keyword, not yet supported.
    //
    // By design of the syntax, `as` only happens at the end of input here.
    (@seq $label:tt [ as $ty:ty ] [ $top:expr , $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        todo!("cast syntax")
    };

    // Reject unsupported non-greedy regex syntax.
    (@seq $label:tt [ * ? $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("non-greedy quantifier `*?` is not supported")
    };

    // Reject unsupported non-greedy regex syntax.
    (@seq $label:tt [ + ? $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("non-greedy quantifier `+?` is not supported")
    };

    // Detect Kleene * and apply it to the preceding term.
    (@seq $label:tt [ * $($tail:tt)* ] [ $top:expr , $($stack:expr ,)* ] [ $top_pat:tt , $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(@seq $label [ $($tail)* ] [ $crate::macros::star($top) , $($stack ,)* ] [ _ , $($pats ,)* ])
    };

    // Detect Kleene + and apply it to the preceding term.
    (@seq $label:tt [ + $($tail:tt)* ] [ $top:expr , $($stack:expr ,)* ] [ $top_pat:tt , $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(@seq $label [ $($tail)* ] [ $crate::macros::plus($top) , $($stack ,)* ] [ _ , $($pats ,)* ])
    };

    // Detect optional `?` and apply it to the preceding term.
    (@seq $label:tt [ ? $($tail:tt)* ] [ $top:expr , $($stack:tt)* ] [ $top_pat:tt , $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(@seq $label [ $($tail)* ] [ $crate::macros::opt($top) , $($stack)* ] [ _ , $($pats ,)* ])
    };

    // A quantifier at the beginning of input (nothing on the stack) is an errror.
    (@seq $label:tt [ * $($tail:tt)* ] [ ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("quantifier `*` has to come after something, not at the start of an expression.")
    };
    (@seq $label:tt [ + $($tail:tt)* ] [ ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("quantifier `+` has to come after something, not at the start of an expression.")
    };
    (@seq $label:tt [ ? $($tail:tt)* ] [ ] [ $($pats:tt ,)* ]) => {
        core::compile_error!("quantifier `?` has to come after something, not at the start of an expression.")
    };

    // call syntax
    (@seq $label:tt [ $f:ident ( $($args:tt)* ) $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(
            @seq $label
            [ $($tail)* ]
            [
                $crate::aoc_parse_helper!(@args ( $f ) [ $( $args )* ] [] ())
                ,
                $($stack ,)*
            ]
            [ _ , $($pats ,)* ]
        )
    };

    // parenthesized subpattern with a label
    (@seq $label:tt [ ( $sublabel:ident : $($expr:tt)* ) $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(
            @seq $label
            [ $($tail)* ]
            [
                $crate::aoc_parse_helper!(@prim ( $($expr)* )) ,
                $($stack ,)*
            ]
            [ $sublabel , $($pats ,)* ]
        )
    };

    // string literal
    (@seq $label:tt [ $x:literal $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(
            @seq $label
            [ $($tail)* ]
            [
                $crate::aoc_parse_helper!(@prim $x) ,
                $($stack ,)*
            ]
            [ #, /* no pattern */ $($pats ,)* ]
        )
    };

    // the first `tt` of any other `term`
    (@seq $label:tt [ $x:tt $($tail:tt)* ] [ $($stack:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(
            @seq $label
            [ $($tail)* ]
            [
                $crate::aoc_parse_helper!(@prim $x) ,
                $($stack ,)*
            ]
            [ _ , $($pats ,)* ]
        )
    };

    // end of input
    (@seq $label:tt [ ] [ $($parts:expr ,)* ] [ $($pats:tt ,)* ]) => {
        $crate::aoc_parse_helper!(@reverse [ $($parts ,)* ] [])
    };

    // anything not matched by this point is an error
    (@seq $label:tt [ $($tail:tt)* ] [ $($parts:expr ,)* ] [ $($pats:tt ,)* ]) => {
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

    // aoc_parse_helper!(@reverse_pats [pattern stack] [output stack])
    //
    // Take the stack of Rust patterns, possibly interspersed with `#`
    // to indicate "no pattern", and produce a single pattern.
    (@reverse_pats [ ] [ $out:pat , ]) => {
        $out  // don't produce a singleton-tuple-pattern
    };
    (@reverse_pats [ ] [ $($out:pat ,)* ]) => {
        ( $( $out , )* )
    };
    (@reverse_pats [ #, $($tail:tt ,)* ] [ $( $out:pat , )* ]) => {
        $crate::aoc_parse_helper!(@reverse_pats [ $( $tail , )* ] [ $( $out , ) * ])
    };
    (@reverse_pats [ $head:pat , $($tail:tt ,)* ] [ $($out:pat ,)* ]) => {
        $crate::aoc_parse_helper!(@reverse_pats [ $( $tail , )* ] [ $head, $($out ,)* ])
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
        $crate::macros::parenthesize(
            $crate::aoc_parse_helper!(@seq _ [ $( $nested )* ] [ ] [ ])
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
                $crate::aoc_parse_helper!(@seq _ [ $( $seq )* ] [ ] [ ]) ,
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
            [ $crate::aoc_parse_helper!(@seq _ [ $( $seq )* ] [ ] [ ]) ]
        )
    };

    // end of a non-first arm of an alternation
    (@list [ , $($tail:tt)* ] [ $($seq:tt)* ] [ $out:expr ]) => {
        $crate::aoc_parse_helper!(
            @list
            [ $( $tail )* ]
            [ ]
            [ $crate::macros::alt($out, $crate::aoc_parse_helper!(@seq _ [ $( $seq )* ] [ ] [ ])) ]
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

    // aoc_parse_helper!(ident : ...) - Labeled expression, `label ::= ident : cast`
    ($label:ident : $($tail:tt)*) => {
        $crate::macros::parenthesize(
            $crate::aoc_parse_helper!(@seq $label [ $($tail)* ] [ ] [ ])
        )
    };

    // Hand anything else off to the @seq submacro.
    ($($tail:tt)*) => {
        $crate::macros::parenthesize(
            $crate::aoc_parse_helper!(@seq _ [ $($tail)* ] [ ] [ ])
        )
    };
}
