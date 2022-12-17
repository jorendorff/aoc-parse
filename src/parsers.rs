mod chars;
mod collections;
mod dynamic;
mod either;
mod empty;
mod exact;
mod lines;
mod map;
mod primitive;
mod regex;
mod repeat;
mod rule_set;
mod sequence;
mod string;

pub use self::regex::RegexParser;
pub use chars::{alnum, alpha, any_char, char_of, digit, digit_bin, digit_hex, lower, upper};
pub use collections::{btree_map, btree_set, hash_map, hash_set, vec_deque};
pub use either::{alt, either, opt, AltParser, Either, EitherParser};
pub use empty::{empty, EmptyParser};
pub use lines::{line, lines, section, sections, LineParser, SectionParser};
pub use map::{map, single_value, skip, MapParser};
pub use primitive::{
    big_int, big_int_bin, big_int_hex, big_uint, big_uint_bin, big_uint_hex, bool, f32, f64, i128,
    i128_bin, i128_hex, i16, i16_bin, i16_hex, i32, i32_bin, i32_hex, i64, i64_bin, i64_hex, i8,
    i8_bin, i8_hex, isize, isize_bin, isize_hex, u128, u128_bin, u128_hex, u16, u16_bin, u16_hex,
    u32, u32_bin, u32_hex, u64, u64_bin, u64_hex, u8, u8_bin, u8_hex, usize, usize_bin, usize_hex,
    BasicParseIter,
};
pub use repeat::{plus, repeat, repeat_sep, star, RepeatParser};
pub use rule_set::{RuleParser, RuleSetBuilder, RuleSetParser};
pub use sequence::{pair, sequence, SequenceParser};
pub use string::StringParser;

// --- Wrappers

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[test]
    fn test_parse() {
        let p = empty();
        assert_parse_eq(p, "", ());
        assert_no_parse(p, "x");

        let p = "ok";
        assert_parse_eq(p, "ok", ());
        assert_no_parse(p, "");
        assert_no_parse(p, "o");
        assert_no_parse(p, "nok");

        let p = sequence("ok", "go");
        assert_parse_eq(p, "okgo", ());
        assert_no_parse(p, "ok");
        assert_no_parse(p, "go");
        assert_no_parse(p, "");

        let p = either(empty(), "ok");
        assert_parse_eq(p, "", Either::Left(()));
        assert_parse_eq(p, "ok", Either::Right(()));
        assert_no_parse(p, "okc");
        assert_no_parse(p, "okok");

        assert_no_parse(&u8, "256");

        assert_parse_eq(&u8, "255", 255u8);
        assert_parse_eq(&sequence("#", u32), "#100", 100u32);
        assert_parse_eq(
            map(&sequence("forward ", u64), |a| a),
            "forward 1234",
            1234u64,
        );
    }
}
