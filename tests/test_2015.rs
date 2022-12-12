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
    let p = parser!(line({"(" => 1i32, ")" => -1}*));
    assert_parse_eq(p, ")())())\n", vec![-1i32, 1, -1, -1, 1, -1, -1]);
}

#[test]
fn day2() {
    let p = parser!(lines(u64 "x" u64 "x" u64));
    assert_parse_eq(p, "4x23x21\n22x29x19\n", vec![(4, 23, 21), (22, 29, 19)]);
}

#[test]
fn day3() {
    let p = parser!(line({
        "^" => (0, -1),
        "<" => (-1, 0),
        "v" => (0, 1),
        ">" => (1, 0),
    }*));

    assert_parse_eq(p, "^>v<\n", vec![(0, -1), (1, 0), (0, 1), (-1, 0)]);
}

#[test]
fn day4() {
    let p = parser!(line(lower+));
    assert_parse_eq(p, "xyzzy\n", vec!['x', 'y', 'z', 'z', 'y']);
}

#[test]
fn day5() {
    let p = parser!(lines(string(lower+)));
    assert_parse_eq(
        p,
        "ugknbfddg\njchzalr\nhaegwjz\ndvszwmarr\n",
        vec!["ugknbfddg", "jchzalr", "haegwjz", "dvszwmarr"],
    );
}

#[test]
fn day6() {
    let p = parser!(lines(
        "turn " {"on" => true, "off" => false} " "
            u32 "," u32 " through " u32 "," u32
    ));
    assert_parse_eq(
        p,
        "turn on 489,959 through 759,964\nturn off 820,516 through 871,914\nturn off 427,423 through 929,502\n",
        vec![(true, 489, 959, 759, 964), (false, 820, 516, 871, 914), (false, 427, 423, 929, 502)],
    );
}

#[test]
fn day7() {
    type Reg = String;

    #[derive(Debug, PartialEq)]
    enum Insn {
        Send(u32, Reg),
        And(Reg, Reg, Reg),
        Or(Reg, Reg, Reg),
        LShift(Reg, u32, Reg),
        RShift(Reg, u32, Reg),
        Not(Reg, Reg),
    }
    use Insn::*;

    let reg = parser!(string(lower+));
    let p = parser!(lines({
        a: u32 " -> " c: reg
            => Send(a, c),
        a: reg " AND " b: reg " -> " c: reg
            => And(a, b, c),
        a: reg " OR " b: reg " -> " c: reg
            => Or(a, b, c),
        a: reg " LSHIFT " b: u32 " -> " c: reg
            => LShift(a, b, c),
        a: reg " RSHIFT " b: u32 " -> " c: reg
            => RShift(a, b, c),
        "NOT " a: reg " -> " c: reg
            => Not(a, c),
    }));

    assert_parse_eq(
        p,
        "\
lf AND lq -> ls
iu RSHIFT 1 -> jn
bo OR bu -> bv
NOT el -> em
",
        vec![
            And("lf".to_string(), "lq".to_string(), "ls".to_string()),
            RShift("iu".to_string(), 1, "jn".to_string()),
            Or("bo".to_string(), "bu".to_string(), "bv".to_string()),
            Not("el".to_string(), "em".to_string()),
        ],
    );
}

#[test]
fn day8() {
    let p = parser!(lines(
        "\"" ({
            lower,
            "\\\\" => '\\',
            "\\\"" => '"',
            "\\x" h: digit_hex l: digit_hex
                => char::from_u32(16 * h as u32 + l as u32).unwrap(),
        })* "\""
    ));

    let example = r#""n\\c"
"\"pa\\x\x18od\\\\"
"x\"\xcaj\\xww"
"#;

    assert_parse_eq(
        p,
        example,
        vec![
            vec!['n', '\\', 'c'],
            vec!['"', 'p', 'a', '\\', 'x', '\u{18}', 'o', 'd', '\\', '\\'],
            vec!['x', '"', '\u{ca}', 'j', '\\', 'x', 'w', 'w'],
        ],
    );
}

