//! The `spelling` check: the words of a text that no dictionary holds.
//!
//! A port of `spelling.go` and `spellfilter.go` in [Vale]'s `check` package,
//! and of the word tokenizer in its `nlp` package, all at `66bbb85`.
//!
//! [Vale]: https://github.com/vale-cli/vale/tree/66bbb85a1a78a5b5497331e04d4033357caced4d/internal

use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use crate::check::Hit;
use crate::rule::Spelling;
use crate::text::Text;

/// The shortest part of an identifier that `split` checks, in letters.
const MIN_PART_LETTERS: usize = 3;

/// A text that is a whole word the tokenizer keeps as it is: initials such as
/// `U.S.`, and a short name with a period such as `Mr.`.
#[expect(
    clippy::expect_used,
    reason = "The pattern is a constant that compiles."
)]
static SPECIAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:[A-Za-z]\.){2,}$|^[A-Z][a-z]{1,2}\.$").expect("the special pattern compiles")
});

/// The shapes of an identifier a `split` rule leaves whole: `snake_case`, and
/// `lowerCamel` with two or more lower-case letters first.
#[expect(
    clippy::expect_used,
    reason = "The pattern is a constant that compiles."
)]
static IDENTIFIER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:[A-Za-z][A-Za-z0-9]*(?:_[A-Za-z0-9]+)+|[a-z]{2,}[A-Z][A-Za-z0-9]*)$")
        .expect("the identifier pattern compiles")
});

/// A word that holds a character no identifier holds: not a letter, a digit,
/// `_`, `-` or `'`.
#[expect(
    clippy::expect_used,
    reason = "The pattern is a constant that compiles."
)]
static NOT_IDENTIFIER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[^\p{L}\p{Decimal_Number}_\-']").expect("the non-identifier pattern compiles")
});

/// One character of a category, tested as a text of its own.
#[expect(
    clippy::expect_used,
    reason = "The patterns are constants that compile."
)]
static LETTER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\p{L}$").expect("the letter pattern compiles"));
#[expect(
    clippy::expect_used,
    reason = "The patterns are constants that compile."
)]
static UPPER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\p{Lu}$").expect("the upper pattern compiles"));
#[expect(
    clippy::expect_used,
    reason = "The patterns are constants that compile."
)]
static LOWER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\p{Ll}$").expect("the lower pattern compiles"));

/// The emoticons the tokenizer keeps whole.
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

/// What the tokenizer reads as a quote, in place of the curly one, and the
/// entity it reads as an apostrophe.
const SANITIZED: [(&str, char); 5] = [
    ("\u{201c}", '"'),
    ("\u{201d}", '"'),
    ("\u{2018}", '\''),
    ("\u{2019}", '\''),
    ("&rsquo;", '\''),
];

/// A word the tokenizer cut from a text.
#[derive(Debug)]
struct Word {
    /// The word as the dictionary reads it: with a straight quote where the
    /// text has a curly one.
    text: String,
    /// The bytes of the word in the text.
    range: Range<usize>,
    /// For a word that holds a replaced quote, the byte of the text that each
    /// byte of `text` came from, and one more that ends the last.
    origin: Option<Vec<usize>>,
}

impl Word {
    /// The bytes in the text of the bytes `part` of the word.
    fn span(&self, part: &Range<usize>) -> Range<usize> {
        match &self.origin {
            Some(origin) => {
                let at = |offset: usize| origin.get(offset).copied().unwrap_or(self.range.end);
                at(part.start)..at(part.end)
            }
            None => self.range.start + part.start..self.range.start + part.end,
        }
    }
}

/// The text of a run of characters with the quotes `SANITIZED` names
/// replaced, and the byte each byte of it came from.
fn sanitize(run: &str, base: usize) -> (String, Option<Vec<usize>>) {
    if !SANITIZED.iter().any(|(old, _)| run.contains(old)) {
        return (run.to_owned(), None);
    }
    let mut text = String::with_capacity(run.len());
    let mut origin = Vec::with_capacity(run.len() + 1);
    let mut at = 0;
    while let Some(rest) = run.get(at..).filter(|rest| !rest.is_empty()) {
        if let Some((old, new)) = SANITIZED.iter().find(|(old, _)| rest.starts_with(old)) {
            text.push(*new);
            origin.push(base + at);
            at += old.len();
        } else if let Some(c) = rest.chars().next() {
            text.push(c);
            origin.extend((0..c.len_utf8()).map(|offset| base + at + offset));
            at += c.len_utf8();
        } else {
            break;
        }
    }
    origin.push(base + run.len());
    (text, Some(origin))
}

/// Whether `token` holds no letter.
fn no_letter(token: &str) -> bool {
    !token.chars().any(|c| is(Category::Letter, c))
}

