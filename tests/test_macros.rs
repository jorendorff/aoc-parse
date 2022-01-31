use aoc_parse::parser;
use aoc_parse::Parser;

#[track_caller]
fn assert_parse(parser: &Parser, s: &str) {
    if let Err(err) = parser.parse(s) {
        panic!("parse failed: {}", err);
    }
}

#[track_caller]
fn assert_no_parse(parser: &Parser, s: &str) {
    if let Ok(m) = parser.parse(s) {
        panic!("expected no match, got: {:?}", m);
    }
}

#[test]
fn test_macros() {
    let p = parser!("hello " "world");
    assert_parse(&p, "hello world");
    assert_no_parse(&p, "hello ");

    let p = parser!("hello " "strange "* "world");
    assert_parse(&p, "hello world");
    assert_parse(&p, "hello strange world");
    assert_parse(&p, "hello strange strange strange strange strange world");
}