#[test]
fn day9() {
    let p = parser!(lines(
        string(alpha+) " to " string(alpha+) " = " usize
    ));

    assert_parse_eq(
        p,
        "\
Tristram to Faerun = 108
AlphaCentauri to Snowdin = 4
Tambi to Norrath = 134
Straylight to Arbre = 127
",
        vec![
            ("Tristram".to_string(), "Faerun".to_string(), 108),
            ("AlphaCentauri".to_string(), "Snowdin".to_string(), 4),
            ("Tambi".to_string(), "Norrath".to_string(), 134),
            ("Straylight".to_string(), "Arbre".to_string(), 127),
        ],
    );
}

#[test]
fn day10() {
    let p = parser!(digit+);
    assert_parse_eq(p, "1113222113", vec![1, 1, 1, 3, 2, 2, 2, 1, 1, 3]);
}

#[test]
fn day11() {
    let p = parser!(alpha+);
    assert_parse_eq(p, "hxbxwxba", vec!['h', 'x', 'b', 'x', 'w', 'x', 'b', 'a']);
}

#[test]
fn day13() {
    let p = parser!(lines(
        string(alpha+) " would " { "gain" => 1, "lose" => -1 }
        " " u32 " happiness units by sitting next to " string(alpha+) "."
        ));

    assert_parse_eq(
        p,
        "\
Alice would gain 54 happiness units by sitting next to Bob.
Bob would gain 83 happiness units by sitting next to Alice.
David would lose 41 happiness units by sitting next to Carol.
",
        vec![
            ("Alice".to_string(), 1, 54, "Bob".to_string()),
            ("Bob".to_string(), 1, 83, "Alice".to_string()),
            ("David".to_string(), -1, 41, "Carol".to_string()),
        ],
    );
}

#[test]
fn day14() {
    let p = parser!(lines(
        string(alpha+) " can fly " u32 " km/s for "
            u32 " seconds, but then must rest for " u32 " seconds."
    ));

    assert_parse_eq(
        p,
        "\
Vixen can fly 8 km/s for 8 seconds, but then must rest for 53 seconds.
Blitzen can fly 13 km/s for 4 seconds, but then must rest for 49 seconds.
Rudolph can fly 20 km/s for 7 seconds, but then must rest for 132 seconds.
",
        vec![
            ("Vixen".to_string(), 8, 8, 53),
            ("Blitzen".to_string(), 13, 4, 49),
            ("Rudolph".to_string(), 20, 7, 132),
        ],
    );
}

#[test]
fn day15() {
    let p = parser!(lines(
        string(alpha+) ": "
            "capacity " i32 ", durability " i32 ", "
            "flavor " i32 ", texture " i32 ", "
            "calories " i32
    ));

    assert_parse_eq(
        p,
        "\
Sprinkles: capacity 5, durability -1, flavor 0, texture 0, calories 5
PeanutButter: capacity -1, durability 3, flavor 0, texture 0, calories 1
Frosting: capacity 0, durability -1, flavor 4, texture 0, calories 6
Sugar: capacity -1, durability 0, flavor 0, texture 2, calories 8
",
        vec![
            ("Sprinkles".to_string(), 5, -1, 0, 0, 5),
            ("PeanutButter".to_string(), -1, 3, 0, 0, 1),
            ("Frosting".to_string(), 0, -1, 4, 0, 6),
            ("Sugar".to_string(), -1, 0, 0, 2, 8),
        ],
    );
}

