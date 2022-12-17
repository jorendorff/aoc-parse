use aoc_parse::{parser, prelude::*};

#[test]
fn day7() {
    let input = "\
light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.
";

    #[derive(Debug, PartialEq)]
    struct Bags {
        count: usize,
        color: String,
    }

    #[derive(Debug, PartialEq)]
    struct Rule {
        color: String,
        contents: Vec<Bags>,
    }

    // This tests backtracking across parser boundaries (`color` will initially
    // match too many words every time it's used), function calls inside
    // `string()`, and label scoping (there are two labels `color:`).
    let color = parser!(string(repeat_sep(alpha+, ' ')));
    let p = parser!(lines(
        color:color
        " bags contain "
        contents:{
            "no other bags" => vec![],
            repeat_sep(
                count:usize ' ' color:color ' ' {"bag", "bags"} => Bags { count, color },
                ", "
            )
        }
        '.'
            => Rule { color, contents}
    ));

    fn bags(count: usize, color: &str) -> Bags {
        let color = color.to_string();
        Bags { count, color }
    }

    assert_eq!(
        p.parse(input).unwrap(),
        vec![
            Rule {
                color: "light red".to_string(),
                contents: vec![bags(1, "bright white"), bags(2, "muted yellow")],
            },
            Rule {
                color: "dark orange".to_string(),
                contents: vec![bags(3, "bright white"), bags(4, "muted yellow")],
            },
            Rule {
                color: "bright white".to_string(),
                contents: vec![bags(1, "shiny gold")],
            },
            Rule {
                color: "muted yellow".to_string(),
                contents: vec![bags(2, "shiny gold"), bags(9, "faded blue")],
            },
            Rule {
                color: "shiny gold".to_string(),
                contents: vec![bags(1, "dark olive"), bags(2, "vibrant plum")],
            },
            Rule {
                color: "dark olive".to_string(),
                contents: vec![bags(3, "faded blue"), bags(4, "dotted black")],
            },
            Rule {
                color: "vibrant plum".to_string(),
                contents: vec![bags(5, "faded blue"), bags(6, "dotted black")],
            },
            Rule {
                color: "faded blue".to_string(),
                contents: vec![],
            },
            Rule {
                color: "dotted black".to_string(),
                contents: vec![],
            },
        ],
    );
}
