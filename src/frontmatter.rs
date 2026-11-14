//! The string values of a Markdown file's front matter.
//!
//! Vale lints each top-level string value of YAML front matter as the scope
//! `text.frontmatter.<key>`, and each top-level string of TOML and JSON front
//! matter the same way ([`metadata.go`][v-meta] and
//! [`frontmatter.go`][v-values] at `66bbb85`).
//! A key, a number, a boolean, a null, a date, a list and a nested mapping hold
//! no text it lints.
//!
//! The [`front-matter`](https://crates.io/crates/front-matter) crate finds the
//! block in every form it supports.
//! It reads each top-level value with the parser of the block's format, and
//! returns the key, the bytes of the value as written, how it is written and,
//! for a string, its text.
//! A block that no parser reads gives no value and is a [`fault`] of the file,
//! where Vale lints nothing of the file.
//!
//! hakkiri blanks the block out of the body before markdown-rs parses it (see
//! `markdown::build`), and reads the values from the file as written.
//!
//! [v-meta]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/metadata.go
//! [v-values]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/frontmatter.go

#[cfg(test)]
mod tests;

use std::iter::repeat_n;
use std::ops::Range;

use front_matter::{Style, Value, find, read};

use crate::tree::{Join, Node, Reading, Scope};

/// The bytes of the front matter block that starts `source`, fences and their
/// line endings included, or `None` when the file has none.
pub(crate) fn span(source: &str) -> Option<Range<usize>> {
    find(source).map(|block| block.span)
}

/// `source` with an ASCII space over each byte of `span` that is not a line
/// feed, so every offset and every line of the file stays where it was.
pub(crate) fn blank(source: &str, span: &Range<usize>) -> String {
    let mut blanked = String::with_capacity(source.len());
    for (at, character) in source.char_indices() {
        if span.contains(&at) && character != '\n' {
            blanked.extend(repeat_n(' ', character.len_utf8()));
        } else {
            blanked.push(character);
        }
    }
    blanked
}

/// The nodes for the string values of the front matter that starts `source`,
/// or none when it has none.
///
/// A rewrite of the text, such as the block and token patterns do, leaves the
/// front matter as it is: Vale lints the metadata before it rewrites the body.
pub(crate) fn leading(source: &str) -> Vec<Node> {
    let Some(block) = find(source) else {
        return Vec::new();
    };
    let Ok(values) = read(source, &block) else {
        return Vec::new();
    };
    values
        .iter()
        .filter_map(|value| value_node(source, value))
        .collect()
}

/// Why the front matter that starts `source` is no YAML, TOML or JSON that its
/// parser reads, as the line of the file and the parser's words, or `None` when
/// it has none or a parser reads it.
pub(crate) fn fault(source: &str) -> Option<String> {
    let block = find(source)?;
    read(source, &block).err().map(|fault| fault.to_string())
}

/// The node of one top-level value of the front matter, or `None` when the
/// parser reads it as no string or it holds no text.
///
/// The segments are the text as written, which escapes are not resolved in,
/// joined as the YAML or TOML rules fold it.
fn value_node(source: &str, value: &Value) -> Option<Node> {
    value.text.as_ref()?;
    let span = &value.span;
    let (segments, join) = match value.style {
        Style::Plain if plain_non_string(source, span) => return None,
        Style::Plain => (lines(source, span), Join::Space),
        Style::SingleQuoted
        | Style::DoubleQuoted
        | Style::BasicString
        | Style::LiteralString
        | Style::String => (
            lines(source, &(span.start + 1..span.end.checked_sub(1)?)),
            Join::Space,
        ),
        Style::Literal => (block_lines(source, span), Join::Newline),
        Style::Folded => (block_lines(source, span), Join::Space),
        Style::MultilineBasicString | Style::MultilineLiteralString => {
            (toml_lines(source, span), Join::Newline)
        }
        Style::Other => return None,
    };
    node(&value.key, segments, join)
}

