use aoc_parse::{parser, prelude::*};

#[test]
fn day2() {
    let input = "\
A Y
B X
C Z
";

    #[derive(Debug, PartialEq)]
    enum Move {
        Rock,
        Paper,
        Scissors,
    }

    #[derive(Debug, PartialEq)]
    enum Goal {
        Win,
        Lose,
        Draw,
    }

    use Goal::*;
    use Move::*;
    let p = parser!(lines(
        {"A" => Rock, "B" => Paper, "C" => Scissors}
        " "
        {"X" => Lose, "Y" => Draw, "Z" => Win}
    ));

    assert_eq!(
        p.parse(input).unwrap(),
        vec![(Rock, Draw), (Paper, Lose), (Scissors, Win)]
    );
}

#[test]
fn day3() {
    let input = "\
vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw
";

    fn priority(item: char) -> u32 {
        match item {
            'a'..='z' => 1 + (item as u32 - 'a' as u32),
            'A'..='Z' => 27 + (item as u32 - 'A' as u32),
            _ => panic!("invalid item {item:?}"),
        }
    }

    fn items(s: &[char]) -> u64 {
        s.iter()
            .copied()
            .map(priority)
            .map(|p| 1u64 << p)
            .fold(0, |a, b| (a | b))
    }

    let p = parser!(lines(
        chars:alpha+ => {
            let n = chars.len();
            assert_eq!(n % 2, 0, "line {:?} has an odd number of characters",
                       chars.into_iter().collect::<String>());
            (items(&chars[..n / 2]), items(&chars[n / 2..]))
        }
    ));

    assert_eq!(
        p.parse(input).unwrap(),
        vec![
            (0x0002_0010_00d5_0080, 0x0000_0081_0009_0148),
            (0x0000_1146_4402_0400, 0x0010_20c1_000c_0040),
            (0x0001_0400_0406_2010, 0x0002_4400_10c0_0080),
            (0x0010_00c4_00c2_0100, 0x0000_6801_1040_440c),
            (0x0000_1012_0010_0080, 0x0010_4800_0010_0008),
            (0x0010_0412_240c_0000, 0x0000_00c0_4089_2000),
        ],
    );
}

#[test]
fn day4() {
    let input = "\
2-4,6-8
2-3,4-5
5-7,7-9
";

    let range = parser!(a:u64 "-" b:u64 => a..(b + 1));
    let p = parser!(lines(range "," range));
    assert_eq!(
        p.parse(input).unwrap(),
        vec![(2..5, 6..9), (2..4, 4..6), (5..8, 7..10),],
    );
}

#[test]
fn day5() {
    let input = "    [D]    
[N] [C]    
[Z] [M] [P]
 1   2   3 

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2
";

    #[derive(Debug, PartialEq)]
    struct Input {
        model: Model,
        moves: Vec<Move>,
    }

    #[derive(Debug, PartialEq)]
    struct Model {
        stacks: Vec<Vec<char>>,
    }

    #[derive(Debug, PartialEq)]
    struct Move {
        quantity: usize,
        source: usize,
        target: usize,
    }

    let move_parser = parser!(
        "move " quantity:usize " from " source:usize " to " target:usize
            =>
            Move { quantity, source: source - 1, target: target - 1 }
    );

    let model = parser!(
        rows:lines(
            repeat_sep(
                {
                    "   " => None,
                    "[" x:alpha "]" => Some(x),
                },
                " "
            )
        )
        =>
        {
            let mut stacks = vec![];
            stacks.resize(rows[0].len(), vec![]);
            for row in rows.iter().rev() {
                for (i, c) in row.iter().copied().enumerate() {
                    if let Some(c) = c {
                        stacks[i].push(c);
                    }
                }
            }
            Model { stacks }
        }
    );

    let p = parser!(
        m:model
        line(repeat_sep(" " digit " ", " "))
        line("")
        moves:lines(move_parser)
        =>
        {
            Input { model: m, moves }
        }
    );

    assert_eq!(
        p.parse(input).unwrap(),
        Input {
            model: Model {
                stacks: vec![vec!['Z', 'N'], vec!['M', 'C', 'D'], vec!['P'],]
            },
            moves: vec![
                Move {
                    quantity: 1,
                    source: 1,
                    target: 0
                },
                Move {
                    quantity: 3,
                    source: 0,
                    target: 2
                },
                Move {
                    quantity: 2,
                    source: 1,
                    target: 0
                },
                Move {
                    quantity: 1,
                    source: 0,
                    target: 1
                },
            ],
        }
    );
}

#[test]
fn day7() {
    let input = "\
$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k
";

    #[derive(Debug, PartialEq)]
    enum Listing {
        LsDir(String),
        LsFile(String, u64),
    }

    #[derive(Debug, PartialEq)]
    enum Command {
        CdRoot,
        CdUp,
        Cd(String),
        Ls(Vec<Listing>),
    }

    use Command::*;
    use Listing::*;
    let p = parser!({
        line("$ cd /") => CdRoot,
        line("$ cd ..") => CdUp,
        d: line("$ cd " string(any_char+)) => Cd(d),
        line("$ ls") output:lines({
            size:u64 " " name:string(any_char+) => LsFile(name, size),
            "dir " name:string(any_char+) => LsDir(name),
        }) => Ls(output),
    }*);

    assert_eq!(
        p.parse(input).unwrap(),
        vec![
            CdRoot,
            Ls(vec![
                LsDir("a".to_string()),
                LsFile("b.txt".to_string(), 14848514),
                LsFile("c.dat".to_string(), 8504156),
                LsDir("d".to_string()),
            ]),
            Cd("a".to_string()),
            Ls(vec![
                LsDir("e".to_string()),
                LsFile("f".to_string(), 29116),
                LsFile("g".to_string(), 2557),
                LsFile("h.lst".to_string(), 62596),
            ]),
            Cd("e".to_string()),
            Ls(vec![LsFile("i".to_string(), 584)]),
            CdUp,
            CdUp,
            Cd("d".to_string()),
            Ls(vec![
                LsFile("j".to_string(), 4060174),
                LsFile("d.log".to_string(), 8033020),
                LsFile("d.ext".to_string(), 5626152),
                LsFile("k".to_string(), 7214296),
            ]),
        ]
    );
}

