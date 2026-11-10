//! The English tokenizer: the words and punctuation marks of one sentence.
//!
//! A port of `tokenize/tokenize.go` in [prose] at `b33e60d`, with the settings
//! of its English tokenizer, `tokenize.New()`.
//! It never rewrites the text, so each token is a byte range of the sentence.
//!
//! [prose]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/tokenize/tokenize.go

use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

/// Spans kept whole: initials such as `U.S.`, and titles such as `Dr.`.
#[expect(
    clippy::expect_used,
    reason = "The pattern is a constant that compiles."
)]
static SPECIAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:[A-Za-z]\.){2,}$|^[A-Z][a-z]{1,2}\.$").expect("the special pattern compiles")
});

/// Characters split off the front of a span, in the order they are tried.
const PREFIXES: &[&str] = &[
    "$", "(", "\"", "[", "\u{201c}", "\u{2018}", "\u{ab}", "\u{201e}", "\u{201a}", "\u{2039}",
    "\u{bf}", "\u{a1}", "\u{ff08}", "\u{300c}", "\u{300e}", "\u{3010}", "\u{300a}", "\u{3008}",
    "\u{3014}",
];

/// Characters split off the end of a span, in the order they are tried.
const SUFFIXES: &[&str] = &[
    ",", ")", "\"", "]", "!", ";", ".", "?", ":", "'", "\u{201d}", "\u{2019}", "\u{bb}",
    "\u{201c}", "\u{2018}", "\u{203a}", "\u{2026}", "\u{3002}", "\u{ff0c}", "\u{3001}", "\u{ff01}",
    "\u{ff1f}", "\u{ff1a}", "\u{ff1b}", "\u{ff09}", "\u{300d}", "\u{300f}", "\u{3011}", "\u{300b}",
    "\u{3009}", "\u{3015}",
];

/// Endings that start a token of their own, such as the `n't` of `don't`.
const CONTRACTIONS: &[&str] = &[
    "'ll",
    "'s",
    "'re",
    "'m",
    "n't",
    "\u{2019}ll",
    "\u{2019}s",
    "\u{2019}re",
    "\u{2019}m",
    "n\u{2019}t",
];

/// Spans that are emoticons, and so are one token.
const EMOTICONS: &[&str] = &[
    "(-8",
    "(-;",
    "(-_-)",
    "(._.)",
    "(:",
    "(=",
    "(o:",
    "(¬_¬)",
    "(ಠ_ಠ)",
    "(╯°□°）╯︵┻━┻",
    "-__-",
    "8-)",
    "8-D",
    "8D",
    ":(",
    ":((",
    ":(((",
    ":()",
    ":)))",
    ":-)",
    ":-))",
    ":-)))",
    ":-*",
    ":-/",
    ":-X",
    ":-]",
    ":-o",
    ":-p",
    ":-x",
    ":-|",
    ":-}",
    ":0",
    ":3",
    ":P",
    ":]",
    ":`(",
    ":`)",
    ":`-(",
    ":o",
    ":o)",
    "=(",
    "=)",
    "=D",
    "=|",
    "@_@",
    "O.o",
    "O_o",
    "V_V",
    "XDD",
    "[-:",
    "^___^",
    "o_0",
    "o_O",
    "o_o",
    "v_v",
    "xD",
    "xDD",
    "¯\\(ツ)/¯",
];

/// The byte range of each token of `sentence`.
///
/// A token is a run between spaces, with the marks in [`PREFIXES`] and
/// [`SUFFIXES`] split off its ends and the endings in [`CONTRACTIONS`] split
/// off inside it.
pub(super) fn tokens(sentence: &str) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    let mut start = 0;
    let mut white = sentence.chars().next().is_some_and(char::is_whitespace);
    for (index, c) in sentence.char_indices() {
        if c.is_whitespace() != white {
            if start < index {
                split(sentence, start..index, &mut found);
            }
            start = if c == ' ' { index + 1 } else { index };
            white = !white;
        }
    }
    if start < sentence.len() {
        split(sentence, start..sentence.len(), &mut found);
    }
    found
}

