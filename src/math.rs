//! Math in a Markdown or MDX file: `$…$` inline and `$$…$$` display math.
//!
//! A port of `math.go` in [Vale] at `66bbb85`. Vale reads both as text that no
//! prose rule reads: an inline span renders as a code span, and a display block
//! as a code block.
//!
//! markdown-rs reads a `$` pair as math whatever stands around it, so `$5 and
//! $10` is math to it. Vale follows Pandoc: the opening `$` must be followed by
//! a character that is not a space, and the closing `$` must follow one and
//! not be followed by a digit. A span may wrap over lines of a paragraph, and
//! it ends where the paragraph does when nothing closes it.
//!
//! hakkiri finds the spans in the text of each paragraph, heading and table
//! cell that markdown-rs parsed, and writes the file again with the same
//! length.
//! The `$` of an inline span is a backtick, so it parses as a code span.
//! The lines of a display block are blank, so the paragraph ends before it and
//! starts again after it.
//! Every place in the tree is then a place in the file.
//!
//! [Vale]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/math.go

#[cfg(test)]
mod tests;

use std::ops::Range;

use markdown::mdast::Node as Mdast;

use crate::tree::{Node, Scope};

/// The math of a file.
#[derive(Debug, Default, Eq, PartialEq)]
pub(crate) struct Math {
    /// The bytes of each inline span, `$` delimiters included.
    pub(crate) inline: Vec<Range<usize>>,
    /// The bytes of each line of a display block that the block holds, from
    /// the first byte of its text to the end of the line.
    pub(crate) display: Vec<Range<usize>>,
    /// The bytes of each run of backticks that opens no code span, in a
    /// paragraph that holds a span.
    pub(crate) loose: Vec<Range<usize>>,
}

impl Math {
    pub(crate) const fn is_empty(&self) -> bool {
        self.inline.is_empty() && self.display.is_empty()
    }
}

/// Whether `byte` is whitespace to Vale: a space, a tab, a line feed, a
/// vertical tab, a form feed or a carriage return.
const fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// Whether the `$` at `at` in `text` can close a span, whichever `$` opens it.
///
/// It must not be escaped, must follow a character that is not a space, and
/// must not be followed by a digit. The first byte of a line follows a line
/// feed, so it never closes.
fn closes(text: &[u8], at: usize) -> bool {
    if text.get(at) != Some(&b'$') {
        return false;
    }
    let before = at.checked_sub(1).and_then(|index| text.get(index));
    if before == Some(&b'\\') || before.is_none_or(|byte| is_space(*byte)) {
        return false;
    }
    !text.get(at + 1).is_some_and(u8::is_ascii_digit)
}

/// The end of the code span that starts with the run of `n` backticks at
/// `from`, or `None` when no run of `n` closes it.
fn code_span_end(text: &[u8], from: usize, n: usize) -> Option<usize> {
    let mut at = from + n;
    while at < text.len() {
        if text.get(at) == Some(&b'`') {
            let run = text
                .get(at..)
                .unwrap_or_default()
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            if run == n {
                return Some(at + run);
            }
            at += run;
        } else {
            at += 1;
        }
    }
    None
}

