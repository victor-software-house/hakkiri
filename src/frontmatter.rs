//! The string values of a Markdown file's front matter.
//!
//! Vale lints each top-level string value of YAML front matter as the scope
//! `text.frontmatter.<key>`, and each top-level string of TOML and JSON front
//! matter the same way ([`metadata.go`][v-meta] and
//! [`frontmatter.go`][v-values] at `66bbb85`).
//! A key, a number, a boolean, a null, a date, a list and a nested mapping hold
//! no text it lints.
//!
//! ctl-core's [`Input::top_level`] reads the three kinds: it gives each
//! top-level key, the bytes of its value as written, how the value is written
//! and, for a string, its text.
//! JSON is read as YAML, which holds it.
//! A block that no parser reads gives no value and is a [`fault`] of the file,
//! where Vale lints nothing of the file.
//!
//! [v-meta]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/metadata.go
//! [v-values]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/frontmatter.go

#[cfg(test)]
mod tests;

use std::ops::Range;

use ctl_core::input::{Entry, Format, Input, Written};

use crate::tree::{Join, Node, Reading, Scope};

/// How the front matter of a file is written.
#[derive(Clone, Copy)]
enum Kind {
    /// Between `---` lines.
    Yaml,
    /// Between `+++` lines.
    Toml,
    /// Between `;;;` lines, which markdown-rs reads as no front matter.
    Json,
}

impl Kind {
    /// The format that reads the block: JSON is a part of YAML.
    const fn format(self) -> Format {
        match self {
            Self::Yaml | Self::Json => Format::Yaml,
            Self::Toml => Format::Toml,
        }
    }
}

/// The nodes for the string values of the front matter that `span` covers in
/// `source`, fences included.
pub(crate) fn nodes(source: &str, span: &Range<usize>, toml: bool) -> Vec<Node> {
    let kind = if toml { Kind::Toml } else { Kind::Yaml };
    content_of(source, span)
        .map(|content| entry_nodes(source, content, kind))
        .unwrap_or_default()
}

/// The nodes for the string values of the front matter that starts `source`,
/// whichever of the three kinds it is, or none when it has none.
///
/// A rewrite of the text, such as the block and token patterns do, leaves the
/// front matter as it is: Vale lints the metadata before it rewrites the body.
pub(crate) fn leading(source: &str) -> Vec<Node> {
    block(source)
        .map(|(kind, content)| entry_nodes(source, content, kind))
        .unwrap_or_default()
}

/// The nodes for the string values of JSON front matter, which a file holds
/// when it starts with a line `;;;`, a JSON object and a line `;;;`.
///
/// markdown-rs reads no such block, so the text stays a paragraph of the body,
/// as it is in Vale, which lints the block twice.
pub(crate) fn json_nodes(source: &str) -> Vec<Node> {
    match block(source) {
        Some((Kind::Json, content)) => entry_nodes(source, content, Kind::Json),
        Some((Kind::Yaml | Kind::Toml, _)) | None => Vec::new(),
    }
}

/// Why the front matter that starts `source` is no YAML, TOML or JSON that its
/// parser reads, as the line of the file and the parser's words, or `None` when
/// it has none or a parser reads it.
pub(crate) fn fault(source: &str) -> Option<String> {
    let (kind, content) = block(source)?;
    let error = input_of(source, &content, kind).top_level().err()?;
    let before = source
        .get(..content.start)
        .map_or(0, |text| text.matches('\n').count());
    error.problems.first().map(|problem| match problem.line {
        Some(line) => format!("line {}: {}", line + before, problem.message),
        None => problem.message.clone(),
    })
}

/// The kind of the front matter that starts `source` and the bytes between
/// its fence lines, or `None` when the file has none or never closes it.
///
/// A fence line may carry blanks after it, as markdown-rs reads it.
fn block(source: &str) -> Option<(Kind, Range<usize>)> {
    [
        ("---", Kind::Yaml),
        ("+++", Kind::Toml),
        (";;;", Kind::Json),
    ]
    .into_iter()
    .find_map(|(fence, kind)| {
        let rest = source.strip_prefix(fence)?.trim_start_matches([' ', '\t']);
        let rest = rest
            .strip_prefix("\r\n")
            .or_else(|| rest.strip_prefix('\n'))?;
        let start = source.len() - rest.len();
        let mut at = start;
        for line in rest.split('\n') {
            if line.trim_end() == fence {
                return Some((kind, start..at));
            }
            at += line.len() + 1;
        }
        None
    })
}

/// The input that reads `content` of `source` as front matter of `kind`.
fn input_of(source: &str, content: &Range<usize>, kind: Kind) -> Input {
    Input::new_with_format(
        "front matter",
        source.get(content.clone()).unwrap_or_default(),
        kind.format(),
    )
}

/// The nodes for the string values of the front matter whose bytes between the
/// fences are `content` in `source`.
///
/// A block the parser refuses has no values here: it is a [`fault`].
fn entry_nodes(source: &str, content: Range<usize>, kind: Kind) -> Vec<Node> {
    let Ok(entries) = input_of(source, &content, kind).top_level() else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|entry| value_node(source, content.start, entry))
        .collect()
}

/// The bytes between the opening fence line and the closing fence line of the
/// front matter that `span` covers in `source`.
fn content_of(source: &str, span: &Range<usize>) -> Option<Range<usize>> {
    let written = source.get(span.clone())?.trim_end();
    let start = written.find('\n')? + 1;
    let end = written.rfind('\n').map_or(start, |at| at + 1).max(start);
    Some(span.start + start..span.start + end)
}

/// The node of one top-level entry of the front matter, or `None` when the
/// parser reads its value as no string or the value holds no text.
///
/// `base` is where the parsed text starts in `source`, so the value's bytes
/// become bytes of the file. The segments are the text as written, which
/// escapes are not resolved in, joined as the YAML or TOML rules fold it.
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "Written is non-exhaustive: a later way to write a value holds no string this reads"
)]
fn value_node(source: &str, base: usize, entry: &Entry) -> Option<Node> {
    entry.string.as_ref()?;
    let value = base + entry.value.start..base + entry.value.end;
    let (segments, join) = match entry.written {
        Written::Plain if plain_non_string(source, &value) => return None,
        Written::Plain => (lines(source, &value), Join::Space),
        Written::SingleQuoted | Written::DoubleQuoted => (
            lines(source, &(value.start + 1..value.end.checked_sub(1)?)),
            Join::Space,
        ),
        Written::Literal => (block_lines(source, &value), Join::Newline),
        Written::Folded => (block_lines(source, &value), Join::Space),
        Written::MultilineBasic | Written::MultilineLiteral => {
            (toml_lines(source, &value), Join::Newline)
        }
        _ => return None,
    };
    node(&entry.key, segments, join)
}

/// Whether the plain scalar at `value` is a date or the merge key `<<`.
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
    !tagged && (scalar == "<<" || is_timestamp(scalar))
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
