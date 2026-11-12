//! The comments of a fenced code block, as nodes of the Markdown or MDX file
//! that holds it.
//!
//! The info string of a fence names a language through langdex. A fence of a
//! language whose comments hakkiri reads is parsed with that language's
//! grammar, as a source file is, and each comment becomes a node at its place
//! in the file.
//!
//! The Markdown parser gives the code of a fence without the indentation and
//! the container markers that sit before each line in the file. Each line of
//! that code must end the line of the file it came from, so a byte of the code
//! has a place in the file. A fence with a line that does not map gives no
//! nodes.

use std::iter;
use std::ops::Range;

use markdown::mdast::Code;

use crate::comments::{self, Dialect};
use crate::language::Language;
use crate::tree::Node;

/// One line of the code of a fence, and where it ends in the file.
#[derive(Debug)]
struct Line {
    /// The byte of the code where the line starts.
    code: usize,
    /// How many bytes of the code the line holds, without its line feed.
    len: usize,
    /// How many of those bytes the file holds: a carriage return that the
    /// parser keeps in the code is no byte of the text.
    kept: usize,
    /// The byte of the file where the line starts.
    file: usize,
}

/// Where each line of the code of a fence is in the file.
#[derive(Debug)]
struct Placement {
    lines: Vec<Line>,
}

impl Placement {
    /// The lines of `code`, the code of the fence that spans `span` of
    /// `source`, or `None` when a line of it does not end the line of the fence
    /// it should.
    ///
    /// The first line of the code is the line after the opening fence.
    fn new(source: &str, span: &Range<usize>, code: &str) -> Option<Self> {
        let mut at = span.start;
        let mut written = source
            .get(span.clone())?
            .split('\n')
            .map(|line| {
                let start = at;
                at += line.len() + 1;
                (start, line.strip_suffix('\r').unwrap_or(line))
            })
            .skip(1);
        let mut lines = Vec::new();
        let mut start = 0;
        for line in code.split('\n') {
            let (file, text) = written.next()?;
            let kept = line.strip_suffix('\r').unwrap_or(line);
            if !text.ends_with(kept) {
                return None;
            }
            lines.push(Line {
                code: start,
                len: line.len(),
                kept: kept.len(),
                file: file + text.len() - kept.len(),
            });
            start += line.len() + 1;
        }
        Some(Self { lines })
    }

    /// The byte of the file for the byte `at` of the code.
    fn place(&self, at: usize) -> usize {
        let index = self.lines.partition_point(|line| line.code + line.len < at);
        self.lines
            .get(index)
            .or_else(|| self.lines.last())
            .map_or(at, |line| {
                line.file + at.saturating_sub(line.code).min(line.kept)
            })
    }

    /// The ranges of the file for the range of the code `range`, one for each
    /// line it holds. An empty range gives the empty range at its place.
    fn ranges(&self, range: Range<usize>) -> Vec<Range<usize>> {
        if range.is_empty() {
            let at = self.place(range.start);
            return iter::once(at..at).collect();
        }
        self.lines
            .iter()
            .filter_map(|line| {
                let start = range.start.max(line.code);
                let end = range.end.min(line.code + line.len);
                let from = line.file + start.saturating_sub(line.code).min(line.kept);
                let to = line.file + end.saturating_sub(line.code).min(line.kept);
                (start < end && from < to).then_some(from..to)
            })
            .collect()
    }
}

/// The word of the info string `lang` that names the language: all of it up to
/// a comma, since rustdoc writes `rust,no_run`.
fn language_word(lang: &str) -> &str {
    lang.split_once(',').map_or(lang, |(word, _)| word)
}

/// The nodes of the comments in the fenced code block `code`, which spans
/// `span` of `source`.
///
/// There are none when hakkiri reads no comments of its language, when the
/// block has none, and when its code does not map line for line onto `source`.
///
/// Each comment node, and every node inside it, has the language of the fence.
pub(crate) fn nodes(source: &str, span: &Range<usize>, code: &Code) -> Vec<Node> {
    let Some((language, _)) = code
        .lang
        .as_deref()
        .map(language_word)
        .and_then(langdex::by_fence)
    else {
        return Vec::new();
    };
    let Some(dialect) = Language::of(language).and_then(Dialect::of) else {
        return Vec::new();
    };
    if code.value.is_empty() {
        return Vec::new();
    }
    let Some(placement) = Placement::new(source, span, &code.value) else {
        return Vec::new();
    };
    // The masks of doc links hold offsets of the fence's code, which the file
    // does not have, so a fence has none.
    let Ok((roots, _masks)) = comments::roots(dialect, &code.value, &[], &[]) else {
        return Vec::new();
    };
    let mut nodes: Vec<Node> = roots.into_iter().flat_map(|root| root.children).collect();
    for comment in &mut nodes {
        let closes = placement.place(comment.closes);
        comment.map_segments(&|range| placement.ranges(range));
        comment.closes = closes;
        comment.language = Some(language);
        // The nodes inside a comment hold the offsets of the comment's text,
        // which the file does not have: each closes where it ends.
        let mut pending: Vec<&mut Node> = comment.children.iter_mut().collect();
        while let Some(node) = pending.pop() {
            node.language = Some(language);
            node.closes = node.span.end;
            node.flushes.clear();
            pending.extend(node.children.iter_mut());
        }
    }
    nodes
}

#[cfg(test)]
mod tests;
