//! The `capitalization` check: whether a text is in the case a rule names.
//!
//! A port of `capitalization.go` and `variables.go` in [Vale] at `66bbb85`.
//!
//! [Vale]: https://github.com/vale-cli/vale/tree/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check

use crate::case::{lower_str, title, upper_str};
use crate::rule::{Capitalization, Case};

/// The text in the case `rule` names, and whether `text` is already in it.
pub(crate) fn evaluate(rule: &Capitalization, text: &str) -> (String, bool) {
    match &rule.case {
        Case::Lower => {
            let expected = lower_str(text);
            let matched = text == expected || excepted(rule, text);
            (expected, matched)
        }
        Case::Upper => {
            let expected = upper_str(text);
            let matched = text == expected || excepted(rule, text);
            (expected, matched)
        }
        Case::Pattern(pattern) => (
            pattern.source().to_owned(),
            pattern.is_match(text).unwrap_or(false) || excepted(rule, text),
        ),
        Case::Title(converter) => agree(rule, text, &converter.convert(text), false),
        Case::Sentence(converter) => agree(rule, text, &converter.convert(text), true),
    }
}

/// Whether an exception matches `text`, anywhere in it.
fn excepted(rule: &Capitalization, text: &str) -> bool {
    rule.exceptions
        .as_ref()
        .is_some_and(|pattern| pattern.is_match(text).unwrap_or(false))
}

/// The words of `text`, as the rule counts them.
fn words<'a>(rule: &Capitalization, text: &'a str) -> Vec<&'a str> {
    rule.words
        .find_all(text)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|range| text.get(range))
        .collect()
}

/// Whether enough of the words of `text` agree with `expected`, its
/// conversion: `expected` and whether the share is at least the threshold.
///
/// A word passes when it equals the converted word at its place, or an
/// exception matches it, or, in title case, it is in capitals. A word past
/// the last converted word passes. In sentence case, a first word that is not
/// as a title would write it ends the count at once.
fn agree(rule: &Capitalization, text: &str, expected: &str, sentence: bool) -> (String, bool) {
    let converted = words(rule, expected);
    let mut count = 0.0_f64;
    let mut total = 0.0_f64;
    for (index, word) in words(rule, text).into_iter().enumerate() {
        match converted.get(index) {
            None => count += 1.0,
            Some(want) if word == *want || excepted(rule, word) => count += 1.0,
            Some(_) if !sentence && word == upper_str(word) => count += 1.0,
            Some(_) if sentence && index == 0 && word != titled(word) => {
                return (expected.to_owned(), false);
            }
            Some(_) => {}
        }
        total += 1.0;
    }
    (expected.to_owned(), count / total >= rule.threshold)
}

/// `word` as Go's `strings.Title` writes it, with every letter that follows a
/// separator in title case, after `word` is in lower case.
fn titled(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut previous = ' ';
    for c in lower_str(word).chars() {
        out.push(if separator(previous) { title(c) } else { c });
        previous = c;
    }
    out
}

/// Whether `c` separates the words of a text, as `strings.Title` reads it.
fn separator(c: char) -> bool {
    if c.is_ascii() {
        return !(c.is_ascii_alphanumeric() || c == '_');
    }
    if c.is_alphabetic() || c.is_numeric() {
        return false;
    }
    c.is_whitespace()
}

#[cfg(test)]
mod tests;
