//! The string values of a Markdown file's front matter.
//!
//! Vale lints each top-level string value of YAML front matter as the scope
//! `text.frontmatter.<key>`, and each top-level string of TOML front matter the
//! same way ([`metadata.go`][v-meta] and [`frontmatter.go`][v-values] at
//! `66bbb85`).
//! A key, a number, a boolean, a null, a date, a list and a nested mapping hold
//! no text it lints.
//!
//! The YAML read here is the part of it that a top-level `key: value` line
//! uses: a plain, single-quoted or double-quoted scalar, which may go on over
//! the lines under it, and a `|` or `>` block scalar.
//!
//! [v-meta]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/metadata.go
//! [v-values]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/core/frontmatter.go

#[cfg(test)]
mod tests;

use std::ops::Range;

use crate::tree::{Join, Node, Reading, Scope};

/// The nodes for the string values of the front matter that `span` covers in
/// `source`, fences included.
pub(crate) fn nodes(source: &str, span: &Range<usize>, toml: bool) -> Vec<Node> {
    let lines = content_lines(source, span);
    if toml {
        toml_values(source, &lines)
    } else {
        yaml_values(source, &lines)
    }
}

/// The nodes for the string values of the front matter that starts `source`,
/// whichever of the three kinds it is, or none when it has none.
///
/// A rewrite of the text, such as the block and token patterns do, leaves the
/// front matter as it is: Vale lints the metadata before it rewrites the body.
pub(crate) fn leading(source: &str) -> Vec<Node> {
    for (fence, toml) in [("---", false), ("+++", true)] {
        let Some(rest) = source.strip_prefix(fence).and_then(|rest| {
            rest.strip_prefix("\r\n")
                .or_else(|| rest.strip_prefix('\n'))
        }) else {
            continue;
        };
        let mut at = source.len() - rest.len();
        for line in rest.split('\n') {
            if line.trim_end() == fence {
                return nodes(source, &(0..at + fence.len()), toml);
            }
            at += line.len() + 1;
        }
        return Vec::new();
    }
    json_nodes(source)
}

/// The nodes for the string values of JSON front matter, which a file holds
/// when it starts with a line `;;;`, a JSON object and a line `;;;`.
///
/// markdown-rs reads no such block, so the text stays a paragraph of the body,
/// as it is in Vale, which lints the block twice.
pub(crate) fn json_nodes(source: &str) -> Vec<Node> {
    let Some(rest) = source
        .strip_prefix(";;;\n")
        .or_else(|| source.strip_prefix(";;;\r\n"))
    else {
        return Vec::new();
    };
    let start = source.len() - rest.len();
    let mut end = None;
    let mut at = start;
    for line in rest.split('\n') {
        if line.trim_end_matches('\r') == ";;;" {
            end = Some(at);
            break;
        }
        at += line.len() + 1;
    }
    let Some(end) = end else { return Vec::new() };
    json_values(source, start..end)
}

/// The top-level string values of the JSON object in `span`.
fn json_values(source: &str, span: Range<usize>) -> Vec<Node> {
    let bytes = source.as_bytes();
    let mut found = Vec::new();
    let mut depth = 0_usize;
    let mut at = span.start;
    let mut key: Option<String> = None;
    while at < span.end {
        match bytes.get(at) {
            Some(b'{' | b'[') => {
                depth += 1;
                at += 1;
            }
            Some(b'}' | b']') => {
                depth = depth.saturating_sub(1);
                at += 1;
            }
            Some(b'"') => {
                let Some(close) = json_string_end(bytes, at, span.end) else {
                    break;
                };
                let inner = at + 1..close;
                if depth == 1 {
                    let after = source
                        .get(close + 1..span.end)
                        .unwrap_or_default()
                        .trim_start();
                    if after.starts_with(':') {
                        key = Some(text(source, &inner).to_owned());
                    } else if let Some(key) = key.take() {
                        found.extend(node(&key, vec![inner], Join::Space));
                    }
                }
                at = close + 1;
            }
            Some(b',') => {
                if depth == 1 {
                    key = None;
                }
                at += 1;
            }
            _ => at += 1,
        }
    }
    found
}

/// The byte of the quote that closes the string opened at `open`, in the first
/// `end` bytes of `bytes`.
fn json_string_end(bytes: &[u8], open: usize, end: usize) -> Option<usize> {
    let mut at = open + 1;
    while at < end {
        match bytes.get(at)? {
            b'\\' => at += 2,
            b'"' => return Some(at),
            _ => at += 1,
        }
    }
    None
}