/// The end of the `{…}` expression that starts at `from`, braces nested.
fn expression_end(text: &[u8], from: usize) -> Option<usize> {
    let mut depth = 0_usize;
    for (offset, byte) in text.get(from..)?.iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(from + offset + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// The inline spans in `text`, which starts at the byte `base` of the file,
/// as byte ranges of the file, and the runs of backticks that open no code
/// span. `braces` says that a `{…}` expression holds no math, as in MDX.
///
/// The closers and the end of an HTML tag are looked up in lists made once, so
/// a paragraph of unclosed `$` or `<` costs no more than its length.
fn inline_spans(text: &str, base: usize, braces: bool) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let bytes = text.as_bytes();
    let closers: Vec<usize> = (0..bytes.len()).filter(|at| closes(bytes, *at)).collect();
    let last_tag_end = bytes.iter().rposition(|byte| *byte == b'>');
    let mut found = Vec::new();
    let mut loose = Vec::new();
    let mut at = 0;
    while let Some(&byte) = bytes.get(at) {
        match byte {
            b'\\' => at += 2,
            b'`' => {
                let run = bytes
                    .get(at..)
                    .unwrap_or_default()
                    .iter()
                    .take_while(|byte| **byte == b'`')
                    .count();
                at = code_span_end(bytes, at, run).unwrap_or_else(|| {
                    loose.push(base + at..base + at + run);
                    at + run
                });
            }
            b'<' if bytes.get(at + 1).is_some_and(|next| {
                next.is_ascii_alphabetic() || matches!(next, b'/' | b'!' | b'?')
            }) =>
            {
                at = last_tag_end
                    .filter(|end| *end > at)
                    .and_then(|_| {
                        bytes
                            .get(at..)
                            .and_then(|rest| rest.iter().position(|byte| *byte == b'>'))
                    })
                    .map_or(at + 1, |close| at + close + 1);
            }
            b'{' if braces => at = expression_end(bytes, at).unwrap_or(at + 1),
            b'$' => {
                let opens = bytes
                    .get(at + 1)
                    .is_some_and(|next| *next != b'$' && !is_space(*next));
                if opens && let Some(close) = closer(&closers, at) {
                    found.push(base + at..base + close + 1);
                    at = close + 1;
                } else {
                    at += 1;
                }
            }
            _ => at += 1,
        }
    }
    (found, loose)
}

/// The index of the `$` that closes the span that opens at `open`: the first
/// of `closers` after the byte that follows `open`.
fn closer(closers: &[usize], open: usize) -> Option<usize> {
    let first = closers.partition_point(|at| *at < open + 2);
    closers.get(first).copied()
}

/// The end of the part of `rest` before a closing `$$`: the index of the `$$`
/// that ends the line, with trailing blanks and at most one `{…}` after it,
/// or `None`.
fn display_closer(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes.get(end - 1).is_some_and(|byte| is_space(*byte)) {
        end -= 1;
    }
    if end > 0 && bytes.get(end - 1) == Some(&b'}') {
        let open = bytes
            .get(..end - 1)?
            .iter()
            .rposition(|byte| *byte == b'{')?;
        end = open;
        while end > 0 && bytes.get(end - 1).is_some_and(|byte| is_space(*byte)) {
            end -= 1;
        }
    }
    if end < 2 || bytes.get(end - 1) != Some(&b'$') || bytes.get(end - 2) != Some(&b'$') {
        return None;
    }
    if end > 2 && bytes.get(end - 3) == Some(&b'\\') {
        return None;
    }
    Some(end - 2)
}

/// The text of a line that the container around a paragraph does not hold:
/// the indentation and the `>` markers that start a line after the first.
fn content_start(line: &str, first: bool) -> usize {
    if first {
        return 0;
    }
    line.len() - line.trim_start_matches([' ', '\t', '>']).len()
}

/// A container around a paragraph, outermost first.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Container {
    /// A list item whose content starts at this column, counted from 0.
    Item(usize),
    /// A block quote.
    Quote,
}

/// The column where the content of the list item that starts at the byte
/// `start` of `source` begins, counted from 0: after the marker and the one to
/// four spaces that follow it.
pub(crate) fn item_content(source: &str, start: usize, column: usize) -> usize {
    let rest = source.get(start..).unwrap_or_default();
    let marker = rest
        .find(|c: char| !c.is_ascii_digit())
        .map_or(1, |digits| digits + 1);
    let after = rest.get(marker..).unwrap_or_default();
    let spaces = after.len() - after.trim_start_matches(' ').len();
    let blank = after
        .get(spaces..)
        .is_none_or(|text| text.starts_with('\n'));
    let padding = if blank || spaces > 4 { 1 } else { spaces };
    column - 1 + marker + padding
}