/// Cuts the run `run` of a text, which starts at byte `base`, into words:
/// strips the prefixes and suffixes the tokenizer strips, and drops what
/// holds no letter.
fn cut(run: &str, base: usize, words: &mut Vec<Word>) {
    let (text, origin) = sanitize(run, base);
    let (mut start, mut end) = (0, text.len());
    while let Some(token) = text.get(start..end).filter(|token| !token.is_empty()) {
        let special = EMOTICONS.contains(&token) || SPECIAL.is_match(token);
        let long = token.len() > 1;
        let first = token.as_bytes().first().copied();
        let last = token.as_bytes().last().copied();
        if !special && long && first.is_some_and(|b| b"$(\"['".contains(&b)) {
            start += 1;
        } else if !special && long && last.is_some_and(|b| b",)\"]!;.?:'".contains(&b)) {
            end -= 1;
        } else {
            if !no_letter(token) {
                let slice = origin
                    .as_ref()
                    .and_then(|origin| origin.get(start..=end))
                    .map(<[usize]>::to_vec);
                let range = match slice.as_deref() {
                    Some([first, .., last]) => *first..*last,
                    _ => base + start..base + end,
                };
                words.push(Word {
                    text: token.to_owned(),
                    range,
                    origin: slice,
                });
            }
            return;
        }
    }
}

/// The words of `text`, as Vale's word tokenizer cuts them: it splits on
/// whitespace, then strips quotes and punctuation from the ends of each run.
fn tokenize(text: &str) -> Vec<Word> {
    let mut words = Vec::new();
    let mut start: Option<usize> = None;
    for (index, c) in text.char_indices() {
        match (c.is_whitespace(), start) {
            (false, None) => start = Some(index),
            (true, Some(from)) => {
                cut(text.get(from..index).unwrap_or_default(), from, &mut words);
                start = None;
            }
            (false, Some(_)) | (true, None) => {}
        }
    }
    if let Some(from) = start {
        cut(text.get(from..).unwrap_or_default(), from, &mut words);
    }
    words
}

/// Whether the filters a rule has unless it sets `custom` skip `word`.
///
/// They skip a word that holds a character other than an ASCII letter, `_`
/// or `'`, and one that ends in capitals.
/// They also skip a word made of a capital, lower-case letters and more
/// capitals, such as `FooBar`.
///
/// With `split` an identifier is taken apart and not skipped, so only a word
/// with a character no identifier holds is.
fn skipped(split: bool, word: &str) -> bool {
    if split {
        return NOT_IDENTIFIER.is_match(word);
    }
    word.bytes()
        .any(|c| !(c.is_ascii_alphabetic() || c == b'_' || c == b'\''))
        || word.bytes().last().is_some_and(|c| c.is_ascii_uppercase())
        || camel(word)
}

/// Whether `[A-Z][a-z]+[A-Z]+\w+` matches in `word`: a capital, lower-case
/// letters, then capitals and a word character.
///
/// The run of capitals gives one back to the word character, so two capitals
/// are enough.
fn camel(word: &str) -> bool {
    let bytes = word.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if !byte.is_ascii_uppercase() {
            continue;
        }
        let lower_end = bytes
            .iter()
            .skip(index + 1)
            .position(|b| !b.is_ascii_lowercase())
            .map_or(bytes.len(), |run| index + 1 + run);
        if lower_end == index + 1 {
            continue;
        }
        let upper_end = bytes
            .iter()
            .skip(lower_end)
            .position(|b| !b.is_ascii_uppercase())
            .map_or(bytes.len(), |run| lower_end + run);
        let capitals = upper_end - lower_end;
        if capitals >= 2
            || (capitals == 1
                && bytes
                    .get(upper_end)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_'))
        {
            return true;
        }
    }
    false
}

/// A category of one character: a letter, a capital or a small letter, as the
/// classes `\p{L}`, `\p{Lu}` and `\p{Ll}` name them.
#[derive(Clone, Copy)]
enum Category {
    Letter,
    Upper,
    Lower,
}

/// Whether `c` is in `category`.
///
/// An ASCII character is tested by its byte, which gives the answer of the
/// class, and any other goes through the class.
fn is(category: Category, c: char) -> bool {
    if c.is_ascii() {
        return match category {
            Category::Letter => c.is_ascii_alphabetic(),
            Category::Upper => c.is_ascii_uppercase(),
            Category::Lower => c.is_ascii_lowercase(),
        };
    }
    let pattern = match category {
        Category::Letter => &LETTER,
        Category::Upper => &UPPER,
        Category::Lower => &LOWER,
    };
    pattern.is_match(c.encode_utf8(&mut [0; 4]))
}

/// One part of an identifier: its text and the byte where it starts.
#[derive(Debug)]
struct Part<'a> {
    text: &'a str,
    at: usize,
}

