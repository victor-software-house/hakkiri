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
    /// A heading of depth 0, which an HTML block can make with `<h0>`.
    Heading0,
    /// A heading of depth 7, which an HTML block can make with `<h7>`.
    Heading7,
    /// A heading of depth 8, which an HTML block can make with `<h8>`.
    Heading8,
    /// A heading of depth 9, which an HTML block can make with `<h9>`.
    Heading9,
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
    /// The caption of a table.
    TableCaption,
    /// The caption of a figure.
    FigureCaption,
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
    /// The text of a `meta` tag that describes the page.
    Description,
    /// The content of a `data` element: a name or an identifier that is in
    /// the document without being prose.
    Meta,
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
    /// A string value of a Markdown file's front matter, such as the `title`.
    FrontMatter,
}

impl Scope {
    /// Every scope.
    pub const ALL: [Self; 31] = [
        Self::Paragraph,
        Self::Heading1,
        Self::Heading2,
        Self::Heading3,
        Self::Heading4,
        Self::Heading5,
        Self::Heading6,
        Self::Heading0,
        Self::Heading7,
        Self::Heading8,
        Self::Heading9,
        Self::Blockquote,
        Self::List,
        Self::TableRow,
        Self::TableHeader,
        Self::TableCell,
        Self::TableCaption,
        Self::FigureCaption,
        Self::Link,
        Self::Code,
        Self::Strong,
        Self::Emphasis,
        Self::Alt,
        Self::Description,
        Self::Meta,
        Self::Raw,
        Self::CommentLine,
        Self::CommentBlock,
        Self::CommentDocLine,
        Self::CommentDocBlock,
        Self::FrontMatter,
    ];

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
            Self::Heading0 => "heading.h0",
            Self::Heading7 => "heading.h7",
            Self::Heading8 => "heading.h8",
            Self::Heading9 => "heading.h9",
            Self::Blockquote => "blockquote",
            Self::List => "list",
            Self::TableRow => "table.row",
            Self::TableHeader => "table.header",
            Self::TableCell => "table.cell",
            Self::TableCaption => "table.caption",
            Self::FigureCaption => "figure.caption",
            Self::Link => "link",
            Self::Code => "code",
            Self::Strong => "strong",
            Self::Emphasis => "emphasis",
            Self::Alt => "alt",
            Self::Description => "description",
            Self::Meta => "meta",
            Self::Raw => "raw",
            Self::CommentLine => "text.comment.line",
            Self::CommentBlock => "text.comment.block",
            Self::CommentDocLine => "text.comment.doc.line",
            Self::CommentDocBlock => "text.comment.doc.block",
            Self::FrontMatter => "frontmatter",
        }
    }

    /// The scope of a heading of `depth`, or `None` above 9.
    pub(crate) const fn heading(depth: u8) -> Option<Self> {
        match depth {
            1 => Some(Self::Heading1),
            2 => Some(Self::Heading2),
            3 => Some(Self::Heading3),
            4 => Some(Self::Heading4),
            5 => Some(Self::Heading5),
            6 => Some(Self::Heading6),
            0 => Some(Self::Heading0),
            7 => Some(Self::Heading7),
            8 => Some(Self::Heading8),
            9 => Some(Self::Heading9),
            10..=u8::MAX => None,
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// What reads between a segment and the one before it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Join {
    /// What the gap in the file holds: a line feed for a line break, a space
    /// for a table pipe, and nothing for any other gap.
    #[default]
    Gap,
    /// Nothing, whatever the gap holds.
    Nothing,
    /// One space.
    Space,
    /// One line feed.
    Newline,
}

/// How a segment of a node reads, where the file does not say.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Reading {
    /// What reads before the segment.
    pub before: Join,
    /// The text of the segment reads as one `*` for each character, because
    /// it is in an element Vale masks, such as `kbd`.
    pub masked: bool,
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
    /// How each segment reads, in the order of `segments`.
    /// It is empty when every segment reads as written.
    pub reads: Vec<Reading>,
    /// The nodes inside this one, in file order.
    pub children: Vec<Node>,
    /// Whether the node is the first thing inside the list item, quote or
    /// table row around it, ahead of any fenced block, thematic break or other
    /// element that holds no prose.
    pub first: bool,
    /// What follows the scope name in the node's scope: the key of a front
    /// matter value. Every other node has none.
    pub label: Option<String>,
    /// The byte where the element that holds the node's text ends.
    ///
    /// A comment inside the element, after the text, is before this byte.
    /// It is the end of the node's span unless the element goes on, as a
    /// paragraph in a tight list item does with the item.
    pub closes: usize,
}

