use std::{num::ParseIntError, str::FromStr};

use lazy_static::lazy_static;
use num_bigint::{BigInt, BigUint, ParseBigIntError};
use num_traits::Num;
use regex::Regex;

use crate::{parsers::regex::RegexParser, ParseContext, ParseIter, Reported};

/// A trivial ParseIter that presents exactly one match and holds a
/// pre-converted value.
///
/// Some parsers, like `u64`, do all the work of conversion as part of
/// confirming that a match is valid. Rather than do the work again during the
/// `convert` phase, the answer is stored in this iterator.
pub struct BasicParseIter<T> {
    pub(crate) end: usize,
    pub(crate) value: T,
}

impl<'parse, T> ParseIter<'parse> for BasicParseIter<T>
where
    T: Clone,
{
    type RawOutput = (T,);

    fn match_end(&self) -> usize {
        self.end
    }

    fn backtrack(&mut self, _context: &mut ParseContext<'parse>) -> Result<(), Reported> {
        Err(Reported)
    }

    fn convert(&self) -> (T,) {
        (self.value.clone(),)
    }
}

// --- Global regexes that are compiled on first use

macro_rules! regexes {
    ( $( $name:ident = $re:expr ; )* ) => {
        $(
            pub(crate) fn $name() -> &'static Regex {
                lazy_static! {
                    static ref RE: Regex = Regex::new($re).unwrap();
                }
                &RE
            }
        )*
    }
}

regexes! {
    uint_regex = r"\A[0-9]+";
    int_regex = r"\A[+-]?[0-9]+";
    bool_regex = r"\A(?:true|false)";
    uint_bin_regex = r"\A[01]+";
    int_bin_regex = r"\A[+-]?[01]+";
    uint_hex_regex = r"\A[0-9A-Fa-f]+";
    int_hex_regex = r"\A[+-]?[0-9A-Fa-f]+";
}

// --- Parsers that use FromStr

macro_rules! from_str_parse_impl {
        ( $( $ty:ident )+ , $re_name:ident) => {
            $(
                /// Parse a value of a primitive type (using its `FromStr`
                /// implementation in the Rust standard library).
                #[allow(non_upper_case_globals)]
                pub const $ty: RegexParser<$ty, <$ty as FromStr>::Err> =
                    RegexParser {
                        regex: $re_name,
                        parse_fn: <$ty as FromStr>::from_str,
                    };
            )+
        };
    }

from_str_parse_impl!(u8 u16 u32 u64 u128 usize, uint_regex);
from_str_parse_impl!(i8 i16 i32 i64 i128 isize, int_regex);
from_str_parse_impl!(bool, bool_regex);

/// Parse a BigUint (using its `FromStr` implementation in the `num-bigint`
/// crate, except that underscores between digits are not accepted and a
/// leading `+` sign is not accepted).
#[allow(non_upper_case_globals)]
pub const big_uint: RegexParser<BigUint, <BigUint as FromStr>::Err> = RegexParser {
    regex: uint_regex,
    parse_fn: <BigUint as FromStr>::from_str,
};

/// Parse a BigInt (using its `FromStr` implementation in the `num-bigint`
/// crate, except that underscores between digits are not accepted).
#[allow(non_upper_case_globals)]
pub const big_int: RegexParser<BigInt, <BigInt as FromStr>::Err> = RegexParser {
    regex: int_regex,
    parse_fn: <BigInt as FromStr>::from_str,
};

// --- Parsers for `_bin` and `_hex` integers

macro_rules! from_str_radix_parsers {
    ( $( ( $ty:ident , $bin:ident , $hex:ident ) ),* ; $bin_re:ident, $hex_re:ident ) => {
        $(
            /// Parse an integer written in base 2, using the `from_str_radix`
            /// static method from the Rust standard library.
            #[allow(non_upper_case_globals)]
            pub const $bin: RegexParser<$ty, ParseIntError> = RegexParser {
                regex: $bin_re,
                parse_fn: |s| $ty::from_str_radix(s, 2),
            };

            /// Parse an integer written in base 16, using the `from_str_radix`
            /// static method from the Rust standard library.
            #[allow(non_upper_case_globals)]
            pub const $hex: RegexParser<$ty, ParseIntError> = RegexParser {
                regex: $hex_re,
                parse_fn: |s| $ty::from_str_radix(s, 16),
            };
        )*
    }
}

from_str_radix_parsers!(
    (u8, u8_bin, u8_hex),
    (u16, u16_bin, u16_hex),
    (u32, u32_bin, u32_hex),
    (u64, u64_bin, u64_hex),
    (u128, u128_bin, u128_hex),
    (usize, usize_bin, usize_hex);
    uint_bin_regex,
    uint_hex_regex
);

from_str_radix_parsers!(
    (i8, i8_bin, i8_hex),
    (i16, i16_bin, i16_hex),
    (i32, i32_bin, i32_hex),
    (i64, i64_bin, i64_hex),
    (i128, i128_bin, i128_hex),
    (isize, isize_bin, isize_hex);
    int_bin_regex,
    int_hex_regex
);