/// The bytes of each line between the fences, line ending and carriage return
/// left out.
fn content_lines(source: &str, span: &Range<usize>) -> Vec<Range<usize>> {
    let text = source.get(span.clone()).unwrap_or_default();
    let mut lines = Vec::new();
    let mut at = span.start;
    for line in text.split('\n') {
        let end = at + line.trim_end_matches('\r').len();
        lines.push(at..end);
        at += line.len() + 1;
    }
    let fence = |line: Option<&Range<usize>>| {
        line.and_then(|line| source.get(line.clone()))
            .is_some_and(|text| matches!(text.trim(), "---" | "+++"))
    };
    if fence(lines.last()) && lines.len() > 1 {
        lines.pop();
    }
    if fence(lines.first()) {
        lines.remove(0);
    }
    lines
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

/// Whether the line `range` is blank.
fn is_blank(source: &str, range: &Range<usize>) -> bool {
    text(source, range).trim().is_empty()
}

/// How many spaces start the line.
fn indent(source: &str, range: &Range<usize>) -> usize {
    let written = text(source, range);
    written.len() - written.trim_start_matches([' ', '\t']).len()
}

/// The bytes of a plain scalar's text on one line: up to a comment.
fn plain(source: &str, range: &Range<usize>) -> Range<usize> {
    let written = text(source, range);
    let end = written
        .find(" #")
        .or_else(|| written.find("\t#"))
        .unwrap_or(written.len());
    trimmed(source, &(range.start..range.start + end))
}

/// The key of the entry in `line` and the offset after its colon.
fn key_of(line: &str) -> Option<(String, usize)> {
    let first = line.chars().next()?;
    if first == '"' || first == '\'' {
        let mut chars = line.char_indices().skip(1);
        let mut close = None;
        while let Some((at, c)) = chars.next() {
            if first == '"' && c == '\\' {
                chars.next();
            } else if c == first {
                close = Some(at);
                break;
            }
        }
        let close = close?;
        let rest = line.get(close + 1..)?;
        let colon = close + 1 + (rest.len() - rest.trim_start().len());
        return line
            .get(colon..)
            .is_some_and(|after| after.starts_with(':'))
            .then(|| (line.get(1..close).unwrap_or_default().to_owned(), colon + 1));
    }
    if matches!(
        first,
        '#' | '[' | '{' | '&' | '*' | '!' | '|' | '>' | '%' | '@' | '`'
    ) {
        return None;
    }
    if line.starts_with("- ") || line == "-" {
        return None;
    }
    let bytes = line.as_bytes();
    let at = (0..bytes.len()).find(|&at| {
        bytes.get(at) == Some(&b':')
            && bytes
                .get(at + 1)
                .is_none_or(|next| matches!(next, b' ' | b'\t'))
    })?;
    let key = line.get(..at)?.trim_end();
    (!key.is_empty()).then(|| (key.to_owned(), at + 1))
}

/// Whether a plain scalar resolves to something other than a string, as
/// `yaml.v3` resolves it: null, a boolean, a number, a date or a merge key.
fn resolves_to_text(scalar: &str) -> bool {
    !(is_null(scalar)
        || matches!(
            scalar,
            "true" | "True" | "TRUE" | "false" | "False" | "FALSE" | "<<"
        )
        || is_number(scalar)
        || is_timestamp(scalar))
}

fn is_null(scalar: &str) -> bool {
    matches!(scalar, "" | "~" | "null" | "Null" | "NULL")
}

fn is_number(scalar: &str) -> bool {
    let unsigned = scalar.strip_prefix(['-', '+']).unwrap_or(scalar);
    let all = |digits: &str, valid: fn(char) -> bool| {
        !digits.is_empty() && digits.chars().all(|c| valid(c) || c == '_')
    };
    if let Some(hex) = unsigned.strip_prefix("0x") {
        return all(hex, |c| c.is_ascii_hexdigit());
    }
    if let Some(octal) = unsigned.strip_prefix("0o") {
        return all(octal, |c| ('0'..='7').contains(&c));
    }
    if let Some(binary) = unsigned.strip_prefix("0b") {
        return all(binary, |c| matches!(c, '0' | '1'));
    }
    if matches!(unsigned, ".inf" | ".Inf" | ".INF") {
        return true;
    }
    if matches!(scalar, ".nan" | ".NaN" | ".NAN") {
        return true;
    }
    if all(unsigned, |c| c.is_ascii_digit()) {
        return unsigned.starts_with(|c: char| c.is_ascii_digit());
    }
    is_float(unsigned)
}

/// A float: digits with a point, an exponent or both, and a digit before or
/// after the point.
fn is_float(unsigned: &str) -> bool {
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(at) => (
            unsigned.get(..at).unwrap_or_default(),
            unsigned.get(at + 1..),
        ),
        None => (unsigned, None),
    };
    if let Some(exponent) = exponent {
        let digits = exponent.strip_prefix(['-', '+']).unwrap_or(exponent);
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    let (whole, fraction) = match mantissa.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (mantissa, None),
    };
    let digits = |part: &str| part.chars().all(|c| c.is_ascii_digit());
    if !digits(whole) || !fraction.is_none_or(digits) {
        return false;
    }
    let any_digit = !whole.is_empty() || fraction.is_some_and(|part| !part.is_empty());
    any_digit && (fraction.is_some() || exponent.is_some())
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

/// The segments of a quoted scalar that starts at the byte `open` of the line
/// `from`, and the line after it, or `None` when nothing closes it.
fn quoted(
    source: &str,
    lines: &[Range<usize>],
    from: usize,
    open: usize,
    quote: char,
) -> Option<(Vec<Range<usize>>, usize)> {
    let mut segments = Vec::new();
    let mut index = from;
    let mut start = open + 1;
    loop {
        let line = lines.get(index)?;
        let written = text(source, &(start..line.end));
        let mut close = None;
        let mut chars = written.char_indices().peekable();
        while let Some((at, c)) = chars.next() {
            if quote == '"' && c == '\\' {
                chars.next();
            } else if c == quote {
                if quote == '\'' && chars.peek().is_some_and(|(_, next)| *next == '\'') {
                    chars.next();
                } else {
                    close = Some(at);
                    break;
                }
            }
        }
        if let Some(close) = close {
            segments.push(trimmed(source, &(start..start + close)));
            return Some((segments, index + 1));
        }
        segments.push(trimmed(source, &(start..line.end)));
        index += 1;
        while lines.get(index).is_some_and(|next| is_blank(source, next)) {
            index += 1;
        }
        let next = lines.get(index)?;
        start = next.start;
    }
}

/// The values of YAML front matter whose lines are `lines`.
fn yaml_values(source: &str, lines: &[Range<usize>]) -> Vec<Node> {
    let mut found = Vec::new();
    let mut index = 0;
    while let Some(line) = lines.get(index) {
        let written = text(source, line);
        let unindented = written.trim_end();
        if indent(source, line) > 0 || unindented.is_empty() || unindented.starts_with('#') {
            index += 1;
            continue;
        }
        let Some((key, after)) = key_of(unindented) else {
            index += 1;
            continue;
        };
        // The lines under the entry: blank or indented.
        let mut end = index + 1;
        while lines
            .get(end)
            .is_some_and(|next| is_blank(source, next) || indent(source, next) > 0)
        {
            end += 1;
        }
        let value = line.start + after;
        let value = trimmed(source, &(value..line.end));
        found.extend(yaml_value(source, lines, index, end, &key, &value));
        index = end;
    }
    found
}

/// The node of the entry `key` whose lines are `from..end`, and whose text on
/// its first line is `value`.
fn yaml_value(
    source: &str,
    lines: &[Range<usize>],
    from: usize,
    end: usize,
    key: &str,
    value: &Range<usize>,
) -> Option<Node> {
    let mut first = text(source, value);
    let mut start = from;
    let mut at = value.start;
    if first.is_empty() || first.starts_with('#') {
        // The value starts on the next line that holds text.
        start = (from + 1..end)
            .find(|&next| lines.get(next).is_some_and(|line| !is_blank(source, line)))?;
        let line = lines.get(start)?;
        let range = trimmed(source, line);
        first = text(source, &range);
        at = range.start;
        let collection = first.starts_with("- ")
            || first == "-"
            || first.starts_with(['[', '{'])
            || key_of(first).is_some();
        if collection {
            return None;
        }
    }
    if let Some(header) = first.split([' ', '\t']).next()
        && start == from
        && (header.starts_with('|') || header.starts_with('>'))
        && header
            .chars()
            .skip(1)
            .all(|c| matches!(c, '-' | '+' | '0'..='9'))
    {
        return block_scalar(source, lines, from, end, key, header.starts_with('>'));
    }
    match first.chars().next()? {
        '[' | '{' | '&' | '*' | '%' | '@' | '`' | '|' | '>' => None,
        '"' => quoted(source, lines, start, at, '"')
            .and_then(|(segments, _)| node(key, segments, Join::Space)),
        '\'' => quoted(source, lines, start, at, '\'')
            .and_then(|(segments, _)| node(key, segments, Join::Space)),
        '!' => {
            let rest = first.strip_prefix("!!str ")?;
            let offset = at + (first.len() - rest.len());
            let range = plain(source, &(offset..at + first.len()));
            node(key, vec![range], Join::Space)
        }
        _ => {
            let line = lines.get(start)?;
            let range = plain(source, &(at..line.end));
            let mut segments = vec![range.clone()];
            for next in lines.get(start + 1..end).unwrap_or_default() {
                let written = text(source, next);
                if written.trim().is_empty() || written.trim_start().starts_with('#') {
                    continue;
                }
                segments.push(plain(source, &trimmed(source, next)));
            }
            if segments.len() == 1 && !resolves_to_text(text(source, &range)) {
                return None;
            }
            node(key, segments, Join::Space)
        }
    }
}

/// The node of the block scalar of the entry `key` that starts on the line
/// `from`, over the lines up to `end`.
fn block_scalar(
    source: &str,
    lines: &[Range<usize>],
    from: usize,
    end: usize,
    key: &str,
    folded: bool,
) -> Option<Node> {
    let body = lines.get(from + 1..end)?;
    let width = body
        .iter()
        .filter(|line| !is_blank(source, line))
        .map(|line| indent(source, line))
        .next()?;
    let segments: Vec<Range<usize>> = body
        .iter()
        .filter(|line| !is_blank(source, line))
        .map(|line| {
            let start = (line.start + width).min(line.end);
            start..start + text(source, &(start..line.end)).trim_end().len()
        })
        .collect();
    node(
        key,
        segments,
        if folded { Join::Space } else { Join::Newline },
    )
}

/// The values of TOML front matter whose lines are `lines`: the strings
/// before the first table.
fn toml_values(source: &str, lines: &[Range<usize>]) -> Vec<Node> {
    let mut found = Vec::new();
    let mut index = 0;
    while let Some(line) = lines.get(index) {
        let written = text(source, line);
        let unindented = written.trim();
        index += 1;
        if unindented.starts_with('[') {
            break;
        }
        if unindented.is_empty() || unindented.starts_with('#') {
            continue;
        }
        let Some((key, value)) = written.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let key = key
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
            .or_else(|| {
                key.strip_prefix('\'')
                    .and_then(|rest| rest.strip_suffix('\''))
            })
            .unwrap_or(key);
        if key.is_empty() || key.contains('.') || key.contains(' ') {
            continue;
        }
        let offset = line.start + written.len() - value.len();
        let value_range = trimmed(source, &(offset..line.end));
        let first = text(source, &value_range);
        for triple in ["\"\"\"", "'''"] {
            if let Some(rest) = first.strip_prefix(triple) {
                let (segments, next) = toml_multiline(
                    source,
                    lines,
                    index - 1,
                    value_range.start + 3,
                    rest,
                    triple,
                );
                found.extend(node(key, segments, Join::Newline));
                index = next;
                break;
            }
        }
        if first.starts_with("\"\"\"") || first.starts_with("'''") {
            continue;
        }
        let quote = first.chars().next().filter(|c| matches!(c, '"' | '\''));
        let Some(quote) = quote else { continue };
        let inner = first.get(1..).unwrap_or_default();
        let mut chars = inner.char_indices();
        let mut close = None;
        while let Some((at, c)) = chars.next() {
            if quote == '"' && c == '\\' {
                chars.next();
            } else if c == quote {
                close = Some(at);
                break;
            }
        }
        if let Some(close) = close {
            let start = value_range.start + 1;
            let segment = start..start + close;
            found.extend(node(key, std::iter::once(segment).collect(), Join::Space));
        }
    }
    found
}

/// The segments of a multi-line TOML string whose opening `triple` is on the
/// line `from`, with `rest` after it from the byte `at`, and the line after
/// the string.
fn toml_multiline(
    source: &str,
    lines: &[Range<usize>],
    from: usize,
    at: usize,
    rest: &str,
    triple: &str,
) -> (Vec<Range<usize>>, usize) {
    let mut segments = Vec::new();
    if let Some(close) = rest.find(triple) {
        segments.push(at..at + close);
        return (segments, from + 1);
    }
    if !rest.trim().is_empty() {
        segments.push(at..at + rest.len());
    }
    let mut index = from + 1;
    while let Some(line) = lines.get(index) {
        let written = text(source, line);
        if let Some(close) = written.find(triple) {
            segments.push(line.start..line.start + close);
            return (segments, index + 1);
        }
        segments.push(line.clone());
        index += 1;
    }
    (segments, index)
}
