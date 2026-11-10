//! Markdown and MDX text into nodes, through markdown-rs.

use std::ops::Range;

use markdown::mdast::Node as Mdast;
use markdown::{Constructs, MdxSignal, ParseOptions};

use crate::error::Error;
use crate::tree::{Node, Scope};

/// Which constructs the parser reads.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Flavor {
    /// A `.md` file: GFM and front matter.
    Markdown,
    /// An `.mdx` file: MDX, GFM and front matter.
    Mdx,
    /// The text of a comment: GFM, with no front matter.
    Comment,
}

impl Flavor {
    /// `source` is the file the options parse.
    fn options(self, source: &str) -> ParseOptions {
        match self {
            Self::Markdown => ParseOptions {
                constructs: Constructs {
                    frontmatter: true,
                    ..Constructs::gfm()
                },
                ..ParseOptions::default()
            },
            Self::Comment => ParseOptions::gfm(),
            Self::Mdx => {
                let tail = source.trim_end().to_owned();
                ParseOptions {
                    constructs: Constructs {
                        frontmatter: true,
                        gfm_autolink_literal: true,
                        gfm_footnote_definition: true,
                        gfm_label_start_footnote: true,
                        gfm_strikethrough: true,
                        gfm_table: true,
                        gfm_task_list_item: true,
                        ..Constructs::mdx()
                    },
                    mdx_esm_parse: Some(Box::new(move |block: &str| esm(block, &tail))),
                    ..ParseOptions::default()
                }
            }
        }
    }
}

/// Where the lexer of an import or export block stands.
#[derive(Clone, Copy, PartialEq)]
enum Lex {
    Code,
    /// A string in single or double quotes. It ends at a line feed.
    Quote(char),
    /// A template literal. Its `${` opens code again.
    Template,
    /// A regular expression literal. It ends at a line feed.
    Regex {
        class: bool,
    },
    LineComment,
    BlockComment,
}

/// The characters after which a `/` starts a regular expression and not a
/// division.
const BEFORE_REGEX: &str = "(,=:[!&|?{};+-*%<>~^";

/// Tells markdown-rs whether an import or export block is complete.
///
/// markdown-rs reads such a block only when it has a parser for it.
/// It ends the block at the first blank line the parser accepts.
/// hakkiri needs the extent of the block, not its syntax.
/// The block is complete when no bracket, string, template literal, regular
/// expression or comment is open.
///
/// `source` is the whole file without its trailing whitespace.
/// A block that ends the file is complete whatever the lexer saw.
/// It cannot grow, and an error would fail the whole file over a misread
/// literal.
fn esm(block: &str, source: &str) -> MdxSignal {
    let mut state = Lex::Code;
    let mut depth = 0_usize;
    // The bracket depth of each `${` that is still open.
    let mut templates: Vec<usize> = Vec::new();
    let mut previous = None;
    let mut chars = block.chars().peekable();
    while let Some(c) = chars.next() {
        state = match (state, c) {
            (Lex::Code, '\'' | '"') => Lex::Quote(c),
            (Lex::Code, '`') => Lex::Template,
            (Lex::Template, '`')
            | (Lex::Quote(_) | Lex::Regex { .. } | Lex::LineComment, '\n')
            | (Lex::Regex { class: false }, '/') => Lex::Code,
            (Lex::Template, '$') if chars.next_if_eq(&'{').is_some() => {
                depth += 1;
                templates.push(depth);
                Lex::Code
            }
            (Lex::Code, '/') if chars.next_if_eq(&'/').is_some() => Lex::LineComment,
            (Lex::Code, '/') if chars.next_if_eq(&'*').is_some() => Lex::BlockComment,
            (Lex::Code, '/') if previous.is_none_or(|p| BEFORE_REGEX.contains(p)) => {
                Lex::Regex { class: false }
            }
            (Lex::Code, '(' | '[' | '{') => {
                depth += 1;
                Lex::Code
            }
            (Lex::Code, '}') if templates.last() == Some(&depth) => {
                templates.pop();
                depth = depth.saturating_sub(1);
                Lex::Template
            }
            (Lex::Code, ')' | ']' | '}') => {
                depth = depth.saturating_sub(1);
                Lex::Code
            }
            (Lex::Quote(_) | Lex::Template | Lex::Regex { .. }, '\\') => {
                chars.next();
                state
            }
            (Lex::Quote(quote), _) if quote == c => Lex::Code,
            (Lex::Regex { .. }, '[') => Lex::Regex { class: true },
            (Lex::Regex { .. }, ']') => Lex::Regex { class: false },
            (Lex::BlockComment, '*') if chars.next_if_eq(&'/').is_some() => Lex::Code,
            _ => state,
        };
        if state == Lex::Code && !c.is_whitespace() {
            previous = Some(c);
        }
    }
    // A block ends at a line end, which closes a string or a regular
    // expression.
    let complete = depth == 0
        && matches!(
            state,
            Lex::Code | Lex::LineComment | Lex::Quote(_) | Lex::Regex { .. }
        );
    if complete || source.ends_with(block.trim_end()) {
        MdxSignal::Ok
    } else {
        MdxSignal::Eof(
            "Unexpected end of file in an import or export".to_owned(),
            Box::new("hakkiri".to_owned()),
            Box::new("esm".to_owned()),
        )
    }
}

