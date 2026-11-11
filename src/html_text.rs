//! The text of an HTML block of a Markdown file, as nodes of the tree.
//!
//! [`crate::html`] reads the block as Vale does and gives the runs of text,
//! with byte ranges in the text of the block.
//! The block sits in the file with the markers of the quotes and list items
//! around it taken out, so this module maps each range back to the file, one
//! line at a time.

use std::ops::Range;

use crate::html::{self, Block, Inline, Place};
use crate::tree::{Join, Node, Reading, Scope};

/// One line of the block, and where it lies in the file.
#[derive(Debug)]
struct Line {
    /// The bytes of the line in the text of the block, without its `\r`.
    value: Range<usize>,
    /// The file byte of `value.start`.
    source: usize,
}

/// The lines of an HTML block, mapped to the file.
#[derive(Debug)]
struct Lines {
    lines: Vec<Line>,
}

impl Lines {
    /// Matches the lines of `html`, the text of the block, to the lines of
    /// `span` in `source`.
    ///
    /// A line of the file holds the block's line, after the markers of the
    /// quotes and list items around it.
    /// Returns `None` when a line of the file does not end with the block's.
    fn new(source: &str, span: &Range<usize>, html: &str) -> Option<Self> {
        let slice = source.get(span.clone())?;
        let mut source_lines = slice.split('\n');
        let mut lines = Vec::new();
        let mut value_at = 0;
        let mut source_at = span.start;
        for line in html.split('\n') {
            let in_file = source_lines.next()?;
            let kept = line.trim_end_matches('\r');
            let in_file_kept = in_file.trim_end_matches('\r');
            if !in_file_kept.ends_with(kept) {
                return None;
            }
            lines.push(Line {
                value: value_at..value_at + kept.len(),
                source: source_at + (in_file_kept.len() - kept.len()),
            });
            value_at += line.len() + 1;
            source_at += in_file.len() + 1;
        }
        Some(Self { lines })
    }

    /// The ranges of the file that hold the bytes `range` of the block, one
    /// for each line.
    fn map(&self, range: &Range<usize>) -> Vec<Range<usize>> {
        self.lines
            .iter()
            .filter_map(|line| {
                let start = range.start.max(line.value.start);
                let end = range.end.min(line.value.end);
                (start < end).then(|| {
                    let offset = line.source - line.value.start;
                    start + offset..end + offset
                })
            })
            .collect()
    }
}

const fn inline_scope(kind: Inline) -> Scope {
    match kind {
        Inline::Link => Scope::Link,
        Inline::Strong => Scope::Strong,
        Inline::Emphasis => Scope::Emphasis,
        Inline::Code => Scope::Code,
    }
}

/// The segments of some pieces of a block, and how each reads.
struct Run {
    segments: Vec<Range<usize>>,
    reads: Vec<Reading>,
}

impl Lines {
    /// The segments of the pieces `pieces` of `block`, a segment for each line
    /// of each piece.
    fn run(&self, block: &Block, pieces: Range<usize>) -> Run {
        let mut run = Run {
            segments: Vec::new(),
            reads: Vec::new(),
        };
        for piece in block.pieces.get(pieces).unwrap_or_default() {
            for (line, range) in self.map(&piece.range).into_iter().enumerate() {
                run.segments.push(range);
                run.reads.push(Reading {
                    before: if line == 0 {
                        piece.reading.before
                    } else {
                        Join::Gap
                    },
                    ..piece.reading
                });
            }
        }
        run
    }

    /// The node for one block of text, or `None` when it holds no text, or
    /// hakkiri has no scope for it.
    fn block(&self, block: &Block, first: bool) -> Option<Node> {
        let whole = self.run(block, 0..block.pieces.len());
        let mut children: Vec<Node> = block
            .captures
            .iter()
            .filter_map(|capture| {
                let inside = self.run(block, capture.pieces.clone());
                Node::from_segments(inline_scope(capture.kind), inside.segments, Vec::new())
                    .map(|node| node.with_reads(inside.reads))
            })
            .collect();
        children.sort_by_key(|child| child.span.start);
        let leaf = |scope, children: Vec<Node>| {
            Node::from_segments(scope, whole.segments.clone(), children)
                .map(|node| node.with_reads(whole.reads.clone()))
        };
        let node = match block.place {
            Place::Paragraph => leaf(Scope::Paragraph, children),
            Place::Heading(depth) => leaf(Scope::heading(depth)?, children),
            Place::Header => leaf(Scope::TableHeader, children),
            Place::Cell => leaf(Scope::TableCell, children),
            Place::TableCaption => leaf(Scope::TableCaption, children),
            Place::FigureCaption => leaf(Scope::FigureCaption, children),
            Place::Meta => leaf(Scope::Meta, Vec::new()),
            Place::List | Place::Blockquote => {
                let inner = Node {
                    first: true,
                    ..leaf(Scope::Paragraph, children)?
                };
                let container = if block.place == Place::List {
                    Scope::List
                } else {
                    Scope::Blockquote
                };
                Node::from_segments(container, whole.segments.clone(), vec![inner])
                    .map(|node| node.with_reads(whole.reads.clone()))
            }
            Place::Skipped => None,
        };
        node.map(|node| Node { first, ..node })
    }
}

/// The nodes for the text of the HTML block `html`, which `span` covers in
/// `source`, or `None` when the lines of the block cannot be matched to the
/// file.
///
/// `first` is whether the block is the first thing inside the item or quote
/// around it, which gives its first text the scope of that item or quote.
pub(crate) fn nodes(
    source: &str,
    span: &Range<usize>,
    html: &str,
    first: bool,
) -> Option<Vec<Node>> {
    let walk = html::walk(html);
    if walk.blocks.is_empty() && walk.alts.is_empty() && walk.descriptions.is_empty() {
        return Some(Vec::new());
    }
    let lines = Lines::new(source, span, html)?;
    let mut nodes = Vec::new();
    for (index, block) in walk.blocks.iter().enumerate() {
        nodes.extend(lines.block(block, first && index == 0));
    }
    for (scope, ranges) in [
        (Scope::Alt, &walk.alts),
        (Scope::Description, &walk.descriptions),
    ] {
        for range in ranges {
            nodes.extend(Node::from_segments(scope, lines.map(range), Vec::new()));
        }
    }
    nodes.sort_by_key(|node| node.span.start);
    Some(nodes)
}
