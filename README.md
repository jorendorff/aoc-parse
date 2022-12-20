# aoc-parse

A parser library designed for Advent of Code.

This library mainly provides a macro, `parser!`, that lets you write
a custom parser for your [AoC] puzzle input in seconds.

For example, my puzzle input for [December 2, 2015][example] looked like this:

```
4x23x21
22x29x19
11x4x11
8x10x5
24x18x16
...
```

The parser for this format is a one-liner:
`parser!(lines(u64 "x" u64 "x" u64))`.

## How to use aoc-parse

It's pretty easy.

```rust
use aoc_parse::{parser, prelude::*};

let p = parser!(lines(u64 "x" u64 "x" u64));
assert_eq!(
    p.parse("4x23x21\n22x29x19\n").unwrap(),
    vec![(4, 23, 21), (22, 29, 19)]
);
```

If you're using [aoc-runner], it might look like this:

```rust
use aoc_runner_derive::*;
use aoc_parse::{parser, prelude::*};

#[aoc_generator(day2)]
fn parse_input(text: &str) -> Vec<(u64, u64, u64)> {
    let p = parser!(lines(u64 "x" u64 "x" u64));
    p.parse(text).unwrap()
}
```

## Patterns

The argument you need to pass to the `parser!` macro is a *pattern*;
all aoc-parse does is **match** strings against your chosen pattern
and **convert** them into Rust values.

Here are some examples of patterns:

```rust
lines(i32)      // matches a list of integers, one per line
                // converts them to a Vec<i32>

line(lower+)    // matches a single line of one or more lowercase letters
                // converts them to a Vec<char>

lines({         // matches lines made up of the characters < = >
    "<" => -1,  // converts them to a Vec<Vec<i32>> filled with -1, 0, and 1
    "=" => 0,
    ">" => 1
}+)
```

Here are the pieces that you can use in a pattern:

*   `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `big_int` - These match an integer,
    written out using decimal digits, with an optional `+` or `-` sign
    at the start, like `0` or `-11474`.

    It's an error if the string contains a number too big to fit in the
    type you chose. For example, `parser!(i8).parse("1000")` is an error.
    (It matches the string, but fails during the "convert" phase.)

    `big_int` parses a [`num_bigint::BigInt`].

*   `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `big_uint` - The same, but
    without the sign.

*   `i8_bin`, `i16_bin`, `i32_bin`, `i64_bin`, `i128_bin`, `isize_bin`, `big_int_bin`,
    `u8_bin`, `u16_bin`, `u32_bin`, `u64_bin`, `u128_bin`, `usize_bin`, `big_uint_bin`,
    `i8_hex`, `i16_hex`, `i32_hex`, `i64_hex`, `i128_hex`, `isize_hex`, `big_int_hex`,
    `u8_hex`, `u16_hex`, `u32_hex`, `u64_hex`, `u128_hex`, `usize_hex`, `big_uint_hex` -
    Match an integer in base 2 or base 16. The `_hex` parsers allow both
    uppercase and lowercase digits `A`-`F`.

*   `bool` - Matches either `true` or `false` and converts it to the
    corresponding `bool` value.

*   `'x'` or `"hello"` - A Rust character or string, in quotes, is a pattern
    that matches that exact text only.

    Exact patterns don't produce a value.

*   <code><var>pattern1 pattern2 pattern3</var>...</code> - Patterns can be
    concatenated to form larger patterns. This is how
    `parser!(u64 "x" u64 "x" u64)` matches the string `4x23x21`. It simply
    matches each subpattern in order. It converts the match to a tuple if
    there are two or more subpatterns that produce values.

*   <code><var>parser_var</var></code> - You can use previously defined
    parsers that you've stored in local variables.

    For example, the `amount` parser below makes use of the `fraction` parser
    defined on the previous line.

    ```
    let fraction = parser!(i64 "/" u64);
    let amount = parser!(fraction " tsp");

    assert_eq!(amount.parse("1/4 tsp").unwrap(), (1, 4));
    ```

    An identifier can also refer to a string or character constant.

Repeating patterns:

*   <code><var>pattern</var>*</code> - Any pattern followed by an asterisk
    matches that pattern zero or more times. It converts the results to a
    `Vec`. For example, `parser!("A"*)` matches the strings `A`, `AA`,
    `AAAAAAAAAAAAAA`, and so on, as well as the empty string.

*   <code><var>pattern</var>+</code> - Matches the pattern one or more times, producing a `Vec`.
    `parser!("A"+)` matches `A`, `AA`, etc., but not the empty string.

*   <code><var>pattern</var>?</code> - Optional pattern, producing a Rust `Option`. For
    example, `parser!("x=" i32?)` matches `x=123`, producing `Some(123)`;
    it also matches `x=`, producing the value `None`.

    These behave just like the `*`, `+`, and `?` special characters in
    regular expressions.

*   <code>repeat_sep(<var>pattern</var>, <var>separator</var>)</code> -
    Match the given *pattern* any number of times, separated by the *separator*.
    This converts only the bits that match *pattern* to Rust values, producing
    a `Vec`. Any parts of the string matched by *separator* are not converted.

Matching single characters:

*   `alpha`, `alnum`, `upper`, `lower` - Match single characters of
    various categories. (These use the Unicode categories, even though
    Advent of Code historically sticks to ASCII.)

*   `digit`, `digit_bin`, `digit_hex` - Match a single ASCII character
    that's a digit in base 10, base 2, or base 16, respectively.
    The digit is converted to its numeric value, as a `usize`.

*   `any_char` - Match the next character, no matter what it is (like `.`
    in a regular expression, except that `any_char` matches newline
    characters too).

