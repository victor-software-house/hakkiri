//! Words that no dictionary lists: numbers, hexadecimal and binary numbers,
//! hashes and numbers with units.
//!
//! A port of `words.go` in [Vale]'s `spell` package at `66bbb85`.
//!
//! [Vale]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell/words.go

use std::sync::LazyLock;

use regex::Regex;

/// Builds a pattern that is a constant.
#[expect(
    clippy::expect_used,
    reason = "Each pattern below is a constant that compiles."
)]
fn constant(pattern: &str) -> Regex {
    Regex::new(pattern).expect("the pattern compiles")
}

/// A number that may hold dots, commas and dashes.
static NUMBER: LazyLock<Regex> = LazyLock::new(|| constant("^([0-9]+[.,-]?)+$"));
/// A number with units, such as `123ms`.
static UNITS: LazyLock<Regex> = LazyLock::new(|| constant("^[0-9]+[a-zA-Z]+$"));
/// A hexadecimal number, such as `0x12FF`.
static HEX: LazyLock<Regex> = LazyLock::new(|| constant("^0?[x][0-9A-Fa-f]+$"));
/// A binary number, such as `0b101`.
static BINARY: LazyLock<Regex> = LazyLock::new(|| constant("^0[b][01]+$"));
/// A SHA-1 hash in lower case.
static HASH: LazyLock<Regex> = LazyLock::new(|| constant("^[0-9a-z]{40}$"));

/// Whether `word` is a number, in any script.
pub(super) fn is_number(word: &str) -> bool {
    NUMBER.is_match(word) || (!word.is_empty() && word.chars().all(char::is_numeric))
}

/// Whether `word` is a hexadecimal number.
pub(super) fn is_hex(word: &str) -> bool {
    HEX.is_match(word)
}

/// Whether `word` is a binary number.
pub(super) fn is_binary(word: &str) -> bool {
    BINARY.is_match(word)
}

/// Whether `word` is a hash.
pub(super) fn is_hash(word: &str) -> bool {
    HASH.is_match(word)
}

/// The units of a number with units, such as `ms` in `123ms`, or `None`.
pub(super) fn units(word: &str) -> Option<&str> {
    if !UNITS.is_match(word) {
        return None;
    }
    word.find(|c: char| !c.is_ascii_digit())
        .and_then(|at| word.get(at..))
}