/// Ends the part of `word` that began at `start`, if any, at `end`.
fn flush<'a>(word: &'a str, start: &mut Option<usize>, end: usize, parts: &mut Vec<Part<'a>>) {
    if let Some(from) = start.take()
        && end > from
        && let Some(text) = word.get(from..end)
    {
        parts.push(Part { text, at: from });
    }
}

/// Breaks an identifier at a character that is not a letter and at a change
/// of case: `getHTTPResponse_v2` is `get`, `HTTP`, `Response` and `v`.
/// The last capital of a run starts the next part, so `HTTPServer` is `HTTP`
/// and `Server`.
fn split_identifier(word: &str) -> Vec<Part<'_>> {
    let chars: Vec<(usize, char)> = word.char_indices().collect();
    let mut parts = Vec::new();
    let mut start: Option<usize> = None;
    for (index, (at, c)) in chars.iter().enumerate() {
        let before = index.checked_sub(1).and_then(|i| chars.get(i)).map(|p| p.1);
        let after = chars.get(index + 1).map(|p| p.1);
        if !is(Category::Letter, *c) {
            flush(word, &mut start, *at, &mut parts);
        } else if start.is_none() {
            start = Some(*at);
        } else if is(Category::Upper, *c)
            && (before.is_some_and(|b| is(Category::Lower, b))
                || (after.is_some_and(|a| is(Category::Lower, a))
                    && before.is_some_and(|b| is(Category::Upper, b))))
        {
            flush(word, &mut start, *at, &mut parts);
            start = Some(*at);
        }
    }
    flush(word, &mut start, word.len(), &mut parts);
    parts
}

/// Whether one part of an identifier passes: a short one or one in capitals
/// is not checked.
fn part_passes(spelling: &Spelling, part: &str) -> bool {
    let letters = part.chars().filter(|c| is(Category::Letter, *c)).count();
    letters < MIN_PART_LETTERS
        || crate::case::upper_str(part) == part
        || spelling.speller.spell(&spelling.speller.convert(part))
}

/// The number of capitals in `text`.
fn capitals(text: &str) -> usize {
    text.chars().filter(|c| is(Category::Upper, *c)).count()
}

/// Whether the dictionary's other-case spelling of `word` is a fix to offer:
/// one that adds capitals inside a word of three letters or more that has at
/// most one, as in `GitHub` or `JSON`.
///
/// A name with a capital only at its start, `Kubernetes`, cannot be told from
/// a word in another language, and a word with more capitals chose them.
fn certain_recase(word: &str, cased: &str) -> bool {
    let inner = cased.chars().skip(1).any(|c| is(Category::Upper, c));
    word.chars().count() >= MIN_PART_LETTERS
        && capitals(word) <= 1
        && capitals(cased) > capitals(word)
        && inner
}

/// A message that says more than the rule's: the dictionary spells `word` in
/// another case, or `found` is shaped like an identifier in a file with
/// inline code.
fn explain(spelling: &Spelling, word: &str, found: &str, extension: &str) -> Option<String> {
    if let Some(cased) = spelling.speller.recase(word)
        && certain_recase(word, &cased)
    {
        return Some(format!("Use '{cased}' instead of '{found}'."));
    }
    if spelling.split || !matches!(extension, "md" | "mdx") || !IDENTIFIER.is_match(found) {
        return None;
    }
    Some(format!("'{found}' looks like code; format it as code."))
}

/// Each word of `text` that no dictionary of the rule holds.
///
/// A word that overlaps a code span is not read. With `split`, an
/// identifier is reported by its misspelt parts, each at its own place.
pub(crate) fn run(spelling: &Spelling, text: &Text, extension: &str) -> Vec<Hit> {
    let mut hits = Vec::new();
    for word in tokenize(text.as_str()) {
        if text.overlaps_code(&word.range) {
            continue;
        }
        let converted = spelling.speller.convert(&word.text);
        if spelling.standard && skipped(spelling.split, &converted) {
            continue;
        }
        if spelling
            .filters
            .iter()
            .any(|filter| filter.is_match(&converted).unwrap_or(false))
        {
            continue;
        }
        if spelling.speller.spell(&converted) {
            continue;
        }
        if spelling.split {
            let parts = split_identifier(&word.text);
            if parts.len() != 1 || parts.first().is_some_and(|part| part.text != word.text) {
                for part in parts {
                    if part_passes(spelling, part.text) {
                        continue;
                    }
                    hits.push(Hit {
                        range: word.span(&(part.at..part.at + part.text.len())),
                        values: vec![part.text.to_owned()],
                        numbered: false,
                        message: None,
                    });
                }
                continue;
            }
        }
        let message = explain(spelling, &converted, &word.text, extension);
        hits.push(Hit {
            range: word.range,
            values: vec![converted],
            numbered: false,
            message,
        });
    }
    hits
}
