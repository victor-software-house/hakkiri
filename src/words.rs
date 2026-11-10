//! Words: the pieces of a text that a count of words counts.

use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

/// The word pattern of the `prose` module Vale counts with.
///
/// `\p{L}` is the general category of letters, so a combining mark ends a
/// word, as it does in `prose`.
#[expect(
    clippy::expect_used,
    reason = "The pattern is a constant that compiles."
)]
static WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:[A-Z]\.){2,}|[\p{N}\p{L}']+").expect("the word pattern compiles")
});

/// The byte range of each word in `text`.
///
/// A word is a run of letters, digits and apostrophes, or initials such as
/// `U.S.`: two or more capital letters, each followed by a period.
/// The pattern is `(?:[A-Z]\.){2,}|[\p{N}\p{L}']+`, the one `prose` uses.
#[must_use]
pub fn words(text: &str) -> Vec<Range<usize>> {
    WORD.find_iter(text).map(|found| found.range()).collect()
}
