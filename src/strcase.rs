//! Sentence and title case converters.
//!
//! A port of the `strcase` package of [prose] at `b33e60d`.
//! It takes the options Vale 3.26.0 gives its converters.
//! Words may keep the case they are written with, and a prefix is left alone.
//! In sentence case, some words make the next word start with a capital.
//!
//! A pattern that gives up on a text, which `fancy-regex` does after too many
//! steps, finds nothing, as prose reads it.
//!
//! [prose]: https://github.com/jdkato/prose/tree/b33e60dbff0492289b227f3681120ac057d8131b/strcase

use std::cmp::Reverse;

use crate::case::{lower_str, title, upper_str};
use crate::pattern::{Pattern, PatternError};
use crate::tagger::Tagger;

/// The articles, conjunctions and short prepositions that stay in lower case.
const SMALL_WORDS: &[&str] = &[
    "a", "an", "and", "as", "at", "but", "by", "en", "for", "if", "in", "nor", "of", "on", "or",
    "per", "the", "to", "vs", "vs.", "via", "v", "v.",
];

/// The longer prepositions that Chicago style keeps in lower case too.
const PREPOSITIONS: &[&str] = &[
    "with",
    "from",
    "into",
    "during",
    "including",
    "until",
    "against",
    "among",
    "throughout",
    "despite",
    "towards",
    "upon",
    "concerning",
    "about",
    "over",
    "through",
    "before",
    "between",
    "after",
    "since",
    "without",
    "under",
    "within",
    "along",
    "following",
    "across",
    "beyond",
    "around",
    "down",
    "near",
    "above",
];

/// The words of a title: a run of letters and digits, and what follows it up
/// to a space, a hyphen or a slash.
const TITLE_WORD: &str = r"[\p{N}\p{L}]+[^\t\n\f\r \-/]*";

/// The words of a sentence, and of the texts a `capitalization` rule counts:
/// a run of letters, digits and asterisks, and what follows it up to a space.
pub const WORD: &str = r"[\p{N}\p{L}*]+[^\t\n\f\r ]*";

/// How a title is cased.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Style {
    /// The Associated Press Stylebook: the articles, conjunctions and
    /// prepositions of three letters or fewer stay in lower case, and `to`
    /// does only before a noun.
    Ap,
    /// The Chicago Manual of Style: the articles, conjunctions and
    /// prepositions of any length stay in lower case.
    Chicago,
}

/// The words that keep the case they are written with.
#[derive(Debug, Default)]
struct Vocab {
    /// Each word, longest first, and its pattern, when it is one.
    entries: Vec<(String, Option<Pattern>)>,
}

impl Vocab {
    fn new(words: &[String]) -> Self {
        let mut longest_first: Vec<&String> = words.iter().collect();
        longest_first.sort_by_key(|word| Reverse(word.len()));
        Self {
            entries: longest_first
                .into_iter()
                .map(|word| (word.clone(), Pattern::new(word).ok()))
                .collect(),
        }
    }

    /// The entry equal to `word` without regard to case, `word` when a
    /// pattern of an entry matches it, or nothing.
    fn lookup(&self, word: &str) -> Option<String> {
        let folded = fold(word);
        self.entries.iter().find_map(|(entry, pattern)| {
            if fold(entry) == folded {
                Some(entry.clone())
            } else if pattern
                .as_ref()
                .is_some_and(|pattern| pattern.is_match(word).unwrap_or(false))
            {
                Some(word.to_owned())
            } else {
                None
            }
        })
    }
}

/// `text` as a case-insensitive comparison reads it.
fn fold(text: &str) -> String {
    lower_str(&upper_str(text))
}

/// `word` with its first character in title case, and the rest in lower case
/// when `strict`.
fn to_title(word: &str, strict: bool) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let rest = chars.as_str();
    let mut out = String::with_capacity(word.len());
    out.push(title(first));
    if strict {
        out.push_str(&lower_str(rest));
    } else {
        out.push_str(rest);
    }
    out
}

/// What a `prefix` pattern finds at the start of a text, and the text after
/// it.
fn split_prefix<'a>(prefix: Option<&Pattern>, text: &'a str) -> (&'a str, &'a str) {
    let Some(pattern) = prefix else {
        return ("", text);
    };
    let found = pattern
        .find_all(text)
        .ok()
        .and_then(|found| found.into_iter().next())
        .and_then(|range| text.get(range));
    match found {
        Some(found) => (found, text.strip_prefix(found).unwrap_or(text)),
        None => ("", text),
    }
}

