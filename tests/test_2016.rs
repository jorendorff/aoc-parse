use std::fmt::Debug;

use aoc_parse::{parser, prelude::*};

#[track_caller]
fn assert_parse_eq<P, E>(parser: P, s: &str, expected: E)
where
    P: Parser,
    P::Output: PartialEq<E> + Debug,
    E: Debug,
{
    match parser.parse(s) {
        Err(err) => panic!("parse failed: {}", err),
        Ok(val) => assert_eq!(val, expected),
    }
}

#[test]
fn day1() {
    #[derive(Debug, PartialEq)]
    enum Dir {
        R,
        L,
    }
    use Dir::*;

    let p = parser!(repeat_sep({'R' => R, 'L' => L} u32, ", "));
    assert_parse_eq(p, "R2, L3", vec![(R, 2), (L, 3)]);
}

#[test]
fn day2() {
    #[derive(Debug, PartialEq)]
    enum Dir {
        R,
        U,
        L,
        D,
    }
    use Dir::*;

    let p = parser!(lines({'R' => R, 'U' => U, 'L' => L, 'D' => D}+));
    assert_parse_eq(
        p,
        "\
            ULL\n\
            RRDDD\n\
            LURDL\n\
            UUUUD\n\
        ",
        vec![
            vec![U, L, L],
            vec![R, R, D, D, D],
            vec![L, U, R, D, L],
            vec![U, U, U, U, D],
        ],
    );
}

#[test]
fn day3() {
    let p = parser!(lines(repeat_sep(u64, ' ')));
    assert_parse_eq(
        p,
        "\
            101 301 501\n\
            102 302 502\n\
            103 303 503\n\
            201 401 601\n\
            202 402 602\n\
            203 403 603\n\
        ",
        vec![
            vec![101, 301, 501],
            vec![102, 302, 502],
            vec![103, 303, 503],
            vec![201, 401, 601],
            vec![202, 402, 602],
            vec![203, 403, 603],
        ],
    );
}

#[test]
fn day4() {
    assert_parse_eq(
        parser!(lines(string(repeat_sep(lower+, '-')) '-' u32 '[' alpha+ ']')),
        "\
            aaaaa-bbb-z-y-x-123[abxyz]\n\
            a-b-c-d-e-f-g-h-987[abcde]\n\
            not-a-real-room-404[oarel]\n\
            totally-real-room-200[decoy]\n\
        ",
        vec![
            ("aaaaa-bbb-z-y-x".to_string(), 123, vec!['a', 'b', 'x', 'y', 'z']),
            ("a-b-c-d-e-f-g-h".to_string(), 987, vec!['a', 'b', 'c', 'd', 'e']),
            ("not-a-real-room".to_string(), 404, vec!['o', 'a', 'r', 'e', 'l']),
            ("totally-real-room".to_string(), 200, vec!['d', 'e', 'c', 'o', 'y']),
        ],
    );
}

#[test]
fn day6() {
    assert_parse_eq(
        parser!(lines(string(lower+))),
        "\
            ewqplnag\n\
            qchqvvsf\n\
            jdhaqbeu\n\
            jsgoijzv\n\
        ",
        vec![
            "ewqplnag".to_string(),
            "qchqvvsf".to_string(),
            "jdhaqbeu".to_string(),
            "jsgoijzv".to_string(),
        ],
    );
}

#[test]
fn day7() {
    #[derive(Debug, PartialEq, Clone)]
    enum Seq {
        Supernet(String),
        Hypernet(String),
    }

    assert_parse_eq(
        parser!(lines({
            (s: string(alpha+)) => Seq::Supernet(s),
            '[' (s: string(alpha+)) ']' => Seq::Hypernet(s),
        }+)),
        "\
            iungssgfnnjlgdferc[xfffplonmzjmxkinhl]dehxdielvncdawomqk[teizynepguvtgofr]fjazkxesmlwryphifh[ppjfvfefqhmuqtdp]luopramrehtriilwlou\n\
            mqxqhcpalwycdxw[fkwhjscfmgywhtvdb]khadwvhkxygtxqx\n\
            ihekzgbwpjxgbau[eqpvqxncntbtsqn]mbtbcujdkbrhxdu\n\
            izikobnovmjzngo[ombcpcvshnedtndu]lnnmdkuapgnxpgyxcmg[bgnxdzmiolfvvaizu]tcvnrfufuvhgmlxcm\n\
        ",
        vec![
            vec![
                Seq::Supernet("iungssgfnnjlgdferc".to_string()),
                Seq::Hypernet("xfffplonmzjmxkinhl".to_string()),
                Seq::Supernet("dehxdielvncdawomqk".to_string()),
                Seq::Hypernet("teizynepguvtgofr".to_string()),
                Seq::Supernet("fjazkxesmlwryphifh".to_string()),
                Seq::Hypernet("ppjfvfefqhmuqtdp".to_string()),
                Seq::Supernet("luopramrehtriilwlou".to_string()),
            ],
            vec![
                Seq::Supernet("mqxqhcpalwycdxw".to_string()),
                Seq::Hypernet("fkwhjscfmgywhtvdb".to_string()),
                Seq::Supernet("khadwvhkxygtxqx".to_string()),
            ],
            vec![
                Seq::Supernet("ihekzgbwpjxgbau".to_string()),
                Seq::Hypernet("eqpvqxncntbtsqn".to_string()),
                Seq::Supernet("mbtbcujdkbrhxdu".to_string()),
            ],
            vec![
                Seq::Supernet("izikobnovmjzngo".to_string()),
                Seq::Hypernet("ombcpcvshnedtndu".to_string()),
                Seq::Supernet("lnnmdkuapgnxpgyxcmg".to_string()),
                Seq::Hypernet("bgnxdzmiolfvvaizu".to_string()),
                Seq::Supernet("tcvnrfufuvhgmlxcm".to_string()),
            ],
        ],
    );
}

#[test]
fn day8() {
    #[derive(Debug, PartialEq, Clone)]
    enum Insn {
        Rect(usize, usize),
        RotateColumn(usize, usize),
        RotateRow(usize, usize),
    }

    assert_parse_eq(
        parser!(lines({
            "rect " (w: usize) 'x' (h: usize) => Insn::Rect(w, h),
            "rotate column x=" (x: usize) " by " (n: usize) => Insn::RotateColumn(x, n),
            "rotate row y=" (y: usize) " by " (n: usize) => Insn::RotateRow(y, n),
        })),
        "\
        rotate row y=0 by 8\n\
        rotate column x=0 by 1\n\
        rect 7x1\n\
        ",
        vec![
            Insn::RotateRow(0, 8),
            Insn::RotateColumn(0, 1),
            Insn::Rect(7, 1),
        ],
    );
}