/// The width in columns that the indentation of `line` adds to the content of
/// its containers, which are `containers`.
///
/// A line that lacks the marker or the indentation of a container continues
/// the paragraph lazily, and holds what it has.
/// A tab runs to the next multiple of four.
fn indent_in(line: &str, containers: &[Container]) -> usize {
    let bytes = line.as_bytes();
    let mut at = 0;
    let mut column = 0;
    let skip_spaces = |at: &mut usize, column: &mut usize, most: usize| {
        while *column < most
            && bytes
                .get(*at)
                .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        {
            *column += if bytes.get(*at) == Some(&b'\t') {
                4 - *column % 4
            } else {
                1
            };
            *at += 1;
        }
    };
    for container in containers {
        match container {
            Container::Item(content) => skip_spaces(&mut at, &mut column, *content),
            Container::Quote => {
                let start = column;
                skip_spaces(&mut at, &mut column, start + 3);
                if bytes.get(at) == Some(&b'>') {
                    at += 1;
                    column += 1;
                    if bytes.get(at) == Some(&b' ') {
                        at += 1;
                        column += 1;
                    }
                }
            }
        }
    }
    let start = column;
    skip_spaces(&mut at, &mut column, usize::MAX);
    column - start
}

/// The lines of the display blocks in the paragraph that `lines` are the lines
/// of, as ranges of the indexes of `lines`.
///
/// A line that starts with `$$` opens a block, even in the middle of a
/// paragraph, and a line that ends with `$$` closes it, or a blank line, or the
/// end of the paragraph.
fn display_blocks(
    text: &str,
    lines: &[Range<usize>],
    containers: &[Container],
) -> Vec<Range<usize>> {
    let mut blocks = Vec::new();
    let mut index = 0;
    while let Some(line) = lines.get(index) {
        let written = text.get(line.clone()).unwrap_or_default();
        let start = content_start(written, index == 0);
        let body = written.get(start..).unwrap_or_default();
        // The text of the paragraph starts at its first character, so only a
        // later line can be indented.
        let indent = if index == 0 {
            0
        } else {
            indent_in(written, containers)
        };
        let run = body.bytes().take_while(|byte| *byte == b'$').count();
        if indent > 3 || run < 2 {
            index += 1;
            continue;
        }
        let after = body.get(run..).unwrap_or_default();
        let mut last = index;
        if display_closer(after).is_none() {
            while let Some(next) = lines.get(last + 1) {
                let next_text = text.get(next.clone()).unwrap_or_default();
                let inside = next_text
                    .get(content_start(next_text, false)..)
                    .unwrap_or_default();
                if inside.trim().is_empty() {
                    break;
                }
                last += 1;
                if display_closer(inside).is_some() {
                    break;
                }
            }
        }
        blocks.push(index..last + 1);
        index = last + 1;
    }
    blocks
}

/// The math in the paragraph, heading or table cell whose text is `text`, from
/// the byte `base` of the file.
///
/// Only a paragraph holds display blocks.
fn in_container(
    text: &str,
    base: usize,
    blocks_allowed: bool,
    braces: bool,
    containers: &[Container],
    math: &mut Math,
) {
    let mut lines = Vec::new();
    let mut at = 0;
    for line in text.split('\n') {
        lines.push(at..at + line.len());
        at += line.len() + 1;
    }
    let blocks = if blocks_allowed {
        display_blocks(text, &lines, containers)
    } else {
        Vec::new()
    };
    let mut next = 0;
    let group = |from: usize, to: usize, math: &mut Math| {
        let (Some(first), Some(last)) = (lines.get(from), lines.get(to.wrapping_sub(1))) else {
            return;
        };
        if from < to
            && let Some(part) = text.get(first.start..last.end)
        {
            let (spans, loose) = inline_spans(part, base + first.start, braces);
            if !spans.is_empty() {
                math.loose.extend(loose);
            }
            math.inline.extend(spans);
        }
    };
    for block in &blocks {
        group(next, block.start, math);
        for index in block.clone() {
            let Some(line) = lines.get(index) else {
                continue;
            };
            let written = text.get(line.clone()).unwrap_or_default();
            let start = line.start + content_start(written, index == 0);
            math.display.push(base + start..base + line.end);
        }
        next = block.end;
    }
    group(next, lines.len(), math);
}