/// Whether the plain scalar at `value` is a date, the merge key `<<` or a
/// number that yaml.v3 reads and the YAML 1.2 core schema does not.
///
/// yaml.v3, the parser Vale reads front matter with, reads them as no string.
/// serde-saphyr reads them as strings, unless a tag such as `!!str` says
/// otherwise.
fn plain_non_string(source: &str, value: &Range<usize>) -> bool {
    let line_start = source
        .get(..value.start)
        .and_then(|before| before.rfind('\n'))
        .map_or(0, |at| at + 1);
    let tagged = source
        .get(line_start..value.start)
        .and_then(|before| before.split_whitespace().next_back())
        .is_some_and(|token| token.starts_with('!'));
    let scalar = text(source, value).trim();
    !tagged && (scalar == "<<" || is_timestamp(scalar) || is_yaml_v3_number(scalar))
}

/// A number as yaml.v3 reads a plain scalar, in
/// [`resolve.go`](https://github.com/go-yaml/yaml/blob/v3/resolve.go): it
/// starts with a digit, `+` or `-`, and with every `_` removed it is an
/// integer by the rules of Go's `strconv.ParseInt(s, 0, 64)`, or a float.
/// The front-matter crate reads the YAML 1.2 core schema, which has no `_` and
/// no `0b`.
/// An integer too large for 64 bits is still a number here.
fn is_yaml_v3_number(scalar: &str) -> bool {
    if !scalar.starts_with(|first: char| first.is_ascii_digit() || first == '+' || first == '-') {
        return false;
    }
    let number: String = scalar
        .chars()
        .filter(|&character| character != '_')
        .collect();
    let unsigned = number.strip_prefix(['+', '-']).unwrap_or(&number);
    is_go_integer(unsigned) || is_go_float(unsigned)
}

/// Whether the unsigned `text` is an integer that `ParseInt` reads with base 0:
/// `0x` hex, `0o` octal, `0b` binary, a leading `0` octal, or decimal.
fn is_go_integer(text: &str) -> bool {
    let prefixed = |lower: &str, upper: &str| {
        text.strip_prefix(lower)
            .or_else(|| text.strip_prefix(upper))
    };
    let (radix, digits) = if let Some(digits) = prefixed("0x", "0X") {
        (16, digits)
    } else if let Some(digits) = prefixed("0o", "0O") {
        (8, digits)
    } else if let Some(digits) = prefixed("0b", "0B") {
        (2, digits)
    } else if text.len() > 1 && text.starts_with('0') {
        (8, text.get(1..).unwrap_or_default())
    } else {
        (10, text)
    };
    !digits.is_empty() && digits.chars().all(|digit| digit.is_digit(radix))
}

/// Whether the unsigned `text` matches yaml.v3's float expression
/// `(\.[0-9]+|[0-9]+(\.[0-9]*)?)([eE][-+]?[0-9]+)?`.
fn is_go_float(text: &str) -> bool {
    let decimal = |part: &str| !part.is_empty() && part.chars().all(|digit| digit.is_ascii_digit());
    let (mantissa, exponent) = match text.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (text, None),
    };
    let mantissa_ok = match mantissa.split_once('.') {
        Some(("", fraction)) => decimal(fraction),
        Some((whole, fraction)) => decimal(whole) && (fraction.is_empty() || decimal(fraction)),
        None => decimal(mantissa),
    };
    mantissa_ok
        && exponent
            .is_none_or(|exponent| decimal(exponent.strip_prefix(['+', '-']).unwrap_or(exponent)))
}

