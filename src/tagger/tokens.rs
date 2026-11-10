//! The English tokenizer: the words and punctuation marks of one sentence.
//!
//! A port of `tokenize/tokenize.go` in [prose] at `b33e60d`, with the settings
//! of its English tokenizer, `tokenize.New()`.
//! It never rewrites the text, so each token is a byte range of the sentence.
//!
//! [prose]: https://github.com/jdkato/prose/blob/b33e60dbff0492289b227f3681120ac057d8131b/tokenize/tokenize.go

use std::ops::Range;

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
pub(super) const CONTRACTIONS: &[&str] = &[
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
    let mut contractions = Contractions::new(text, span.clone());
    // Suffixes come off the end, so they are collected and appended in order.
    let mut suffixes: Vec<Range<usize>> = Vec::new();
    // Where the run of initials at the current `from` ends.
    let mut initials: Option<(usize, usize)> = None;
    while from < to {
        let rest = text.get(from..to).unwrap_or_default();
        if is_special(text, from, to, &mut initials) {
            found.push(from..to);
            break;
        }
        if let Some(length) = prefix_length(rest).or_else(|| contractions.earliest(from, to)) {
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

/// Whether `text[from..to]` must stay whole: an emoticon, initials such as
/// `U.S.`, or a title such as `Dr.`.
///
/// `initials` holds where the run of initials from `from` ends, once it is
/// found, and stays right for every `to` while `from` stays the same. A run is
/// scanned once for each `from`, so a token of many marks costs a pass over
/// the token and not a pass for each mark.
pub(super) fn is_special(
    text: &str,
    from: usize,
    to: usize,
    initials: &mut Option<(usize, usize)>,
) -> bool {
    let Some(span) = text.get(from..to) else {
        return false;
    };
    is_emoticon(span) || is_initials(text, from, to, initials) || is_title(span)
}

/// Whether `text[from..to]` is two or more of a letter and a period.
fn is_initials(text: &str, from: usize, to: usize, known: &mut Option<(usize, usize)>) -> bool {
    let length = to.saturating_sub(from);
    if length < 4 || !length.is_multiple_of(2) || text.as_bytes().get(to - 1) != Some(&b'.') {
        return false;
    }
    let end = match *known {
        Some((at, end)) if at == from => end,
        _ => {
            let bytes = text.as_bytes();
            let mut end = from;
            while bytes.get(end).is_some_and(u8::is_ascii_alphabetic)
                && bytes.get(end + 1) == Some(&b'.')
            {
                end += 2;
            }
            *known = Some((from, end));
            end
        }
    };
    to <= end
}

/// Whether `span` is a capital, one or two lower-case letters and a period.
fn is_title(span: &str) -> bool {
    matches!(
        span.as_bytes(),
        [first, middle @ .., b'.']
            if first.is_ascii_uppercase()
                && (1..=2).contains(&middle.len())
                && middle.iter().all(u8::is_ascii_lowercase)
    )
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

/// Where each ending of [`CONTRACTIONS`] occurs in one span.
///
/// Every ending holds an apostrophe, at its start or one byte after it, so
/// each ending is checked at the apostrophes of the span, once.
/// A span with no apostrophe holds nothing.
pub(super) struct Contractions {
    /// For each ending, the byte offset of each place it occurs, in order.
    starts: Vec<Vec<usize>>,
    /// For each ending, the first place in `starts` at or after the last
    /// `from` asked about. `from` only grows, so it only moves forward.
    cursors: Vec<usize>,
}

impl Contractions {
    /// The endings in `text[span]`. Case is ignored for ASCII letters only.
    pub(super) fn new(text: &str, span: Range<usize>) -> Self {
        let apostrophes: Vec<usize> = text
            .get(span.clone())
            .unwrap_or_default()
            .char_indices()
            .filter(|(_, c)| *c == '\'' || *c == '\u{2019}')
            .map(|(at, _)| span.start + at)
            .collect();
        if apostrophes.is_empty() {
            return Self {
                starts: Vec::new(),
                cursors: Vec::new(),
            };
        }
        let starts: Vec<Vec<usize>> = CONTRACTIONS
            .iter()
            .map(|ending| {
                let offset = ending.find(['\'', '\u{2019}']).unwrap_or(0);
                apostrophes
                    .iter()
                    .filter_map(|at| at.checked_sub(offset))
                    .filter(|start| {
                        *start >= span.start
                            && text
                                .as_bytes()
                                .get(*start..*start + ending.len())
                                .is_some_and(|found| {
                                    *start + ending.len() <= span.end
                                        && found.eq_ignore_ascii_case(ending.as_bytes())
                                })
                    })
                    .collect()
            })
            .collect();
        let cursors = vec![0; starts.len()];
        Self { starts, cursors }
    }

    /// Where the earliest ending in `text[from..to]` starts, counted from
    /// `from`, when that is not the first byte.
    ///
    /// Each ending counts only at its first place from `from` on, as in
    /// prose, so an ending that starts at `from` counts for nothing.
    /// `from` must not be smaller than in an earlier call.
    pub(super) fn earliest(&mut self, from: usize, to: usize) -> Option<usize> {
        self.starts
            .iter()
            .zip(&mut self.cursors)
            .zip(CONTRACTIONS)
            .filter_map(|((starts, cursor), ending)| {
                while starts.get(*cursor).is_some_and(|start| *start < from) {
                    *cursor += 1;
                }
                let start = *starts.get(*cursor)?;
                (start > from && start + ending.len() <= to).then_some(start - from)
            })
            .min()
    }
}