/// Breaks the run `span` of `text` into tokens, and appends them to `found`.
fn split(text: &str, span: Range<usize>, found: &mut Vec<Range<usize>>) {
    if text
        .get(span.clone())
        .is_none_or(|run| run.chars().all(char::is_whitespace))
    {
        return;
    }
    let (mut from, mut to) = (span.start, span.end);
    // Every contraction ending holds an apostrophe, so the places to look are
    // found once.
    let apostrophes: Vec<usize> = text
        .get(span.clone())
        .unwrap_or_default()
        .char_indices()
        .filter(|(_, c)| *c == '\'' || *c == '\u{2019}')
        .map(|(at, _)| span.start + at)
        .collect();
    // Suffixes come off the end, so they are collected and appended in order.
    let mut suffixes: Vec<Range<usize>> = Vec::new();
    // The earliest contraction ending found for the current `from`. It stays
    // right while `to` shrinks, unless `to` cuts into the ending.
    let mut contraction: Option<(usize, Option<Contraction>)> = None;
    while from < to {
        let rest = text.get(from..to).unwrap_or_default();
        if is_special(rest) {
            found.push(from..to);
            break;
        }
        if let Some(length) = prefix_length(rest).or_else(|| {
            let known = match contraction {
                Some((at, ending)) if at == from && ending.is_none_or(|e| e.end <= to) => ending,
                _ => {
                    let ending = find_contraction(text, &apostrophes, from, to);
                    contraction = Some((from, ending));
                    ending
                }
            };
            known.map(|ending| ending.start - from)
        }) {
            found.push(from..from + length);
            from += length;
            continue;
        }
        if let Some(length) = suffix_length(rest) {
            suffixes.push(to - length..to);
            to -= length;
            continue;
        }
        found.push(from..to);
        break;
    }
    found.extend(suffixes.into_iter().rev());
}

/// Whether `span` is an emoticon.
pub(super) fn is_emoticon(span: &str) -> bool {
    EMOTICONS.contains(&span)
}

/// Whether `span` must stay whole.
fn is_special(span: &str) -> bool {
    // Both patterns of `SPECIAL` end with a period, so the others need no scan.
    is_emoticon(span) || (span.ends_with('.') && SPECIAL.is_match(span))
}

/// The length of the prefix of `span`, when it has one and holds more.
fn prefix_length(span: &str) -> Option<usize> {
    PREFIXES
        .iter()
        .find(|prefix| span.starts_with(**prefix) && span.len() > prefix.len())
        .map(|prefix| prefix.len())
}

/// The length of the suffix of `span`, when it has one and holds more.
fn suffix_length(span: &str) -> Option<usize> {
    SUFFIXES
        .iter()
        .find(|suffix| span.ends_with(**suffix) && span.len() > suffix.len())
        .map(|suffix| suffix.len())
}

/// Where a contraction ending lies in `text`.
#[derive(Clone, Copy)]
struct Contraction {
    start: usize,
    end: usize,
}

/// The earliest contraction ending in `text[from..to]` that does not start at
/// `from`.
///
/// Case is ignored for ASCII letters only. Each ending is searched for on its
/// own and only its first match counts, as in prose.
/// An ending holds an apostrophe, at its start or one byte after it, so the
/// search starts just before the first apostrophe of `apostrophes` in the
/// window.
fn find_contraction(
    text: &str,
    apostrophes: &[usize],
    from: usize,
    to: usize,
) -> Option<Contraction> {
    let first = *apostrophes
        .get(apostrophes.partition_point(|at| *at < from))
        .filter(|at| **at < to)?;
    let scan = text.floor_char_boundary(first.saturating_sub(1)).max(from);
    let window = text.get(scan..to)?;
    CONTRACTIONS
        .iter()
        .filter_map(|ending| {
            let start = scan + find_ignoring_ascii_case(window, ending)?;
            Some((start, start + ending.len()))
        })
        .filter(|(start, _)| *start > from)
        .min()
        .map(|(start, end)| Contraction { start, end })
}

/// The byte offset of the first place `needle` occurs in `haystack`, ignoring
/// the case of ASCII letters.
fn find_ignoring_ascii_case(haystack: &str, needle: &str) -> Option<usize> {
    let length = needle.len();
    let last = haystack.len().checked_sub(length)?;
    if length == 0 {
        return None;
    }
    (0..=last).find(|at| {
        haystack
            .as_bytes()
            .get(*at..*at + length)
            .is_some_and(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
    })
}
