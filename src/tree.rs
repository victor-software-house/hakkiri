//! The document tree: scopes, nodes and spans.

use std::fmt;
use std::ops::Range;

/// The scope of a node, named as a rule's `scope` names it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Scope {
    /// A Markdown paragraph.
    Paragraph,
    /// A heading of depth 1.
    Heading1,
    /// A heading of depth 2.
    Heading2,
    /// A heading of depth 3.
    Heading3,
    /// A heading of depth 4.
    Heading4,
    /// A heading of depth 5.
    Heading5,
    /// A heading of depth 6.
    Heading6,
    /// A block quote.
    Blockquote,
    /// A list item.
    List,
    /// A table row, the header row included.
    TableRow,
    /// A cell in the first row of a table.
    TableHeader,
    /// A cell in a row below the first.
    TableCell,
    /// A link, a link reference or a bare URL.
    Link,
    /// A code span.
    Code,
    /// Strong text.
    Strong,
    /// Emphasized text.
    Emphasis,
    /// The alt text of an image.
    Alt,
    /// The text of a file, or of one comment in a source file.
    Raw,
    /// A `//` comment.
    CommentLine,
    /// A `/* */` comment.
    CommentBlock,
    /// A Rust `///` or `//!` comment.
    CommentDocLine,
    /// A `/** */` comment, or a Rust `/*! */` comment.
    CommentDocBlock,
}

impl Scope {
    /// The lowercase dotted name a rule's `scope` uses, such as `heading.h1`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Heading1 => "heading.h1",
            Self::Heading2 => "heading.h2",
            Self::Heading3 => "heading.h3",
            Self::Heading4 => "heading.h4",
            Self::Heading5 => "heading.h5",
            Self::Heading6 => "heading.h6",
            Self::Blockquote => "blockquote",
            Self::List => "list",
            Self::TableRow => "table.row",
            Self::TableHeader => "table.header",
            Self::TableCell => "table.cell",
            Self::Link => "link",
            Self::Code => "code",
            Self::Strong => "strong",
            Self::Emphasis => "emphasis",
            Self::Alt => "alt",
            Self::Raw => "raw",
            Self::CommentLine => "text.comment.line",
            Self::CommentBlock => "text.comment.block",
            Self::CommentDocLine => "text.comment.doc.line",
            Self::CommentDocBlock => "text.comment.doc.block",
        }
    }

    /// The scope of a heading of `depth`, or `None` outside 1 to 6.
    pub(crate) const fn heading(depth: u8) -> Option<Self> {
        match depth {
            1 => Some(Self::Heading1),
            2 => Some(Self::Heading2),
            3 => Some(Self::Heading3),
            4 => Some(Self::Heading4),
            5 => Some(Self::Heading5),
            6 => Some(Self::Heading6),
            0 | 7..=u8::MAX => None,
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One node of the tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    /// The scope of the node.
    pub scope: Scope,
    /// The byte span of the node's text in the original file.
    /// It runs from the first byte of its first segment to the end of its
    /// last.
    ///
    /// The span also covers what lies between the segments: the markers
    /// between the lines of a comment, and syntax that is not prose.
    /// Use it for a position, and read the text through `segments`.
    pub span: Range<usize>,
    /// The byte ranges of the node's text in the original file, in file order.
    ///
    /// A gap between two segments holds what is not part of the text.
    /// It is a comment marker, or the indentation that a comment strips.
    /// It can also be syntax that is not prose, such as a code block, an HTML
    /// tag or an MDX expression.
    /// A line of a comment with no text gives an empty range.
    pub segments: Vec<Range<usize>>,
    /// The nodes inside this one, in file order.
    pub children: Vec<Node>,
}

impl Node {
    /// A node whose text is the one range `span`.
    pub(crate) fn new(scope: Scope, span: Range<usize>, children: Vec<Self>) -> Self {
        Self {
            scope,
            segments: vec![span.clone()],
            span,
            children,
        }
    }

    /// A node with `segments`, or `None` when there is no segment.
    pub(crate) fn from_segments(
        scope: Scope,
        segments: Vec<Range<usize>>,
        children: Vec<Self>,
    ) -> Option<Self> {
        let span = segments.first()?.start..segments.last()?.end;
        Some(Self {
            scope,
            span,
            segments,
            children,
        })
    }

    /// The text of this node in `source`: its segments in order.
    ///
    /// A line feed separates two segments with a line break between them,
    /// and one space separates two cells of a table row.
    /// Any other gap reads as nothing.
    /// Returns `None` when a segment is not a slice of `source`.
    #[must_use]
    pub fn text(&self, source: &str) -> Option<String> {
        let mut text = String::new();
        let mut previous_end = None;
        for segment in &self.segments {
            if let Some(end) = previous_end {
                let gap = source.get(end..segment.start)?;
                if gap.contains('\n') {
                    text.push('\n');
                } else if gap.contains('|') && gap.chars().all(|c| c == '|' || c.is_whitespace()) {
                    text.push(' ');
                }
            }
            text.push_str(source.get(segment.clone())?);
            previous_end = Some(segment.end);
        }
        Some(text)
    }

    /// Replaces the segments and the span of this node and of every node below
    /// it. `map` gives the segments of a range from its current place.
    ///
    /// The walk keeps its own stack, so the depth of the tree costs no call
    /// stack.
    pub(crate) fn map_segments(&mut self, map: &impl Fn(Range<usize>) -> Vec<Range<usize>>) {
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            node.segments = node.segments.iter().cloned().flat_map(map).collect();
            if let (Some(first), Some(last)) = (node.segments.first(), node.segments.last()) {
                node.span = first.start..last.end;
            }
            pending.extend(node.children.iter_mut());
        }
    }
}

/// The tree of one file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tree {
    /// The `raw` roots: one for a Markdown or MDX file, one per comment group
    /// for a source file.
    pub roots: Vec<Node>,
}