*   <code>char_of(<var>str</var>)</code> - Match the next character if it's
    one of the characters in *str*. For example, `char_of(">^<v")` matches
    exactly one character, either `>`, `^`, `<`, or `v`. Returns the index
    of the character within the list of options (in this case, `0`, `1`,
    `2`, or `3`).

Matching multiple characters:

*   <code>string(<var>pattern</var>)</code> - Matches the given *pattern*,
    but instead of converting it to some value, simply return the matched
    characters as a `String`.

    By default, `alpha+` returns a `Vec<char>`, and sometimes that is handy
    in AoC, but often it's better to have it return a `String`.

Custom conversion:

*   <code>... <var>name1</var>:<var>pattern1</var> ... => <var>expr</var></code> -
    On successfully matching the patterns to the left of `=>`, evaluate the Rust
    expression *expr* to convert the results to a single Rust value.

    Use this to convert input to structs. For instance, suppose your puzzle input
    contains each elf's name and height:

    ```text
    Holly=33
    Ivy=7
    DouglasFir=1093
    ```

    and you'd like to turn this into a vector of `struct Elf` values. The
    code you need is:

    ```
    struct Elf {
        name: String,
        height: u32,
    }

    let p = parser!(lines(
        elf:string(alpha+) '=' ht:u32 => Elf { name: elf, height: ht }
    ));
    ```

    The name `elf` applies to the pattern `string(alpha+)` and the name
    `ht` applies to the pattern `i32`. The bit after the `=>` is
    plain old Rust code.

    The *name*s are in scope only for the following *expr* in the same
    set of matching parentheses or braces.

Alternatives:

*   <code>{<var>pattern1</var>, <var>pattern2</var>, ...}</code> -
    Matches any one of the *patterns*. First try matching *pattern1*; if it
    matches, stop. If not, try *pattern2*, and so on. All the patterns must
    produce the same type of Rust value.

    This is sort of like a Rust `match` expression.

    For example, `parser!({"<" => -1, ">" => 1})` either matches `<`,
    returning the value `-1`, or matches `>`, returing `1`.

    Alternatives are handy when you want to convert the input into an enum.
    For example, my puzzle input for December 23, 2015 was a list of instructions
    that looked (in part) like this:

    ```text
    jie a, +4
    tpl a
    inc a
    jmp +2
    hlf a
    jmp -7
    ```

    This can be easily parsed into a vector of beautiful enums, like so:

    ```
    enum Reg {
        A,
        B,
    }

    enum Insn {
        Hlf(Reg),
        Tpl(Reg),
        Inc(Reg),
        Jmp(isize),
        Jie(Reg, isize),
        Jio(Reg, isize),
    }

    use Reg::*;
    use Insn::*;

    let reg = parser!({"a" => A, "b" => B});
    let p = parser!(lines({
        "hlf " r:reg => Hlf(r),
        "tpl " r:reg => Tpl(r),
        "inc " r:reg => Inc(r),
        "jmp " offset:isize => Jmp(offset),
        "jie " r:reg ", " offset:isize => Jie(r, offset),
        "jio " r:reg ", " offset:isize => Jio(r, offset),
    }));
    ```

Lines and sections:

*   <code>line(<var>pattern</var>)</code> - Matches a single line of text that
    matches *pattern*, and the newline at the end of the line.

    This is like <code>^<var>pattern</var>\n</code> in regular expressions,
    with two minor differences:

    -   <code>line(<var>pattern</var>)</code> will only ever match exactly
        one line of text, even if *pattern* could match more newlines.

    -   If your input does not end with a newline,
        <code>line(<var<pattern</var>)</code> can still match the
        non-newline-terminated "line" at the end.

    `line(string(any_char+))` matches a line of text, strips off the newline
    character, and returns the rest as a `String`.

    `line("")` matches a blank line.

*   <code>lines(<var>pattern</var>)</code> - Matches any number of lines of
    text matching *pattern*. Each line must be terminated by a newline, `'\n'`.

    Equivalent to <code>line(<var>pattern</var>)*</code>.

    ```
    let p = parser!(lines(repeat_sep(digit, " ")));
    assert_eq!(
        p.parse("1 2 3\n4 5 6\n").unwrap(),
        vec![vec![1, 2, 3], vec![4, 5, 6]],
    );
    ```

*   <code>section(<var>pattern</var>)</code> - Matches zero or more nonblank lines,
    followed by either a blank line or the end of input. The nonblank lines must match
    *pattern*.

    `section()` consumes the blank line. *pattern* should not expect to see it.

    It's common for an AoC puzzle input to have several lines of data, then
    a blank line, and then a different kind of data. You can parse this with
    <code>section(<var>p1</var>) section(<var>p2</var>)</code>.

    `section(lines(u64))` matches a section that's a list of numbers, one per line.

*   <code>sections(<var>pattern</var>)</code> - Matches any number of sections
    matching *pattern*. Equivalent to <code>section(<var>pattern</var>)*</code>

    Bringing it all together to parse a complex example:

    ```
    let example = "\
    Wiring Diagram #1:
    a->q->E->z->J
    D->f->D

    Wiring Diagram #2:
    g->r->f
    g->B
    ";

    let p = parser!(sections(
        line("Wiring Diagram #" usize ":")
        lines(repeat_sep(alpha, "->"))
    ));
    assert_eq!(
        p.parse(example).unwrap(),
        vec![
            (1, vec![vec!['a', 'q', 'E', 'z', 'J'], vec!['D', 'f', 'D']]),
            (2, vec![vec!['g', 'r', 'f'], vec!['g', 'B']]),
        ],
    );
    ```

[AoC]: https://adventofcode.com/
[example]: https://adventofcode.com/2015/day/2
[aoc-runner]: https://lib.rs/crates/aoc-runner

License: MIT
