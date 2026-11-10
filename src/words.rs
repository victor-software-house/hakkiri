//! Words: the pieces of a text that a count of words counts.

use std::ops::Range;

/// The byte range of each word in `text`.
///
/// A word is a run of letters, digits and apostrophes, or initials such as
/// `U.S.`: two or more capital letters, each followed by a period.
/// This is the word pattern of the `prose` module Vale counts with,
/// `(?:[A-Z]\.){2,}|[\p{N}\p{L}']+`, read without a regular expression.
///
/// The cost is linear in the length of `text`.
#[must_use]
pub fn words(text: &str) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < text.len() {
        if let Some(end) = initials(text, at) {
            found.push(at..end);
            at = end;
            continue;
        }
        let rest = text.get(at..).unwrap_or_default();
        let length = rest
            .char_indices()
            .find(|&(_, c)| !is_word_character(c))
            .map_or(rest.len(), |(index, _)| index);
        if length > 0 {
            found.push(at..at + length);
            at += length;
        } else {
            at += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
    found
}

/// Whether `c` can be part of a run that makes a word.
fn is_word_character(c: char) -> bool {
    c.is_alphabetic() || c.is_numeric() || c == '\''
}

/// The end of the initials that start at `at`, or `None` when fewer than two
/// capital letters with periods start there.
fn initials(text: &str, at: usize) -> Option<usize> {
    let mut end = at;
    let mut count = 0;
    let mut chars = text.get(at..)?.chars();
    while let (Some(letter), Some('.')) = (chars.next(), chars.next()) {
        if !letter.is_ascii_uppercase() {
            break;
        }
        end += letter.len_utf8() + 1;
        count += 1;
    }
    (count >= 2).then_some(end)
}
