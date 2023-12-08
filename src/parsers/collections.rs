//! Parsers that produce collections other than `Vec`s.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::marker::PhantomData;

use crate::parsers::map::{MapParser, Mapping};
use crate::Parser;

#[derive(Debug, Clone, Copy, Default)]
pub struct Collect<C> {
    phantom: PhantomData<fn() -> C>,
}

impl<V, C> Mapping<(V,)> for Collect<C>
where
    V: IntoIterator,
    C: FromIterator<V::Item>,
{
    type RawOutput = (C,);

    fn apply(&self, (value,): (V,)) -> (C,) {
        (value.into_iter().collect(),)
    }
}

/// Convert the output of `parser` from a `Vec<(K, V)>` or other collection of pairs
/// into a `HashMap`.
///
/// For example, the parser `lines(alpha " = " u64)` produces a `Vec<(char, u64)>`.
/// Wrapping that parser in `hash_map` makes it produce a `HashMap<char, u64>` instead:
///
/// ```
/// # use aoc_parse::{parser, prelude::*};
/// let p = parser!(hash_map(lines(alpha " = " u64)));
///
/// let h = p.parse("X = 33\nY = 75\n").unwrap();
/// assert_eq!(h[&'X'], 33);
/// assert_eq!(h[&'Y'], 75);
/// ```
pub fn hash_map<P, K, V>(parser: P) -> MapParser<P, Collect<HashMap<K, V>>>
where
    P: Parser,
    P::Output: IntoIterator<Item = (K, V)>,
{
    MapParser {
        inner: parser,
        mapper: Collect::default(),
    }
}

/// Convert the output of `parser` from a `Vec<V>` or other collection
/// into a `HashSet`.
///
/// For example, the parser `alpha+` produces a `Vec<char>`,
/// so `hash_set(alpha+)` produces a `HashSet<char>`:
///
/// ```
/// # use aoc_parse::{parser, prelude::*};
/// let p = parser!(hash_set(alpha+));
///
/// let set = p.parse("xZjZZd").unwrap();
/// assert_eq!(set.len(), 4); // x Z j d
/// assert!(set.contains(&'d'));
/// assert!(!set.contains(&'r'));
/// ```
pub fn hash_set<P, V>(parser: P) -> MapParser<P, Collect<HashSet<V>>>
where
    P: Parser,
    P::Output: IntoIterator<Item = V>,
{
    MapParser {
        inner: parser,
        mapper: Collect::default(),
    }
}

/// Convert the output of `parser` from a `Vec<(K, V)>` or other collection of pairs
/// into a `BTreeMap`.
pub fn btree_map<P, K, V>(parser: P) -> MapParser<P, Collect<BTreeMap<K, V>>>
where
    P: Parser,
    P::Output: IntoIterator<Item = (K, V)>,
{
    MapParser {
        inner: parser,
        mapper: Collect::default(),
    }
}

/// Convert the output of `parser` from a `Vec<V>` or other collection
/// into a `BTreeSet`.
pub fn btree_set<P, V>(parser: P) -> MapParser<P, Collect<BTreeSet<V>>>
where
    P: Parser,
    P::Output: IntoIterator<Item = V>,
{
    MapParser {
        inner: parser,
        mapper: Collect::default(),
    }
}

/// Convert the output of `parser` from a `Vec<V>` or other collection
/// into a `VecDeque`.
pub fn vec_deque<P, V>(parser: P) -> MapParser<P, Collect<VecDeque<V>>>
where
    P: Parser,
    P::Output: IntoIterator<Item = V>,
{
    // NOTE: A mapping that uses `Into` might be faster, but I'm not sure. The standard library has
    // some specializations to do iterator-based colletion operations like this in-place.
    MapParser {
        inner: parser,
        mapper: Collect::default(),
    }
}