/// A date, or a date and a time: `2024-01-01`, `2024-01-01T10:00:00Z`.
fn is_timestamp(scalar: &str) -> bool {
    let bytes = scalar.as_bytes();
    let digits = |from: usize, to: usize| {
        bytes
            .get(from..to)
            .is_some_and(|part| !part.is_empty() && part.iter().all(u8::is_ascii_digit))
    };
    if !(digits(0, 4) && bytes.get(4) == Some(&b'-')) {
        return false;
    }
    let rest = scalar.get(5..).unwrap_or_default();
    let Some((month, day_and_more)) = rest.split_once('-') else {
        return false;
    };
    let day_end = day_and_more
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(day_and_more.len());
    let (day, tail) = day_and_more.split_at(day_end);
    let number =
        |part: &str| (1..=2).contains(&part.len()) && part.chars().all(|c| c.is_ascii_digit());
    if !number(month) || !number(day) {
        return false;
    }
    if tail.is_empty() {
        return true;
    }
    let Some(time) = tail.strip_prefix(['T', 't', ' ']) else {
        return false;
    };
    let clock = time
        .find(|c: char| !(c.is_ascii_digit() || c == ':' || c == '.'))
        .unwrap_or(time.len());
    let (clock, zone) = time.split_at(clock);
    let parts: Vec<&str> = clock.split(':').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit() || c == '.'))
        && (zone.is_empty()
            || zone.trim_start() == "Z"
            || zone.trim_start() == "z"
            || zone
                .trim_start()
                .strip_prefix(['+', '-'])
                .is_some_and(|offset| {
                    !offset.is_empty() && offset.chars().all(|c| c.is_ascii_digit() || c == ':')
                }))
}

/// The lines of `range` without the blanks at their ends, and the blank lines
/// left out.
fn lines(source: &str, range: &Range<usize>) -> Vec<Range<usize>> {
    let mut at = range.start;
    let mut found = Vec::new();
    for line in text(source, range).split('\n') {
        let line_range = trimmed(source, &(at..at + line.len()));
        if !line_range.is_empty() {
            found.push(line_range);
        }
        at += line.len() + 1;
    }
    found
}

/// The lines of the block scalar in `range`, under its header line, each cut at
/// the indent of the first line that holds text.
fn block_lines(source: &str, range: &Range<usize>) -> Vec<Range<usize>> {
    let written = text(source, range);
    let Some(header) = written.find('\n') else {
        return Vec::new();
    };
    let mut at = range.start + header + 1;
    let mut found = Vec::new();
    let mut width = None;
    for line in written.get(header + 1..).unwrap_or_default().split('\n') {
        let end = at + line.len();
        if !line.trim().is_empty() {
            let width =
                *width.get_or_insert(line.len() - line.trim_start_matches([' ', '\t']).len());
            let start = (at + width).min(end);
            found.push(start..start + text(source, &(start..end)).trim_end().len());
        }
        at = end + 1;
    }
    found
}

/// The lines of the multi-line TOML string in `range`, between its delimiters.
/// A line break right after the opening delimiter is no part of the string.
fn toml_lines(source: &str, range: &Range<usize>) -> Vec<Range<usize>> {
    let Some(end) = range.end.checked_sub(3) else {
        return Vec::new();
    };
    let inner = range.start + 3..end;
    let mut at = inner.start;
    let mut found = Vec::new();
    for (index, line) in text(source, &inner).split('\n').enumerate() {
        let line_end = at + line.trim_end_matches('\r').len();
        if index > 0 || !line.trim().is_empty() {
            found.push(at..line_end);
        }
        at += line.len() + 1;
    }
    found
}

/// The text of `range` in `source`.
fn text<'a>(source: &'a str, range: &Range<usize>) -> &'a str {
    source.get(range.clone()).unwrap_or_default()
}

/// `range` without the blanks at its ends.
fn trimmed(source: &str, range: &Range<usize>) -> Range<usize> {
    let written = text(source, range);
    let start = range.start + (written.len() - written.trim_start().len());
    let end = range.start + written.trim_end().len();
    start..end.max(start)
}

/// The node of one value: its segments, how they join, and its key.
fn node(key: &str, segments: Vec<Range<usize>>, join: Join) -> Option<Node> {
    if segments.iter().all(Range::is_empty) {
        return None;
    }
    let reads = segments
        .iter()
        .enumerate()
        .map(|(index, _)| Reading {
            before: if index == 0 { Join::Gap } else { join },
            masked: false,
        })
        .collect();
    let mut node = Node::from_segments(Scope::FrontMatter, segments, Vec::new())?.with_reads(reads);
    node.label = Some(key.to_owned());
    Some(node)
}
