use std::fmt::Debug;

use aoc_parse::{parser, prelude::*};

#[test]
fn day21() {
    let input = "\
../.# => ##./#../...
.#./..#/### => #..#/..../..../#..#
";

    type Grid = Vec<Vec<bool>>;

    #[derive(Debug, PartialEq, Eq)]
    enum EnhancementRule {
        ByTwos(Grid, Grid),
        ByThrees(Grid, Grid),
    }

    let p = parser!(
        rule bit: bool = { '#' => true, '.' => false };
        rule g2: Grid = repeat_sep_n(repeat_n(bit, 2), '/', 2);
        rule g3: Grid = repeat_sep_n(repeat_n(bit, 3), '/', 3);
        rule g4: Grid = repeat_sep_n(repeat_n(bit, 4), '/', 4);
        lines({
            input:g2 " => " output:g3 => EnhancementRule::ByTwos(input, output),
            input:g3 " => " output:g4 => EnhancementRule::ByThrees(input, output),
        })
    );

    assert_eq!(
        p.parse(input).unwrap(),
        vec![
            EnhancementRule::ByTwos(
                vec![vec![false, false], vec![false, true]],
                vec![
                    vec![true, true, false],
                    vec![true, false, false],
                    vec![false, false, false],
                ]
            ),
            EnhancementRule::ByThrees(
                vec![
                    vec![false, true, false],
                    vec![false, false, true],
                    vec![true, true, true],
                ],
                vec![
                    vec![true, false, false, true],
                    vec![false, false, false, false],
                    vec![false, false, false, false],
                    vec![true, false, false, true],
                ]
            )
        ],
    );
}