/// Parse a [`BigUint`] written in base 2 (using its [`Num`] impl from the
/// `num-bigint` crate, except that underscores between digits are not
/// accepted and a leading `+` sign is not accepted).
#[allow(non_upper_case_globals)]
pub const big_uint_bin: RegexParser<BigUint, ParseBigIntError> = RegexParser {
    regex: uint_bin_regex,
    parse_fn: |s| BigUint::from_str_radix(s, 2),
};

/// Parse a [`BigUint`] written in base 16 (using its [`Num`] impl from the
/// `num-bigint` crate, except that underscores between digits are not
/// accepted and a leading `+` sign is not accepted).
#[allow(non_upper_case_globals)]
pub const big_uint_hex: RegexParser<BigUint, ParseBigIntError> = RegexParser {
    regex: uint_hex_regex,
    parse_fn: |s| BigUint::from_str_radix(s, 16),
};

/// Parse a [`BigInt`] written in base 2 (using its [`Num`] impl from the
/// `num-bigint` crate, except that underscores between digits are not
/// accepted).
#[allow(non_upper_case_globals)]
pub const big_int_bin: RegexParser<BigInt, ParseBigIntError> = RegexParser {
    regex: int_bin_regex,
    parse_fn: |s| BigInt::from_str_radix(s, 2),
};

/// Parse a [`BigInt`] written in base 16 (using its [`Num`] impl from the
/// `num-bigint` crate, except that underscores between digits are not
/// accepted).
#[allow(non_upper_case_globals)]
pub const big_int_hex: RegexParser<BigInt, ParseBigIntError> = RegexParser {
    regex: int_hex_regex,
    parse_fn: |s| BigInt::from_str_radix(s, 16),
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;

    #[test]
    fn test_bool() {
        assert_parse_eq(bool, "true", true);
        assert_parse_eq(bool, "false", false);
        assert_no_parse(bool, "t");
        assert_no_parse(bool, "");
        assert_no_parse(bool, " true");
        assert_no_parse(bool, "false ");
    }

    #[test]
    fn test_parse_hex() {
        assert_no_parse(&i32_hex, "+");
        assert_no_parse(&i32_hex, "-");
        assert_no_parse(&i32_hex, "+ 4");
        assert_no_parse(&i32_hex, "+ 4");
        assert_parse_eq(&i32_hex, "7BCDEF01", 0x7bcdef01);
        assert_parse_eq(&i32_hex, "7fffffff", i32::MAX);
        assert_no_parse(&i32_hex, "80000000");
        assert_parse_eq(&i32_hex, "-80000000", i32::MIN);
        assert_no_parse(&i32_hex, "-80000001");

        let p = sequence(i32_hex, i32_hex);
        assert_no_parse(&p, "12");
        assert_no_parse(&p, "01230123ABCDABCD");
        assert_parse_eq(&p, "-1+1", (-1, 1));

        assert_no_parse(&u32_hex, "-1");
        assert_no_parse(&u32_hex, "+d3d32e2e");
        assert_parse_eq(&u32_hex, "ffffffff", u32::MAX);
        assert_parse_eq(&u32_hex, "ffffffff", u32::MAX);
        assert_parse_eq(
            &u32_hex,
            "0000000000000000000000000000000000000000000000000000000000000000ffffffff",
            u32::MAX,
        );
    }

    #[test]
    fn test_bigint() {
        assert_no_parse(big_uint, "");
        assert_no_parse(big_uint, "+");
        assert_no_parse(big_uint, "+11");
        assert_no_parse(big_uint, "-");
        assert_parse_eq(big_uint, "0", BigUint::default());
        assert_parse_eq(
            big_uint,
            "982371952794802135871309821709317509287109324809324983409383209484381293480",
            "982371952794802135871309821709317509287109324809324983409383209484381293480"
                .parse::<BigUint>()
                .unwrap(),
        );

        assert_no_parse(big_int, "");
        assert_no_parse(big_int, "+");
        assert_no_parse(big_int, "-");
        assert_parse_eq(big_int, "-0", BigInt::default());
        assert_parse_eq(big_int, "+0", BigInt::default());
        assert_parse_eq(big_int, "00", BigInt::default());
        assert_no_parse(big_int, "-+31");
        assert_no_parse(big_int, " 0");
        assert_parse_eq(
            big_int,
            "-4819487135612398473187093223859843207984094321710984370927309128460723598212",
            "-4819487135612398473187093223859843207984094321710984370927309128460723598212".parse::<BigInt>().unwrap(),
        );

        assert_parse_eq(
            big_uint_hex,
            "0000000000000000000000000000000000000000000000000000000000000000ffffffff",
            BigUint::from(u32::MAX),
        );
        assert_no_parse(big_uint_hex, "13a4g3");
        assert_no_parse(big_uint_bin, "1001012");
        assert_no_parse(big_int_hex, "13A4G3");
        assert_no_parse(big_int_bin, "1001012");
    }
}