#[test]
fn day8() {
    let input = "\
30373
25512
65332
33549
35390
";
    let p = parser!(lines((a:digit => a as i32)+));
    assert_eq!(
        p.parse(input).unwrap(),
        vec![
            vec![3, 0, 3, 7, 3],
            vec![2, 5, 5, 1, 2],
            vec![6, 5, 3, 3, 2],
            vec![3, 3, 5, 4, 9],
            vec![3, 5, 3, 9, 0],
        ],
    );
}

#[test]
fn day9() {
    let input = "\
R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2
";

    let p = parser!(lines(
        {
            "L" => (-1, 0),
            "R" => (1, 0),
            "U" => (0, -1),
            "D" => (0, 1),
        }
        " " usize
    ));

    const L: (i32, i32) = (-1, 0);
    const R: (i32, i32) = (1, 0);
    const U: (i32, i32) = (0, -1);
    const D: (i32, i32) = (0, 1);
    assert_eq!(
        p.parse(input).unwrap(),
        vec![
            (R, 4),
            (U, 4),
            (L, 3),
            (D, 1),
            (R, 4),
            (D, 1),
            (L, 5),
            (R, 2),
        ],
    );
}

#[test]
fn day11() {
    let input = "\
Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1
";

    #[derive(Debug, PartialEq, Copy, Clone)]
    enum Op {
        Add,
        Mul,
    }

    #[derive(Debug, PartialEq, Copy, Clone)]
    enum Operand {
        Old,
        Lit(u64),
    }

    #[derive(Debug, PartialEq)]
    struct Monkey {
        i: usize,
        items: Vec<u64>,
        oper: (Operand, Op, Operand),
        test: u64,
        if_true: usize,
        if_false: usize,
    }

    let op = parser!({'+' => Op::Add, '*' => Op::Mul});

    let operand = parser!({
        "old" => Operand::Old,
        x:u64 => Operand::Lit(x),
    });

    let p = parser!(sections(
        i:        line("Monkey " usize ":")
        items:    line("  Starting items: " repeat_sep(u64, ", "))
        oper:     line("  Operation: new = " operand ' ' op ' ' operand)
        test:     line("  Test: divisible by " u64)
        if_true:  line("    If true: throw to monkey " usize)
        if_false: line("    If false: throw to monkey " usize)
            => Monkey { i, items, oper, test, if_true, if_false }
    ));

    //            => Monkey { i, items, oper, test, if_true, if_false }

    use Op::*;
    use Operand::*;
    assert_eq!(
        p.parse(input).unwrap(),
        vec![
            Monkey {
                i: 0,
                items: vec![79, 98],
                oper: (Old, Mul, Lit(19)),
                test: 23,
                if_true: 2,
                if_false: 3
            },
            Monkey {
                i: 1,
                items: vec![54, 65, 75, 74],
                oper: (Old, Add, Lit(6)),
                test: 19,
                if_true: 2,
                if_false: 0
            },
            Monkey {
                i: 2,
                items: vec![79, 60, 97],
                oper: (Old, Mul, Old),
                test: 13,
                if_true: 1,
                if_false: 3
            },
            Monkey {
                i: 3,
                items: vec![74],
                oper: (Old, Add, Lit(3)),
                test: 17,
                if_true: 0,
                if_false: 1
            },
        ],
    );
}

#[test]
fn day15() {
    let input = "\
Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
";

    let point = parser!("x=" i64 ", y=" i64);
    let p = parser!(lines(
        "Sensor at " point ": closest beacon is at " point
    ));

    assert_eq!(
        p.parse(input).unwrap(),
        vec![((2, 18), (-2, 15)), ((9, 16), (10, 16))],
    );
}

#[test]
fn day16() {
    let input = "\
Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II
";

    let p = parser!(lines(
        "Valve " n:string(alpha+) " has flow rate=" f:u64
            "; " {"tunnels lead to valves", "tunnel leads to valve"} " "
            t:repeat_sep(string(alpha+), ", ")
            => (n, f, t)
    ));

    assert_eq!(
        p.parse(input).unwrap(),
        vec![
            (
                "AA".to_string(),
                0,
                vec!["DD".to_string(), "II".to_string(), "BB".to_string()]
            ),
            (
                "BB".to_string(),
                13,
                vec!["CC".to_string(), "AA".to_string()]
            ),
            (
                "CC".to_string(),
                2,
                vec!["DD".to_string(), "BB".to_string()]
            ),
            (
                "DD".to_string(),
                20,
                vec!["CC".to_string(), "AA".to_string(), "EE".to_string()]
            ),
            (
                "EE".to_string(),
                3,
                vec!["FF".to_string(), "DD".to_string()]
            ),
            (
                "FF".to_string(),
                0,
                vec!["EE".to_string(), "GG".to_string()]
            ),
            (
                "GG".to_string(),
                0,
                vec!["FF".to_string(), "HH".to_string()]
            ),
            ("HH".to_string(), 22, vec!["GG".to_string()]),
            (
                "II".to_string(),
                0,
                vec!["AA".to_string(), "JJ".to_string()]
            ),
            ("JJ".to_string(), 21, vec!["II".to_string()]),
        ],
    );
}
