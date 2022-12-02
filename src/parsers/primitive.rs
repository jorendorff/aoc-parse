use std::{num::ParseIntError, str::FromStr};

use lazy_static::lazy_static;
use regex::Regex;

use crate::parsers::regex::RegexParser;

// --- Default parsers for some types that implement FromStr

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
    uint_regex = r"\A(0|[1-9][0-9]*)";
    int_regex = r"\A(?:0|[+-]?[1-9][0-9]*)";
    bool_regex = r"true|false";
    uint_bin_regex = r"\A[01]+";
    int_bin_regex = r"\A[+-]?[01]+";
}

macro_rules! from_str_parse_impl {
        ( $( $ty:ident )+ , $re_name:ident) => {
            $(
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

macro_rules! from_str_radix_parsers {
        ( $( ( $ty:ident , $bin:ident , $hex:ident ) ),* : $re_name:ident ) => {
            $(
                #[allow(non_upper_case_globals)]
                pub const $bin: RegexParser<$ty, ParseIntError> = RegexParser {
                    regex: $re_name,
                    parse_fn: |s| $ty::from_str_radix(s, 2),
                };

                #[allow(non_upper_case_globals)]
                pub const $hex: RegexParser<$ty, ParseIntError> = RegexParser {
                    regex: $re_name,
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
    (usize, usize_bin, usize_hex): uint_bin_regex
);

from_str_radix_parsers!(
    (i8, i8_bin, i8_hex),
    (i16, i16_bin, i16_hex),
    (i32, i32_bin, i32_hex),
    (i64, i64_bin, i64_hex),
    (i128, i128_bin, i128_hex),
    (isize, isize_bin, isize_hex): int_bin_regex
);