#[test]
fn day16() {
    let p = parser!(lines(
        "Sue " usize ": " repeat_sep(string(lower+) ": " usize, ", ")
    ));

    assert_parse_eq(
        p,
        "\
Sue 1: goldfish: 6, trees: 9, akitas: 0
Sue 2: goldfish: 7, trees: 1, akitas: 0
Sue 500: perfumes: 4, cars: 9, trees: 4
",
        vec![
            (
                1,
                vec![
                    ("goldfish".to_string(), 6),
                    ("trees".to_string(), 9),
                    ("akitas".to_string(), 0),
                ],
            ),
            (
                2,
                vec![
                    ("goldfish".to_string(), 7),
                    ("trees".to_string(), 1),
                    ("akitas".to_string(), 0),
                ],
            ),
            (
                500,
                vec![
                    ("perfumes".to_string(), 4),
                    ("cars".to_string(), 9),
                    ("trees".to_string(), 4),
                ],
            ),
        ],
    );
}

#[test]
fn day17() {
    let p = parser!(lines(u32));
    assert_parse_eq(p, "50\n44\n11\n49\n7\n18\n", vec![50, 44, 11, 49, 7, 18]);
}

#[test]
fn day18() {
    let p = parser!(lines({"." => false, "#" => true}+));
    assert_parse_eq(
        p,
        "\
.#.#.#
...##.
#.#..#
####..
",
        vec![
            vec![false, true, false, true, false, true],
            vec![false, false, false, true, true, false],
            vec![true, false, true, false, false, true],
            vec![true, true, true, true, false, false],
        ],
    );
}

#[test]
fn day19() {
    let p = parser!(
        section(lines(alpha+ " => " alpha+))
        section(line(alpha+))
    );
    assert_parse_eq(
        p,
        "\
Ca => SiRnFYFAr
F => CaF
F => PMg
H => NTh
O => CRnFYFAr
e => HF

CRnSiRnCaPTiMgYCaPTi
",
        (
            vec![
                (
                    vec!['C', 'a'],
                    vec!['S', 'i', 'R', 'n', 'F', 'Y', 'F', 'A', 'r'],
                ),
                (vec!['F'], vec!['C', 'a', 'F']),
                (vec!['F'], vec!['P', 'M', 'g']),
                (vec!['H'], vec!['N', 'T', 'h']),
                (vec!['O'], vec!['C', 'R', 'n', 'F', 'Y', 'F', 'A', 'r']),
                (vec!['e'], vec!['H', 'F']),
            ],
            vec![
                'C', 'R', 'n', 'S', 'i', 'R', 'n', 'C', 'a', 'P', 'T', 'i', 'M', 'g', 'Y', 'C',
                'a', 'P', 'T', 'i',
            ],
        ),
    );
}

#[test]
fn day21() {
    let p = parser!(
        line("Hit Points: " u32)
        line("Damage: " u32)
        line("Armor: " u32)
    );
    assert_parse_eq(
        p,
        "\
Hit Points: 104
Damage: 8
Armor: 1
",
        (104, 8, 1),
    );
}

#[test]
fn day22() {
    let p = parser!(
        line("Hit Points: " u32)
        line("Damage: " u32)
    );
    assert_parse_eq(
        p,
        "\
Hit Points: 55
Damage: 8
",
        (55, 8),
    );
}

#[test]
fn day23() {
    #[derive(Debug, Copy, Clone, PartialEq)]
    enum Reg {
        A,
        B,
    }
    use Reg::*;
    let reg = parser!({"a" => A, "b" => B});

    #[derive(Debug, Copy, Clone, PartialEq)]
    enum Insn {
        Hlf(Reg),
        Tpl(Reg),
        Inc(Reg),
        Jmp(isize),
        Jie(Reg, isize),
        Jio(Reg, isize),
    }
    use Insn::*;

    let p = parser!(lines({
        "hlf " r: reg => Hlf(r),
        "tpl " r: reg => Tpl(r),
        "inc " r: reg => Inc(r),
        "jmp " offset: isize => Jmp(offset),
        "jie " r: reg ", " offset: isize => Jie(r, offset),
        "jio " r: reg ", " offset: isize => Jio(r, offset),
    }));

    assert_parse_eq(
        p,
        "\
inc a
jio a, +2
tpl a
inc a
",
        vec![Inc(A), Jio(A, 2), Tpl(A), Inc(A)],
    );
}
