use aoc_parse::{parser, prelude::*};

#[test]
fn day1() {
    let input = "199\n200\n208\n210\n";
    let p = parser!(lines(u64));
    assert_eq!(p.parse(input).unwrap(), vec![199, 200, 208, 210]);
}

#[test]
fn day2() {
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
        "down " n:u64 => Command::Down(n),
        "up " n:u64 => Command::Up(n),
        "forward " n:u64 => Command::Forward(n),
    }));

    use Command::*;
    assert_eq!(
        p.parse(input).unwrap(),
        vec![Forward(5), Down(5), Forward(8), Up(3), Down(8), Forward(2)],
    );
}

#[test]
fn day3() {
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
        vec![0b00100, 0b11110, 0b10110, 0b10111, 0b10101],
    );
}

#[test]
fn day5() {
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
            x1:usize ',' y1:usize " -> " x2:usize ',' y2:usize
                => Line { x1, y1, x2, y2 }
        )
    );
    assert_eq!(
        p.parse(input).unwrap(),
        vec![
            Line {
                x1: 0,
                y1: 9,
                x2: 5,
                y2: 9
            },
            Line {
                x1: 8,
                y1: 0,
                x2: 0,
                y2: 8
            },
            Line {
                x1: 9,
                y1: 4,
                x2: 3,
                y2: 4
            },
            Line {
                x1: 2,
                y1: 2,
                x2: 2,
                y2: 1
            },
        ],
    );
}

#[test]
fn day6() {
    let input = "3,4,3,1,2\n";
    let p = parser!(line(repeat_sep(usize, ',')));
    assert_eq!(p.parse(input).unwrap(), vec![3usize, 4, 3, 1, 2]);
}

#[test]
fn day7() {
    let input = "16,1,2,0,4,2,7,1,2,14\n";
    let p = parser!(line(repeat_sep(i64, ',')));
    assert_eq!(
        p.parse(input).unwrap(),
        vec![16, 1, 2, 0, 4, 2, 7, 1, 2, 14],
    );
}

#[test]
fn day8() {
    let input = "\
be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb | fdgacbe cefdb cefbgd gcbe
edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec | fcgedb cgb dgebacf gc
fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef | cg cg fdcagb cbg
fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega | efabcd cedba gadfec cb
";
    let segment = parser!(
        s:char_of("abcdefg")+
            => s.into_iter().map(|i| 1u8 << i).sum::<u8>()
    );
    let p = parser!(lines(
        repeat_sep(segment, " ") " | " repeat_sep(segment, " ")
    ));

    assert_eq!(
        p.parse(input).unwrap()[2],
        (
            vec![
                0b1111011, 0b1000100, 0b0011111, 0b1101011, 0b1101111, 0b1111110, 0b1001111,
                0b1100101, 0b1000110, 0b1111111
            ],
            vec![0b1000100, 0b1000100, 0b1101111, 0b1000110],
        ),
    );
}