/// Converts a text to sentence case.
#[derive(Debug)]
pub struct Sentence {
    vocab: Vocab,
    words: Pattern,
    prefix: Option<Pattern>,
    indicators: Vec<String>,
}

impl Sentence {
    /// A converter that keeps the words of `vocab` as written, leaves a
    /// `prefix` alone, and starts the word after one that ends with an
    /// `indicators` entry with a capital.
    ///
    /// # Errors
    ///
    /// Returns a [`PatternError`] when `prefix`, or the words of `vocab`
    /// taken together, are not a pattern.
    pub fn new(
        vocab: &[String],
        prefix: &str,
        indicators: &[String],
    ) -> Result<Self, PatternError> {
        let vocab = Vocab::new(vocab);
        let alternatives: Vec<&str> = vocab
            .entries
            .iter()
            .map(|(entry, _)| entry.as_str())
            .collect();
        let words = if alternatives.is_empty() {
            format!("(?i){WORD}")
        } else {
            format!(r"(?i)\b(?:{})\b|{WORD}", alternatives.join("|"))
        };
        Ok(Self {
            vocab,
            words: Pattern::new(&words)?,
            prefix: if prefix.is_empty() {
                None
            } else {
                Some(Pattern::new(prefix)?)
            },
            indicators: indicators.to_vec(),
        })
    }

    /// Whether the word after `word`, at `index` in the text, starts with a
    /// capital: `word` ends with an indicator, or is the first and holds a
    /// number and a period.
    fn indicates(&self, word: &str, index: usize) -> bool {
        self.indicators
            .iter()
            .any(|indicator| word.ends_with(indicator.as_str()))
            || (index == 0 && has_number_list(word))
    }

    /// `text` in sentence case.
    #[must_use]
    pub fn convert(&self, text: &str) -> String {
        let (prefix, rest) = split_prefix(self.prefix.as_ref(), text);
        let tokens: Vec<&str> = self
            .words
            .find_all(rest)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|range| rest.get(range))
            .collect();
        let mut made: Vec<String> = Vec::with_capacity(tokens.len());
        for (index, token) in tokens.iter().enumerate() {
            let after_indicator = index
                .checked_sub(1)
                .and_then(|before| tokens.get(before).map(|word| (word, before)))
                .is_some_and(|(word, before)| self.indicates(word, before));
            made.push(if let Some(entry) = self.vocab.lookup(token) {
                entry
            } else if is_acronym(token) {
                (*token).to_owned()
            } else if index == 0 || after_indicator {
                to_title(token, true)
            } else {
                lower_str(token)
            });
        }
        format!("{prefix}{}", made.join(" "))
    }
}

/// Whether `word` holds a run of digits and then a period.
fn has_number_list(word: &str) -> bool {
    let bytes = word.as_bytes();
    bytes.iter().enumerate().any(|(at, byte)| {
        byte.is_ascii_digit()
            && bytes
                .get(at..)
                .and_then(|rest| rest.iter().find(|next| !next.is_ascii_digit()))
                == Some(&b'.')
    })
}

/// Whether `token` is written in capitals, like `TAP` or `CLI`, so sentence
/// case keeps it as written. A plural or possessive and the punctuation after
/// the word do not count against it.
fn is_acronym(token: &str) -> bool {
    let token = token.trim_end_matches(|c: char| !c.is_alphabetic() && !c.is_numeric());
    let token = ["'s", "\u{2019}s", "s"]
        .iter()
        .find_map(|suffix| token.strip_suffix(suffix))
        .unwrap_or(token);
    let mut letters = 0;
    for c in token.chars() {
        if c.is_lowercase() {
            return false;
        }
        if c.is_alphabetic() {
            letters += 1;
        }
    }
    letters >= 2
}

/// Converts a text to title case in one style.
#[derive(Debug)]
pub struct Title {
    style: Style,
    vocab: Vocab,
    words: Pattern,
    prefix: Option<Pattern>,
}