/// The math in `root`, a parse of `source`: in each paragraph, heading and
/// table cell. `braces` says that a `{…}` expression holds no math.
pub(crate) fn find(root: &Mdast, source: &str, braces: bool) -> Math {
    let mut math = Math::default();
    // Each container that holds a node is one entry, with the entry of the
    // container around it.
    let mut chains: Vec<(Option<usize>, Container)> = Vec::new();
    let mut pending = vec![(root, None)];
    while let Some((node, chain)) = pending.pop() {
        if matches!(
            node,
            Mdast::Paragraph(_) | Mdast::Heading(_) | Mdast::TableCell(_)
        ) && let Some(position) = node.position()
            && let Some(text) = source.get(position.start.offset..position.end.offset)
        {
            let paragraph = matches!(node, Mdast::Paragraph(_));
            let containers = containers_of(&chains, chain);
            in_container(
                text,
                position.start.offset,
                paragraph,
                braces,
                &containers,
                &mut math,
            );
            continue;
        }
        let chain = container_of(node, source).map_or(chain, |container| {
            chains.push((chain, container));
            Some(chains.len() - 1)
        });
        if let Some(children) = node.children() {
            pending.extend(children.iter().rev().map(|child| (child, chain)));
        }
    }
    math.inline.sort_by_key(|span| span.start);
    math.display.sort_by_key(|line| line.start);
    math.loose.sort_by_key(|run| run.start);
    math
}

/// The container `node` is, when it is one that indents the lines it holds.
fn container_of(node: &Mdast, source: &str) -> Option<Container> {
    match node {
        Mdast::ListItem(_) => node.position().map(|position| {
            Container::Item(item_content(
                source,
                position.start.offset,
                position.start.column,
            ))
        }),
        Mdast::Blockquote(_) => Some(Container::Quote),
        _ => None,
    }
}

/// The containers from the entry `chain` of `chains` outwards, outermost first.
fn containers_of(
    chains: &[(Option<usize>, Container)],
    mut chain: Option<usize>,
) -> Vec<Container> {
    let mut containers = Vec::new();
    while let Some((outer, container)) = chain.and_then(|index| chains.get(index)) {
        containers.push(*container);
        chain = *outer;
    }
    containers.reverse();
    containers
}

/// Gives the code span that each inline span became its `$` delimiters, which
/// Vale's code span holds.
///
/// A span that touches another code span or span is one code span with it, so
/// the delimiter at each end is looked for on its own.
/// Only the segments of the node change: its span stays the inner text, which
/// the text around it reads as one word.
pub(crate) fn include_delimiters(nodes: &mut [Node], inline: &[Range<usize>]) {
    let mut pending: Vec<&mut Node> = nodes.iter_mut().collect();
    while let Some(node) = pending.pop() {
        if node.scope == Scope::Code
            && let (Some(first), Some(last)) = (node.segments.first(), node.segments.last())
        {
            let opens = first
                .start
                .checked_sub(1)
                .and_then(|from| inline.binary_search_by_key(&from, |span| span.start).ok())
                .and_then(|index| inline.get(index))
                .map(|span| span.start);
            let closes = inline
                .binary_search_by_key(&(last.end + 1), |span| span.end)
                .ok()
                .and_then(|index| inline.get(index))
                .map(|span| span.end);
            if let (Some(start), Some(first)) = (opens, node.segments.first_mut()) {
                first.start = start;
            }
            if let (Some(end), Some(last)) = (closes, node.segments.last_mut()) {
                last.end = end;
            }
        }
        pending.extend(node.children.iter_mut());
    }
}

/// `source` with the same length, where each inline span reads as a code span
/// and each line of a display block is blank.
///
/// A backtick inside a span is an apostrophe, since a code span would end at
/// it. So is a run of backticks that opens no code span, in a paragraph with a
/// span: left alone, it would pair with the backtick of a span, or join it into
/// a longer run. The tree reads the text from the file, so the change shows
/// nowhere.
pub(crate) fn mask(source: &str, math: &Math) -> String {
    let mut bytes = source.as_bytes().to_vec();
    for run in &math.loose {
        for byte in bytes.get_mut(run.clone()).unwrap_or_default() {
            *byte = b'\'';
        }
    }
    for line in &math.display {
        for byte in bytes.get_mut(line.clone()).unwrap_or_default() {
            *byte = b' ';
        }
    }
    for span in &math.inline {
        for at in span.start + 1..span.end - 1 {
            if let Some(byte) = bytes.get_mut(at).filter(|byte| **byte == b'`') {
                *byte = b'\'';
            }
        }
        for at in [span.start, span.end - 1] {
            if let Some(byte) = bytes.get_mut(at) {
                *byte = b'`';
            }
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| source.to_owned())
}
