//! The text a check reads, and where each byte of it lies in the file.

use std::borrow::Cow;
use std::ops::Range;

use crate::entity;
use crate::tree::{Join, Node, Reading};

/// How a span inside a node's text reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Replace {
    /// A code span reads as the one-letter word `x`.
    Code,
    /// The text of an excluded inline element reads as spaces of the same
    /// byte width.
    Blank,
}

/// How a character reference such as `&amp;` in a node's text reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Decode {
    /// As written.
    No,
    /// As the character it stands for, as in the text of an element.
    Text,
    /// As in the value of an attribute, where a name with no `;` that `=`
    /// follows stays as written.
    Attribute,
}

/// The text of one block, built from the segments of a node.
///
/// A line break between two segments is one line feed, and markup between
/// them is nothing.
/// Each code span is one `x`.
/// A character reference such as `&amp;` reads as the character it stands for,
/// and one of its characters is placed at the whole reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Text {
    content: String,
    /// The file byte of each byte of `text`, and one more entry that ends the
    /// last byte.
    origin: Vec<usize>,
    /// The file byte after each byte of `text`.
    ends: Vec<usize>,
    /// The bytes of `text` that read a code span.
    code: Vec<Range<usize>>,
}

/// The parts of a node that decide how its text reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Parts<'a> {
    /// The file ranges of the text, in file order.
    pub(crate) segments: &'a [Range<usize>],
    /// How each segment reads, or none when every segment reads as written.
    pub(crate) reads: &'a [Reading],
    /// The spans of the table rows the segments lie in, in file order.
    pub(crate) rows: &'a [Range<usize>],
    /// The file ranges that read otherwise, in file order and with no overlap.
    pub(crate) replacements: &'a [(Range<usize>, Replace)],
    /// How a character reference reads.
    pub(crate) decode: Decode,
}

impl Text {
    /// The text of `parts` in `source`.
    ///
    /// Returns `None` when a segment is not a slice of `source`.
    pub(crate) fn build(source: &str, parts: &Parts<'_>) -> Option<Self> {
        let mut built = Self {
            content: String::new(),
            origin: Vec::new(),
            ends: Vec::new(),
            code: Vec::new(),
        };
        let mut previous_end: Option<usize> = None;
        for (index, segment) in parts.segments.iter().enumerate() {
            let reading = parts.reads.get(index).copied().unwrap_or_default();
            if let Some(end) = previous_end {
                source.get(end..segment.start)?;
                // The replacements are in file order with no overlap, so the
                // ones that reach back to `end` and forward to the segment lie
                // together after a binary search.
                let from = parts
                    .replacements
                    .partition_point(|(range, _)| range.end < segment.start);
                let inside_code = parts
                    .replacements
                    .iter()
                    .skip(from)
                    .take_while(|(range, _)| range.start <= end)
                    .any(|(range, kind)| *kind == Replace::Code && segment.start <= range.end);
                let separator = match reading.before {
                    Join::Gap if inside_code => None,
                    Join::Gap => Node::separator(source, parts.rows, end, segment.start),
                    Join::Nothing => None,
                    Join::Space => Some(' '),
                    Join::Newline => Some('\n'),
                };
                if let Some(separator) = separator {
                    built.push(separator, end, end + 1);
                }
            }
            if reading.masked {
                built.copy(source, segment.clone(), parts.decode, true)?;
                previous_end = Some(segment.end);
                continue;
            }
            let mut cursor = segment.start;
            let first = parts
                .replacements
                .partition_point(|(range, _)| range.end <= segment.start);
            for (range, kind) in parts.replacements.iter().skip(first) {
                if segment.end <= range.start {
                    break;
                }
                let from = range.start.max(segment.start);
                let to = range.end.min(segment.end);
                built.copy(source, cursor..from, parts.decode, false)?;
                match kind {
                    Replace::Code => {
                        if range.start >= segment.start {
                            let at = built.content.len();
                            built.code.push(at..at + 1);
                            built.push('x', range.start, range.start + 1);
                        }
                    }
                    Replace::Blank => {
                        for (offset, c) in source.get(from..to)?.char_indices() {
                            for byte in 0..c.len_utf8() {
                                built.push(' ', from + offset + byte, from + offset + byte + 1);
                            }
                        }
                    }
                }
                cursor = to;
            }
            built.copy(source, cursor..segment.end, parts.decode, false)?;
            previous_end = Some(segment.end);
        }
        built.origin.push(previous_end.unwrap_or_default());
        Some(built)
    }