impl Title {
    /// A converter of `style` that keeps the words of `vocab` as written and
    /// leaves a `prefix` alone.
    ///
    /// # Errors
    ///
    /// Returns a [`PatternError`] when `prefix` is not a pattern.
    pub fn new(style: Style, vocab: &[String], prefix: &str) -> Result<Self, PatternError> {
        Ok(Self {
            style,
            vocab: Vocab::new(vocab),
            words: Pattern::new(TITLE_WORD)?,
            prefix: if prefix.is_empty() {
                None
            } else {
                Some(Pattern::new(prefix)?)
            },
        })
    }

    /// Whether the word `lower`, number `index` of the title and tagged with
    /// `tags`, stays in lower case when `bounding` says it is first or last.
    fn lowers(&self, lower: &str, tags: &[String], index: usize, bounding: bool) -> bool {
        match self.style {
            Style::Ap => {
                if lower == "to"
                    && let Some(next) = tags.get(index + 1)
                {
                    return next.starts_with("NN");
                }
                !bounding && SMALL_WORDS.contains(&lower)
            }
            Style::Chicago => {
                !bounding && (SMALL_WORDS.contains(&lower) || PREPOSITIONS.contains(&lower))
            }
        }
    }

    /// `text` in title case.
    ///
    /// Each word is read where prose reads it.
    /// Its place is searched for in a copy of the text whose curly quotes,
    /// dashes and ellipses are plain marks, from where the last word ended,
    /// counted in characters.
    /// The tag after `to` is the one of the word that follows it in the
    /// Treebank tokenizer's words, which are not always the words of the title.
    #[must_use]
    pub fn convert(&self, text: &str) -> String {
        let (prefix, rest) = split_prefix(self.prefix.as_ref(), text);
        let plain = sanitize(rest);
        let plain = plain.as_bytes();
        let end = to_isize(plain.len());
        let for_tagging = if rest.ends_with(['.', '!', '?']) {
            rest.to_owned()
        } else {
            format!("{rest}.")
        };
        let tags: Vec<String> = Tagger::english()
            .map(|tagger| tagger.tag_treebank(&for_tagging))
            .unwrap_or_default();
        let mut out = String::with_capacity(rest.len());
        let mut copied = 0;
        let mut index = 0_isize;
        for (word_number, range) in self
            .words
            .find_all(rest)
            .unwrap_or_default()
            .into_iter()
            .enumerate()
        {
            let Some(word) = rest.get(range.clone()) else {
                continue;
            };
            out.push_str(rest.get(copied..range.start).unwrap_or_default());
            copied = range.end;
            let from = usize::try_from(index).unwrap_or_default();
            let found = plain
                .get(from..)
                .and_then(|tail| find_bytes(tail, word.as_bytes()));
            let position = found.map_or(-1, to_isize) + index;
            let before = byte_at(plain, position - 1);
            let width = to_isize(word.chars().count());
            index = position + width;
            let lower = lower_str(word);
            out.push_str(&if let Some(entry) = self.vocab.lookup(word) {
                entry
            } else if self.lowers(&lower, &tags, word_number, position == 0 || index == end)
                && matches!(before, b' ' | b'-' | b'/')
                && byte_at(plain, position - 2) != b':'
                && byte_at(plain, position - 2) != b'-'
                && (byte_at(plain, position + width) != b'-' || before == b'-')
            {
                lower
            } else {
                to_title(word, false)
            });
        }
        out.push_str(rest.get(copied..).unwrap_or_default());
        format!("{prefix}{out}")
    }
}

/// `text` with the curly quotes, the dashes and the ellipsis as ASCII marks.
fn sanitize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{201c}' | '\u{201d}' => out.push('"'),
            '\u{2018}' | '\u{2019}' => out.push('\''),
            '\u{2013}' | '\u{2014}' => out.push('-'),
            '\u{2026}' => out.push_str("..."),
            _ => out.push(c),
        }
    }
    out
}

/// The byte at `at`, or 0 when `at` is outside the text.
fn byte_at(bytes: &[u8], at: isize) -> u8 {
    usize::try_from(at)
        .ok()
        .and_then(|at| bytes.get(at))
        .copied()
        .unwrap_or(0)
}

/// Where `needle` first occurs in `haystack`.
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// A length as a signed number, which is how prose counts positions.
fn to_isize(length: usize) -> isize {
    isize::try_from(length).unwrap_or(isize::MAX)
}
