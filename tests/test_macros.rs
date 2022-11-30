use std::fmt::Debug;

use aoc_parse::{parser, prelude::*};

#[track_caller]
fn assert_parse<'s, P>(parser: &'s P, s: &'s str)
where
    P: Parser<'s, 's>,
{
    if let Err(err) = parser.parse(s) {
        panic!("parse failed: {}", err);
    }
}

#[track_caller]
fn assert_parse_eq<'s, P, E>(parser: &'s P, s: &'s str, expected: E)
where
    P: Parser<'s, 's>,
    P::Output: PartialEq<E> + Debug,
    E: Debug,
{
    match parser.parse(s) {
        Err(err) => panic!("parse failed: {}", err),
        Ok(val) => assert_eq!(val, expected),
    }
}

#[track_caller]
fn assert_no_parse<'s, P>(parser: &'s P, s: &'s str)
where
    P: Parser<'s, 's>,
    P::Output: Debug,
{
    if let Ok(m) = parser.parse(s) {
        panic!("expected no match, got: {:?}", m);
    }
}

#[test]
fn test_macros() {
    let p = parser!("hello " "world");
    assert_parse_eq(&p, "hello world", ());
    assert_no_parse(&p, "hello ");

    let p = parser!("hello " "strange "* "world");
    assert_parse(&p, "hello world");
    assert_parse(&p, "hello strange world");
    assert_parse(&p, "hello strange strange strange strange strange world");

    let p = parser!({"one", "two"});
    assert_parse(&p, "one");
    assert_parse(&p, "two");
    assert_no_parse(&p, "");
    assert_no_parse(&p, "onetwo");
    assert_no_parse(&p, "twoone");

    let p = parser!(lines("whee!"));
    assert_parse(&p, "");
    assert_parse(&p, "whee!\nwhee!\nwhee!\n");
    assert_parse(&p, "whee!\n");
    assert_no_parse(&p, "whee!");
    assert_no_parse(&p, "\n");
    assert_no_parse(&p, "whee!\n\n");

    let p = parser!(_a: "ok" => "OK");
    assert_parse_eq(&p, "ok", "OK");

    let p = parser!((_a: "hello") " " (_b: "world") => "!");
    assert_parse_eq(&p, "hello world", "!");
    assert_no_parse(&p, "");
    assert_no_parse(&p, "hello");
    assert_no_parse(&p, "hello ");
    assert_no_parse(&p, "helloworld");
    assert_no_parse(&p, " world");
    assert_no_parse(&p, "world");
    assert_no_parse(&p, "hello world ");

    let bit = parser!({ "0" => false, "1" => true });
    let p = parser!(bit*);
    assert_parse_eq(
        &p,
        "0010101",
        vec![false, false, true, false, true, false, true],
    );
}

#[test]
fn test_alpha() {
    use aoc_parse::{ParseError, ParseIter, Parser, Result};

    #[allow(non_camel_case_types)]
    struct alpha;
    enum AlphaIter<'source> {
        Before(&'source str, usize),
        Success(char),
        Error,
    }

    impl<'parse, 'source> Parser<'parse, 'source> for alpha {
        type Output = char;
        type RawOutput = (char,);
        type Iter = AlphaIter<'source>;
        fn parse_iter(&'parse self, source: &'source str, start: usize) -> AlphaIter<'source> {
            AlphaIter::Before(source, start)
        }
    }

    impl<'source> ParseIter for AlphaIter<'source> {
        type RawOutput = (char,);
        fn next_parse(&mut self) -> Option<Result<usize>> {
            if let AlphaIter::Before(source, start) = *self {
                match source[start..].chars().next() {
                    Some(c) if c.is_alphabetic() => {
                        *self = AlphaIter::Success(c);
                        Some(Ok(start + c.len_utf8()))
                    }
                    _ => {
                        *self = AlphaIter::Error;
                        Some(Err(ParseError::new_expected(source, start, "letter")))
                    }
                }
            } else {
                None
            }
        }

        fn take_data(&mut self) -> (char,) {
            match self {
                AlphaIter::Success(c) => (*c,),
                _ => panic!("invalid state"),
            }
        }
    }

    let p = parser!(a: alpha+ => a.into_iter().collect::<String>());
    assert_no_parse(&p, "");
    assert_no_parse(&p, " hello");
    assert_parse_eq(&p, "hello", "hello");
    assert_parse_eq(&p, "京", "京");
}

mod ad2021 {
    use aoc_parse::{parser, prelude::*};

    #[test]
    fn dec01() {
        let input = "199\n200\n208\n210\n";
        let p = parser!(lines(u64));
        assert_eq!(p.parse(input).unwrap(), vec![199, 200, 208, 210]);
    }

    #[test]
    fn dec02() {
        let input = "forward 5
down 5
forward 8
up 3
down 8
forward 2
";

        #[derive(Debug, PartialEq)]
        enum Command {
            Forward(u64),
            Down(u64),
            Up(u64),
        }

        let p = parser!(lines({
            "down " (n: u64) => Command::Down(n),
            "up " (n: u64) => Command::Up(n),
            "forward " (n: u64) => Command::Forward(n),
        }));

        use Command::*;
        assert_eq!(
            p.parse(input).unwrap(),
            vec![Forward(5), Down(5), Forward(8), Up(3), Down(8), Forward(2)],
        );
    }

    #[test]
    fn dec03() {
        let input = "00100
11110
10110
10111
10101
";
        let p = parser!(lines(u32_bin));
        assert_eq!(
            p.parse(input).unwrap(),
            vec![0b00100u32, 0b11110, 0b10110, 0b10111, 0b10101],
        );
    }
}
