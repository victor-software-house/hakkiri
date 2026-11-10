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
    pub span: Range<usize>,
    /// The nodes inside this one, in file order.
    pub children: Vec<Node>,
}

impl Node {
    /// Replaces the span of this node and of every node below it.
    pub(crate) fn map_spans(&mut self, map: &impl Fn(Range<usize>) -> Range<usize>) {
        self.span = map(self.span.clone());
        for child in &mut self.children {
            child.map_spans(map);
        }
    }
}

/// The tree of one file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tree {
    /// The `raw` roots: one for a Markdown or MDX file, one per comment for a
    /// source file.
    pub roots: Vec<Node>,
}