impl Node {
    /// A node whose text is the one range `span`.
    pub(crate) fn new(scope: Scope, span: Range<usize>, children: Vec<Self>) -> Self {
        Self {
            scope,
            segments: vec![span.clone()],
            reads: Vec::new(),
            closes: span.end,
            span,
            children,
            first: false,
            label: None,
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
            closes: span.end,
            span,
            segments,
            reads: Vec::new(),
            children,
            first: false,
            label: None,
        })
    }

    /// This node with `reads`, which hold one reading for each segment.
    /// Readings that are all the default give no field.
    pub(crate) fn with_reads(mut self, reads: Vec<Reading>) -> Self {
        let plain = reads.iter().all(|reading| *reading == Reading::default());
        self.reads = if plain { Vec::new() } else { reads };
        self
    }

    /// How segment `index` reads.
    pub(crate) fn reading(&self, index: usize) -> Reading {
        self.reads.get(index).copied().unwrap_or_default()
    }

    /// What reads between two segments: the byte `end` of `source`, where one
    /// segment ends, and `next`, where the next one starts.
    ///
    /// A line feed reads where the gap holds a line break, and one space where
    /// the gap lies inside one of the table `rows` and holds a pipe that no
    /// backslash escapes.
    /// Any other gap, such as a link's destination, and a gap that is not a
    /// slice of `source`, reads as nothing.
    pub(crate) fn separator(
        source: &str,
        rows: &[Range<usize>],
        end: usize,
        next: usize,
    ) -> Option<char> {
        let gap = source.get(end..next)?;
        if gap.contains('\n') {
            return Some('\n');
        }
        if !Self::in_row(rows, end, next) {
            return None;
        }
        let mut previous = source
            .get(..end)
            .and_then(|before| before.chars().next_back());
        for c in gap.chars() {
            if c == '|' && previous != Some('\\') {
                return Some(' ');
            }
            previous = Some(c);
        }
        None
    }

    /// The spans of the table rows in this node and below it, in document
    /// order. A row's cells are not read: a row holds no other row.
    pub(crate) fn rows(&self) -> Vec<Range<usize>> {
        let mut rows = Vec::new();
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            if node.scope == Scope::TableRow {
                rows.push(node.span.clone());
            } else {
                pending.extend(node.children.iter().rev());
            }
        }
        rows
    }

    /// Whether the gap from the byte `end` to the byte `next` lies inside one
    /// of the `rows`.
    fn in_row(rows: &[Range<usize>], end: usize, next: usize) -> bool {
        let row = rows.partition_point(|row| row.end < next);
        rows.get(row).is_some_and(|row| row.start <= end)
    }

    /// The text of this node in `source`: its segments in order.
    ///
    /// A line feed separates two segments with a line break between them,
    /// and one space separates two cells of a table row, at a pipe in a gap
    /// inside the row. Any other gap, such as a link's destination, reads as
    /// nothing.
    /// A segment's [`Reading`] gives the separator and the masking instead of
    /// the gap.
    /// A character reference is not read.
    /// Returns `None` when a segment is not a slice of `source`.
    #[must_use]
    pub fn text(&self, source: &str) -> Option<String> {
        let rows = self.rows();
        let mut text = String::new();
        let mut previous_end = None;
        for (index, segment) in self.segments.iter().enumerate() {
            let reading = self.reading(index);
            if let Some(end) = previous_end {
                source.get(end..segment.start)?;
                text.extend(match reading.before {
                    Join::Gap => Self::separator(source, &rows, end, segment.start),
                    Join::Nothing => None,
                    Join::Space => Some(' '),
                    Join::Newline => Some('\n'),
                });
            }
            let written = source.get(segment.clone())?;
            if reading.masked {
                text.extend(written.chars().map(|_| '*'));
            } else {
                text.push_str(written);
            }
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
            node.remap(map);
            pending.extend(node.children.iter_mut());
        }
    }

    /// Replaces the segments and the span of this node alone. `map` gives the
    /// segments of a range from its current place.
    ///
    /// A node whose segments all map to nothing is left with none, and keeps
    /// its old span.
    pub(crate) fn remap(&mut self, map: &impl Fn(Range<usize>) -> Vec<Range<usize>>) {
        let mut segments = Vec::new();
        let mut reads = Vec::new();
        for (index, segment) in self.segments.iter().enumerate() {
            let reading = self.reading(index);
            for (part, range) in map(segment.clone()).into_iter().enumerate() {
                segments.push(range);
                // The text that follows a part of a segment was joined to
                // it by the segment's own text.
                reads.push(Reading {
                    before: if part == 0 { reading.before } else { Join::Gap },
                    ..reading
                });
            }
        }
        self.reads = if self.reads.is_empty() {
            Vec::new()
        } else {
            reads
        };
        self.segments = segments;
        if let (Some(first), Some(last)) = (self.segments.first(), self.segments.last()) {
            self.span = first.start..last.end;
        }
    }
}

/// Replaces the segments and the span of each of `nodes` and of the nodes below
/// them, and the byte each node closes at.
///
/// `map` gives the segments of a range from its current place, and `place`
/// the new byte for a byte, or `None` when it has none.
/// A node whose segments all map to nothing gives way to its children.
///
/// The walk keeps its own stack, so the depth of the tree costs no call stack.
pub(crate) fn remap_nodes(
    nodes: &mut Vec<Node>,
    map: &impl Fn(Range<usize>) -> Vec<Range<usize>>,
    place: &impl Fn(usize) -> Option<usize>,
) {
    let mut pending: Vec<&mut Vec<Node>> = vec![nodes];
    while let Some(level) = pending.pop() {
        let mut at = 0;
        while let Some(node) = level.get_mut(at) {
            node.remap(map);
            node.closes = place(node.closes).unwrap_or(node.span.end);
            if node.segments.is_empty() {
                let children = std::mem::take(&mut node.children);
                level.splice(at..=at, children);
            } else {
                at += 1;
            }
        }
        pending.extend(level.iter_mut().map(|node| &mut node.children));
    }
}

/// The tree of one file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tree {
    /// The `raw` roots: one for a Markdown or MDX file, one per comment group
    /// for a source file.
    pub roots: Vec<Node>,
    /// The number of fenced and indented code blocks in a Markdown or MDX
    /// file, and 0 for a source file.
    pub code_blocks: usize,
    /// The HTML comments of a Markdown file and the `{/* … */}` comments of an
    /// MDX file, in file order, which may turn rules off and on. A source file
    /// gives none.
    pub comments: Vec<Comment>,
}

/// One HTML comment of a Markdown file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comment {
    /// The byte of the file where the comment starts.
    pub at: usize,
    /// What is between `<!--` and the end of the comment, without the blanks
    /// at its ends.
    pub text: String,
}