/// The tree of a whole Markdown or MDX file: one `raw` root over the prose
/// nodes.
pub(crate) fn document(source: &str, flavor: Flavor) -> Result<Vec<Node>, Error> {
    let children = blocks(source, flavor)?;
    Ok(vec![Node::new(Scope::Raw, 0..source.len(), children)])
}

/// The prose nodes of `source`, with spans in `source`.
pub(crate) fn blocks(source: &str, flavor: Flavor) -> Result<Vec<Node>, Error> {
    let root = markdown::to_mdast(source, &flavor.options(source)).map_err(Error::Mdx)?;
    Ok(Builder { source }.visit(&root, Scope::TableCell).nodes)
}

/// What a visit of one mdast node gives its parent.
#[derive(Debug, Default)]
struct Visited {
    /// The prose nodes, in file order.
    nodes: Vec<Node>,
    /// MDX syntax inside the node that is not prose: JSX tags, expressions and
    /// ESM.
    cuts: Vec<Range<usize>>,
}

impl Visited {
    fn cut(span: Option<Range<usize>>) -> Self {
        Self {
            nodes: Vec::new(),
            cuts: span.into_iter().collect(),
        }
    }

    fn append(mut self, other: Self) -> Self {
        self.nodes.extend(other.nodes);
        self.cuts.extend(other.cuts);
        self
    }
}

struct Builder<'a> {
    source: &'a str,
}

/// How a node of the mdast takes part in the tree.
enum Kind {
    /// A node of this scope over the nodes below it.
    Scoped(Scope),
    /// A table cell, of the scope the table gives its row.
    Cell,
    /// A JSX element: its tags are gaps in the text around it.
    Jsx,
    /// No node of its own: the nodes below it go to its parent.
    Transparent,
}

/// What the builder does with a node before it looks below it.
enum Step {
    /// The node has no prose below it. This is what it gives its parent.
    Done(Visited),
    /// The nodes below the node come first.
    Open(Kind),
}

/// A node whose children the builder is still visiting.
struct Frame<'n> {
    node: &'n Mdast,
    kind: Kind,
    /// The scope of a table cell found below the node.
    cell: Scope,
    children: std::slice::Iter<'n, Mdast>,
    index: usize,
    inner: Visited,
}

impl<'n> Frame<'n> {
    fn new(node: &'n Mdast, kind: Kind, cell: Scope) -> Self {
        Self {
            node,
            kind,
            cell,
            children: node.children().map_or(&[][..], Vec::as_slice).iter(),
            index: 0,
            inner: Visited::default(),
        }
    }

    /// The next child and the scope of a table cell below it.
    fn next_child(&mut self) -> Option<(&'n Mdast, Scope)> {
        let child = self.children.next()?;
        let cell = match (self.node, self.index) {
            (Mdast::Table(_), 0) => Scope::TableHeader,
            (Mdast::Table(_), _) => Scope::TableCell,
            _ => self.cell,
        };
        self.index += 1;
        Some((child, cell))
    }
}

