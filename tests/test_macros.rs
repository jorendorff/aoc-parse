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

mod names_and_scopes {
    use super::assert_parse_eq;

    // `=>` should work even if `Parser` has not been imported.
    #[test]
    fn test_map_syntax() {
        use aoc_parse::{parser, prelude::u64};

        let p = parser!((a: u64) " " (b: u64) => 100 * a + b);

        assert_parse_eq(&p, "31 34", 3134);
    }
}

#[test]
fn test_chars() {
    let p = parser!(a: alpha+ => a.into_iter().collect::<String>());
    assert_no_parse(&p, "");
    assert_no_parse(&p, " hello");
    assert_parse_eq(&p, "hello", "hello");
    assert_parse_eq(&p, "京", "京");

    let cls = parser!((upper lower*)+);
    assert_parse_eq(
        &cls,
        "EntityManagerFactory",
        vec![
            ('E', vec!['n', 't', 'i', 't', 'y']),
            ('M', vec!['a', 'n', 'a', 'g', 'e', 'r']),
            ('F', vec!['a', 'c', 't', 'o', 'r', 'y']),
        ],
    );

    assert_parse_eq(
        &parser!(string((upper lower*)+)),
        "EntityManagerFactory",
        "EntityManagerFactory".to_string(),
    );

    let p = parser!(lines(digit+));
    assert_parse_eq(&p, "0\n", vec![vec![0]]);
    assert_no_parse(&p, "14a0\n");
    assert_parse_eq(
        &p,
        "1482\n3271\n5390\n",
        vec![vec![1, 4, 8, 2], vec![3, 2, 7, 1], vec![5, 3, 9, 0]],
    );
}

#[test]
fn test_backtracking() {
    assert_parse_eq(
        &parser!(lines(digit_bin+) line(digit_bin+) line(digit_bin+)),
        "01101\n10110\n01010\n00001\n",
        (
            vec![vec![0, 1, 1, 0, 1], vec![1, 0, 1, 1, 0]],
            vec![0, 1, 0, 1, 0],
            vec![0, 0, 0, 0, 1],
        ),
    );
}

#[test]
fn test_root() {
    // At one point, the string below would parse to `(1, 4, 3, 5)`. This was a
    // bug; the RawOutput type of `fraction` should be a singleton tuple.

    let fraction = parser!(i64 "/" u64);
    let range = parser!(fraction ".." fraction);

    assert_parse_eq(&range, "1/4..3/5", ((1, 4), (3, 5)));
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
        let input = "\
forward 5
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
        let input = "\
00100
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

    #[test]
    fn dec05() {
        let input = "\
0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
";

        #[derive(Debug, PartialEq)]
        struct Line {
            x1: usize,
            y1: usize,
            x2: usize,
            y2: usize,
        }

        let p = parser!(
            lines(
                (x1: usize) "," (y1: usize) " -> " (x2: usize) "," (y2: usize)
                    => Line { x1, y1, x2, y2 }
            )
        );
        assert_eq!(
            p.parse(input).unwrap(),
            vec![
                Line { x1: 0, y1: 9, x2: 5, y2: 9 },
                Line { x1: 8, y1: 0, x2: 0, y2: 8 },
                Line { x1: 9, y1: 4, x2: 3, y2: 4 },
                Line { x1: 2, y1: 2, x2: 2, y2: 1 },
            ],
        );
    }

    #[test]
    fn dec06() {
        let input = "3,4,3,1,2\n";
        let p = parser!(repeat_sep(usize, ",") "\n");
        assert_eq!(p.parse(input).unwrap(), vec![3usize, 4, 3, 1, 2]);
    }

    #[test]
    fn dec07() {
        let input = "16,1,2,0,4,2,7,1,2,14\n";
        let p = parser!(repeat_sep(i64, ",") "\n");
        assert_eq!(
            p.parse(input).unwrap(),
            vec![16, 1, 2, 0, 4, 2, 7, 1, 2, 14],
        );
    }
}
