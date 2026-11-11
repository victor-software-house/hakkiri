//! HTML character references, read as `golang.org/x/net/html` at v0.62.0 reads
//! them.
//!
//! Vale reads the text of a Markdown file through that package, which turns
//! `&amp;` into `&` and `&copy` into `©` before a rule sees the text.
//! The named references are in `data/entities/html5.tsv`.
//!
//! The package unescapes the text of a token twice, once when the tokenizer
//! gives the token and once in Vale, so `&amp;lt;` reads as `<`.
//! This module unescapes once, so it reads `&lt;`.

use std::borrow::Cow;
use std::ops::Range;
use std::sync::LazyLock;

const TABLE: &str = include_str!("../data/entities/html5.tsv");

/// The longest name of a reference that needs no `;`.
const LONGEST_WITHOUT_SEMICOLON: usize = 6;

/// What code points `0x80` to `0x9F` read as, which old pages wrote for the
/// characters of Windows-1252.
const WINDOWS_1252: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
];

/// One named reference: its name, as the table writes it, and what it reads as.
struct Named {
    name: &'static str,
    first: char,
    second: Option<char>,
}

/// The named references, sorted by name.
static NAMED: LazyLock<Vec<Named>> = LazyLock::new(|| {
    TABLE
        .lines()
        .filter_map(|line| {
            let (name, points) = line.split_once('\t')?;
            let mut points = points
                .split(' ')
                .filter_map(|point| u32::from_str_radix(point, 16).ok())
                .filter_map(char::from_u32);
            Some(Named {
                name,
                first: points.next()?,
                second: points.next(),
            })
        })
        .collect()
});

fn named(name: &str) -> Option<&'static Named> {
    let table = NAMED.as_slice();
    let at = table.binary_search_by(|entry| entry.name.cmp(name)).ok()?;
    table.get(at)
}

/// What a reference reads as, and how many bytes of the text it takes.
struct Reference {
    first: char,
    second: Option<char>,
    length: usize,
}

impl Reference {
    const fn of(first: char, length: usize) -> Self {
        Self {
            first,
            second: None,
            length,
        }
    }
}

/// A numeric reference, `text` starting with `&#`.
fn numeric(text: &[u8]) -> Option<Reference> {
    let hex = matches!(text.get(2), Some(b'x' | b'X'));
    let digits = if hex { 3 } else { 2 };
    let mut at = digits;
    let mut value: u32 = 0;
    while let Some(&byte) = text.get(at) {
        let digit = if hex {
            char::from(byte).to_digit(16)
        } else {
            char::from(byte).to_digit(10)
        };
        let Some(digit) = digit else { break };
        if value <= 0x0010_ffff {
            value = value * if hex { 16 } else { 10 } + digit;
        }
        at += 1;
    }
    if at == digits {
        return None;
    }
    if text.get(at) == Some(&b';') {
        at += 1;
    }
    let read = match value {
        0 => None,
        0x80..=0x9f => WINDOWS_1252
            .get(usize::try_from(value - 0x80).ok()?)
            .copied(),
        _ => char::from_u32(value),
    }
    .unwrap_or('\u{FFFD}');
    Some(Reference::of(read, at))
}

/// The reference at the start of `text`, which starts with `&`.
///
/// `attribute` is true for the value of an attribute, where a name with no
/// `;` that `=` follows is not a reference, and a legacy name is read only
/// whole.
fn reference(text: &str, attribute: bool) -> Option<Reference> {
    let bytes = text.as_bytes();
    if bytes.get(1) == Some(&b'#') {
        return numeric(bytes);
    }
    let mut end = 1;
    while let Some(&byte) = bytes.get(end) {
        end += 1;
        if byte.is_ascii_alphanumeric() {
            continue;
        }
        if byte != b';' {
            end -= 1;
        }
        break;
    }
    let name = text.get(1..end).filter(|name| !name.is_empty())?;
    if attribute && !name.ends_with(';') && bytes.get(end) == Some(&b'=') {
        return None;
    }
    if let Some(found) = named(name) {
        return Some(Reference {
            first: found.first,
            second: found.second,
            length: end,
        });
    }
    if attribute {
        return None;
    }
    let longest = (name.len() - 1).min(LONGEST_WITHOUT_SEMICOLON);
    (2..=longest).rev().find_map(|length| {
        let found = named(name.get(..length)?)?;
        Some(Reference::of(found.first, length + 1))
    })
}

/// Each reference in `text`, as the bytes it takes and what it reads as.
pub(crate) fn references(text: &str, attribute: bool) -> Vec<(Range<usize>, String)> {
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(offset) = text.get(at..).and_then(|rest| rest.find('&')) {
        let start = at + offset;
        let Some(reference) = text
            .get(start..)
            .and_then(|rest| reference(rest, attribute))
        else {
            at = start + 1;
            continue;
        };
        let mut read = String::from(reference.first);
        read.extend(reference.second);
        let end = start + reference.length;
        found.push((start..end, read));
        at = end;
    }
    found
}

/// `text` with each reference read.
pub(crate) fn unescape(text: &str, attribute: bool) -> Cow<'_, str> {
    let found = references(text, attribute);
    if found.is_empty() {
        return Cow::Borrowed(text);
    }
    let mut read = String::with_capacity(text.len());
    let mut at = 0;
    for (range, value) in found {
        read.push_str(text.get(at..range.start).unwrap_or_default());
        read.push_str(&value);
        at = range.end;
    }
    read.push_str(text.get(at..).unwrap_or_default());
    Cow::Owned(read)
}