    /// Appends `c`, which came from the file bytes `start` to `end`.
    ///
    /// A character of more than one byte puts every byte at the same bytes of
    /// the file.
    fn push(&mut self, c: char, start: usize, end: usize) {
        self.content.push(c);
        for _ in 0..c.len_utf8() {
            self.origin.push(start);
            self.ends.push(end);
        }
    }

    /// Appends the file bytes `range`, each as itself.
    ///
    /// With `decode`, a character reference reads as its character, and with
    /// `mask`, every character reads as `*`.
    fn copy(
        &mut self,
        source: &str,
        range: Range<usize>,
        decode: Decode,
        mask: bool,
    ) -> Option<()> {
        if range.start >= range.end {
            return Some(());
        }
        let text = source.get(range.clone())?;
        if decode == Decode::No && !mask {
            self.content.push_str(text);
            for byte in range {
                self.origin.push(byte);
                self.ends.push(byte + 1);
            }
            return Some(());
        }
        let mut references = if decode == Decode::No {
            Vec::new()
        } else {
            entity::references(text, decode == Decode::Attribute)
        }
        .into_iter()
        .peekable();
        let mut skip_to = 0;
        for (offset, c) in text.char_indices() {
            if offset < skip_to {
                continue;
            }
            let start = range.start + offset;
            if let Some((reference, read)) = references.next_if(|(found, _)| found.start == offset)
            {
                skip_to = reference.end;
                for read in read.chars() {
                    self.push(
                        if mask { '*' } else { read },
                        range.start + reference.start,
                        range.start + reference.end,
                    );
                }
            } else if mask {
                self.push('*', start, start + c.len_utf8());
            } else {
                // A character of the file keeps a place for each of its bytes.
                self.content.push(c);
                for byte in start..start + c.len_utf8() {
                    self.origin.push(byte);
                    self.ends.push(byte + 1);
                }
            }
        }
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
        let last = self.ends.get(range.end - 1).copied()?;
        Some(start..last)
    }

    /// Whether the bytes `range` of the text overlap a code span.
    #[must_use]
    pub fn overlaps_code(&self, range: &Range<usize>) -> bool {
        let first = self.code.partition_point(|code| code.end <= range.start);
        self.code
            .get(first)
            .is_some_and(|code| code.start < range.end.max(range.start + 1))
    }

    /// The text as a pattern reads it: each code span is one `*`, a byte that
    /// no word and no space matches, as Vale masks it.
    ///
    /// A `*` and an `x` are one byte each, so every offset of the text holds.
    #[must_use]
    pub fn pattern_reading(&self) -> Cow<'_, str> {
        if self.code.is_empty() {
            return Cow::Borrowed(&self.content);
        }
        let mut masked = self.content.clone();
        for code in &self.code {
            masked.replace_range(code.clone(), "*");
        }
        Cow::Owned(masked)
    }

    /// Whether the first or the last byte of the bytes `range` of the text
    /// lies in a code span.
    ///
    /// A match that has text on both sides of a code span reads as a match
    /// across it, and one that starts or ends on a span does not.
    #[must_use]
    pub fn edge_in_code(&self, range: &Range<usize>) -> bool {
        if range.is_empty() {
            return self.overlaps_code(range);
        }
        let lies_in = |at: usize| {
            let first = self.code.partition_point(|code| code.end <= at);
            self.code.get(first).is_some_and(|code| code.start <= at)
        };
        lies_in(range.start) || lies_in(range.end - 1)
    }

    /// The part of this text in the bytes `range`, or `None` when `range` is
    /// not inside the text or not on character boundaries.
    #[must_use]
    pub fn slice(&self, range: Range<usize>) -> Option<Self> {
        let text = self.content.get(range.clone())?.to_owned();
        let origin = self.origin.get(range.start..=range.end)?.to_vec();
        let ends = self.ends.get(range.start..range.end)?.to_vec();
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
            ends,
            code,
        })
    }
}

#[cfg(test)]
mod tests;
