//! The text a check reads, and where each byte of it lies in the file.

use std::ops::Range;

use crate::tree::Node;

/// How a span inside a node's text reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Replace {
    /// A code span reads as the one-letter word `x`.
    Code,
    /// The text of an excluded inline element reads as spaces of the same
    /// byte width.
    Blank,
}

/// The text of one block, built from the segments of a node.
///
/// A line break between two segments is one line feed, and markup between
/// them is nothing.
/// Each code span is one `x`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Text {
    content: String,
    /// The file byte of each byte of `text`, and one more entry that ends the
    /// last byte.
    origin: Vec<usize>,
    /// The bytes of `text` that read a code span.
    code: Vec<Range<usize>>,
}

impl Text {
    /// The text of `segments` in `source`.
    ///
    /// `rows` are the spans of the table rows the segments lie in, in file
    /// order, and `replacements` are the file ranges that read otherwise, in
    /// file order and with no overlap.
    /// Returns `None` when a segment is not a slice of `source`.
    pub(crate) fn build(
        source: &str,
        segments: &[Range<usize>],
        rows: &[Range<usize>],
        replacements: &[(Range<usize>, Replace)],
    ) -> Option<Self> {
        let mut built = Self {
            content: String::new(),
            origin: Vec::new(),
            code: Vec::new(),
        };
        let mut previous_end: Option<usize> = None;
        for segment in segments {
            if let Some(end) = previous_end {
                source.get(end..segment.start)?;
                let inside_code = replacements.iter().any(|(range, kind)| {
                    *kind == Replace::Code && range.start <= end && segment.start <= range.end
                });
                if !inside_code
                    && let Some(separator) = Node::separator(source, rows, end, segment.start)
                {
                    built.content.push(separator);
                    built.origin.push(end);
                }
            }
            let mut cursor = segment.start;
            for (range, kind) in replacements {
                if range.end <= segment.start || segment.end <= range.start {
                    continue;
                }
                let from = range.start.max(segment.start);
                let to = range.end.min(segment.end);
                built.copy(source, cursor..from)?;
                match kind {
                    Replace::Code => {
                        if range.start >= segment.start {
                            let at = built.content.len();
                            built.code.push(at..at + 1);
                            built.content.push('x');
                            built.origin.push(range.start);
                        }
                    }
                    Replace::Blank => {
                        for (offset, c) in source.get(from..to)?.char_indices() {
                            for byte in 0..c.len_utf8() {
                                built.content.push(' ');
                                built.origin.push(from + offset + byte);
                            }
                        }
                    }
                }
                cursor = to;
            }
            built.copy(source, cursor..segment.end)?;
            previous_end = Some(segment.end);
        }
        built.origin.push(previous_end.unwrap_or_default());
        Some(built)
    }

    /// Appends the file bytes `range`, each as itself.
    fn copy(&mut self, source: &str, range: Range<usize>) -> Option<()> {
        if range.is_empty() {
            return Some(());
        }
        self.content.push_str(source.get(range.clone())?);
        self.origin.extend(range);
        Some(())
    }

    /// The text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// The file byte that byte `index` of the text came from, or a position at
    /// the end of the text when `index` is its length.
    #[must_use]
    pub fn file_offset(&self, index: usize) -> Option<usize> {
        self.origin.get(index).copied()
    }

    /// The file bytes of the bytes `range` of the text.
    #[must_use]
    pub fn file_range(&self, range: Range<usize>) -> Option<Range<usize>> {
        let start = self.file_offset(range.start)?;
        if range.is_empty() {
            return Some(start..start);
        }
        let last = self.file_offset(range.end - 1)?;
        Some(start..last + 1)
    }

    /// Whether the bytes `range` of the text overlap a code span.
    #[must_use]
    pub fn overlaps_code(&self, range: &Range<usize>) -> bool {
        self.code
            .iter()
            .any(|code| code.start < range.end.max(range.start + 1) && range.start < code.end)
    }

    /// The part of this text in the bytes `range`, or `None` when `range` is
    /// not inside the text or not on character boundaries.
    #[must_use]
    pub fn slice(&self, range: Range<usize>) -> Option<Self> {
        let text = self.content.get(range.clone())?.to_owned();
        let origin = self.origin.get(range.start..=range.end)?.to_vec();
        let code = self
            .code
            .iter()
            .filter(|code| code.start < range.end && range.start < code.end)
            .map(|code| {
                code.start.max(range.start) - range.start..code.end.min(range.end) - range.start
            })
            .collect();
        Some(Self {
            content: text,
            origin,
            code,
        })
    }
}