impl Builder<'_> {
    /// Visits `root` and everything below it.
    ///
    /// The visit keeps its own stack of frames, so the depth of the document
    /// costs no call stack.
    fn visit(&self, root: &Mdast, cell: Scope) -> Visited {
        let mut stack = Vec::new();
        match self.step(root) {
            Step::Done(visited) => return visited,
            Step::Open(kind) => stack.push(Frame::new(root, kind, cell)),
        }
        loop {
            let Some(top) = stack.last_mut() else {
                return Visited::default();
            };
            if let Some((child, cell)) = top.next_child() {
                match self.step(child) {
                    Step::Done(visited) => {
                        top.inner = std::mem::take(&mut top.inner).append(visited);
                    }
                    Step::Open(kind) => stack.push(Frame::new(child, kind, cell)),
                }
                continue;
            }
            let Some(frame) = stack.pop() else {
                return Visited::default();
            };
            let visited = self.finish(frame);
            match stack.last_mut() {
                Some(parent) => {
                    parent.inner = std::mem::take(&mut parent.inner).append(visited);
                }
                None => return visited,
            }
        }
    }

    fn step(&self, node: &Mdast) -> Step {
        match node {
            Mdast::Paragraph(_) => Step::Open(Kind::Scoped(Scope::Paragraph)),
            Mdast::Heading(heading) => {
                Step::Open(Scope::heading(heading.depth).map_or(Kind::Transparent, Kind::Scoped))
            }
            Mdast::Blockquote(_) => Step::Open(Kind::Scoped(Scope::Blockquote)),
            Mdast::ListItem(_) => Step::Open(Kind::Scoped(Scope::List)),
            Mdast::TableRow(_) => Step::Open(Kind::Scoped(Scope::TableRow)),
            Mdast::TableCell(_) => Step::Open(Kind::Cell),
            Mdast::Link(_) | Mdast::LinkReference(_) => Step::Open(Kind::Scoped(Scope::Link)),
            Mdast::InlineCode(_) => Step::Open(Kind::Scoped(Scope::Code)),
            Mdast::Strong(_) => Step::Open(Kind::Scoped(Scope::Strong)),
            Mdast::Emphasis(_) => Step::Open(Kind::Scoped(Scope::Emphasis)),
            Mdast::Image(_) | Mdast::ImageReference(_) => Step::Done(self.alt(node)),
            Mdast::MdxJsxFlowElement(_) | Mdast::MdxJsxTextElement(_) => Step::Open(Kind::Jsx),
            // Syntax that is not prose is a gap in the text around it.
            Mdast::MdxjsEsm(_)
            | Mdast::MdxFlowExpression(_)
            | Mdast::MdxTextExpression(_)
            | Mdast::Toml(_)
            | Mdast::Yaml(_)
            | Mdast::InlineMath(_)
            | Mdast::Html(_)
            | Mdast::Code(_)
            | Mdast::Math(_)
            | Mdast::Definition(_) => Step::Done(Visited::cut(span_of(node))),
            Mdast::Root(_)
            | Mdast::FootnoteDefinition(_)
            | Mdast::List(_)
            | Mdast::Delete(_)
            | Mdast::Table(_) => Step::Open(Kind::Transparent),
            Mdast::Break(_)
            | Mdast::FootnoteReference(_)
            | Mdast::Text(_)
            | Mdast::ThematicBreak(_) => Step::Done(Visited::default()),
        }
    }

    /// What a node gives its parent once the nodes below it are visited.
    fn finish(&self, frame: Frame<'_>) -> Visited {
        match frame.kind {
            Kind::Scoped(scope) => self.scoped(frame.node, scope, frame.inner),
            Kind::Cell => self.scoped(frame.node, frame.cell, frame.inner),
            Kind::Jsx => Self::jsx(frame.node, frame.inner),
            Kind::Transparent => frame.inner,
        }
    }

    /// A node of `scope` over `node`, holding the nodes in `inner`.
    ///
    /// Syntax that is not prose and sits inside the node is a gap between its
    /// segments.
    /// A table row, a block quote and a list item have the segments of the
    /// nodes inside them.
    /// No marker, bullet or indentation is in their text.
    /// A container with no prose inside it gives no node.
    fn scoped(&self, node: &Mdast, scope: Scope, inner: Visited) -> Visited {
        let Some(span) = span_of(node).map(|span| self.trimmed(scope, span)) else {
            return inner;
        };
        let Visited {
            nodes: children,
            cuts,
        } = inner;
        let segments = match scope {
            // A container reads as the content inside it.
            Scope::TableRow | Scope::Blockquote | Scope::List => children
                .iter()
                .flat_map(|child| child.segments.clone())
                .collect(),
            Scope::Heading1
            | Scope::Heading2
            | Scope::Heading3
            | Scope::Heading4
            | Scope::Heading5
            | Scope::Heading6 => {
                let atx = self
                    .source
                    .get(span.clone())
                    .is_some_and(|text| !text.contains('\n'));
                self.heading_text(self.lines_of(&span, &cuts), atx)
            }
            Scope::Paragraph
            | Scope::TableHeader
            | Scope::TableCell
            | Scope::Link
            | Scope::Code
            | Scope::Strong
            | Scope::Emphasis
            | Scope::Alt
            | Scope::Raw
            | Scope::CommentLine
            | Scope::CommentBlock
            | Scope::CommentDocLine
            | Scope::CommentDocBlock => self.lines_of(&span, &cuts),
        };
        // A node with no text gives way to the nodes below it.
        let nodes = if segments.is_empty() {
            children
        } else {
            Node::from_segments(scope, segments, children)
                .into_iter()
                .collect()
        };
        Visited { nodes, cuts }
    }

    /// The lines of `span` that no cut covers, one range each.
    fn lines_of(&self, span: &Range<usize>, cuts: &[Range<usize>]) -> Vec<Range<usize>> {
        self.segments(span, cuts)
            .into_iter()
            .flat_map(|part| self.lines(part))
            .collect()
    }

    /// The lines of a heading without its syntax.
    ///
    /// An ATX heading loses its opening `#` run with the space after it, and
    /// its closing `#` run.
    /// A setext heading loses its underline.
    fn heading_text(&self, mut lines: Vec<Range<usize>>, atx: bool) -> Vec<Range<usize>> {
        let text_of = |line: &Range<usize>| self.source.get(line.clone()).unwrap_or_default();
        if atx {
            if let Some(first) = lines.first_mut() {
                let text = text_of(first);
                let rest = text.trim_start_matches('#');
                first.start += text.len() - rest.trim_start().len();
            }
            if let Some(last) = lines.last_mut() {
                let text = text_of(last);
                let body = text.trim_end_matches('#');
                if body.len() < text.len() && (body.is_empty() || body.ends_with([' ', '\t'])) {
                    last.end = last.start + body.trim_end().len();
                }
            }
        } else if lines.len() > 1
            && lines.last().is_some_and(|last| {
                let text = text_of(last).trim();
                text.chars().all(|c| c == '=') || text.chars().all(|c| c == '-')
            })
        {
            lines.pop();
        }
        lines.retain(|line| !line.is_empty());
        lines
    }

    /// `span` without the line ending and padding that markdown-rs leaves at
    /// its end.
    ///
    /// A table cell also loses its pipes and padding.
    /// markdown-rs starts a cell at its leading pipe and ends it after the
    /// padding or the closing pipe.
    fn trimmed(&self, scope: Scope, span: Range<usize>) -> Range<usize> {
        let Some(text) = self.source.get(span.clone()) else {
            return span;
        };
        let text = text.trim_end();
        if !matches!(scope, Scope::TableHeader | Scope::TableCell) {
            return span.start..span.start + text.len();
        }
        let unpadded = text.trim_start();
        let pipe = usize::from(unpadded.starts_with('|'));
        let body = unpadded.get(pipe..).unwrap_or_default();
        let start = span.start
            + (text.len() - unpadded.len())
            + pipe
            + (body.len() - body.trim_start().len());
        let mut body = body.trim();
        if let Some(before) = body
            .strip_suffix('|')
            .filter(|before| !before.ends_with('\\'))
        {
            body = before.trim_end();
        }
        start..start + body.len()
    }

    /// The parts of `span` that no cut covers, without the whitespace at
    /// either end of the text.
    ///
    /// Returns no part when only whitespace is left.
    fn segments(&self, span: &Range<usize>, cuts: &[Range<usize>]) -> Vec<Range<usize>> {
        let mut inside: Vec<&Range<usize>> = cuts
            .iter()
            .filter(|cut| cut.start < span.end && span.start < cut.end)
            .collect();
        inside.sort_by_key(|cut| cut.start);
        let mut parts = Vec::new();
        let mut at = span.start;
        for cut in inside {
            if at < cut.start {
                parts.push(at..cut.start);
            }
            at = at.max(cut.end);
        }
        if at < span.end {
            parts.push(at..span.end);
        }
        let text_of = |part: &Range<usize>| self.source.get(part.clone()).unwrap_or_default();
        if parts.iter().all(|part| text_of(part).trim().is_empty()) {
            return Vec::new();
        }
        if let Some(first) = parts.first_mut() {
            first.start += text_of(first).len() - text_of(first).trim_start().len();
        }
        if let Some(last) = parts.last_mut() {
            last.end = last.start + text_of(last).trim_end().len();
        }
        parts.retain(|part| !part.is_empty());
        parts
    }

    /// One range for each line of `part`, without its line ending.
    ///
    /// A line after the first also loses the indentation and the block quote
    /// markers that open it: they belong to the containers around the node.
    /// A line that is left empty gives no range.
    fn lines(&self, part: Range<usize>) -> Vec<Range<usize>> {
        let Some(text) = self.source.get(part.clone()) else {
            return vec![part];
        };
        let count = text.split('\n').count();
        let mut lines = Vec::new();
        let mut offset = 0;
        for (index, line) in text.split('\n').enumerate() {
            let start = part.start + offset;
            offset += line.len() + 1;
            let line = if index + 1 < count {
                line.strip_suffix('\r').unwrap_or(line)
            } else {
                line
            };
            let opening = if index == 0 { 0 } else { opening_length(line) };
            lines.push(start + opening..start + line.len());
        }
        lines.retain(|line| !line.is_empty());
        lines
    }

    /// A JSX element: its children stay prose and its tags are gaps.
    fn jsx(node: &Mdast, mut visited: Visited) -> Visited {
        if let Some(span) = span_of(node) {
            let children = node.children().map_or(&[][..], Vec::as_slice);
            let first = children.first().and_then(span_of);
            let last = children.last().and_then(span_of);
            match (first, last) {
                (Some(first), Some(last)) => {
                    visited.cuts.push(span.start..first.start);
                    visited.cuts.push(last.end..span.end);
                }
                _ => visited.cuts.push(span),
            }
        }
        visited
    }

    /// The `alt` node of an image or an image reference: the label between
    /// `![` and its closing bracket.
    fn alt(&self, node: &Mdast) -> Visited {
        let span = span_of(node).and_then(|span| {
            let label = self.source.get(span.clone())?.strip_prefix("![")?;
            let length = label_length(label).filter(|length| *length > 0)?;
            let start = span.start + "![".len();
            Some(start..start + length)
        });
        Visited {
            nodes: span
                .map(|span| Node::new(Scope::Alt, span, Vec::new()))
                .into_iter()
                .collect(),
            cuts: Vec::new(),
        }
    }
}

fn span_of(node: &Mdast) -> Option<Range<usize>> {
    node.position()
        .map(|position| position.start.offset..position.end.offset)
}

/// The byte length of the indentation and block quote markers that open
/// `line`, a line of text inside a paragraph.
///
/// A line of a paragraph never starts with `>` unless a block quote around
/// the paragraph put it there, because a `>` would end the paragraph.
fn opening_length(line: &str) -> usize {
    let mut at = 0;
    loop {
        let rest = line.get(at..).unwrap_or_default();
        at += rest.len() - rest.trim_start_matches([' ', '\t']).len();
        if line.get(at..).is_some_and(|rest| rest.starts_with('>')) {
            at += 1;
        } else {
            return at;
        }
    }
}

/// The byte length of a link label that starts `label`, up to its closing
/// bracket.
fn label_length(label: &str) -> Option<usize> {
    let mut depth = 0_usize;
    let mut escaped = false;
    for (index, byte) in label.bytes().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' => escaped = true,
            b'[' => depth += 1,
            b']' if depth == 0 => return Some(index),
            b']' => depth -= 1,
            _ => {}
        }
    }
    None
}
