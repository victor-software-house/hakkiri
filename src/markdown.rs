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
    fn options(self) -> ParseOptions {
        match self {
            Self::Markdown => ParseOptions {
                constructs: Constructs {
                    frontmatter: true,
                    ..Constructs::gfm()
                },
                ..ParseOptions::default()
            },
            Self::Comment => ParseOptions::gfm(),
            Self::Mdx => ParseOptions {
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
                mdx_esm_parse: Some(Box::new(esm)),
                ..ParseOptions::default()
            },
        }
    }
}

/// Where the lexer of an import or export block stands.
#[derive(Clone, Copy, PartialEq)]
enum Lex {
    Code,
    Quote(char),
    LineComment,
    BlockComment,
}

/// Tells markdown-rs whether an import or export block is complete.
///
/// markdown-rs reads such a block only when it has a parser for it.
/// It ends the block at the first blank line the parser accepts.
/// hakkiri needs the extent of the block, not its syntax, so the block is
/// complete when no bracket, string, template literal or comment is open.
fn esm(block: &str) -> MdxSignal {
    let mut state = Lex::Code;
    let mut depth = 0_usize;
    let mut chars = block.chars().peekable();
    while let Some(c) = chars.next() {
        state = match (state, c) {
            (Lex::Code, '\'' | '"' | '`') => Lex::Quote(c),
            (Lex::Code, '/') if chars.next_if_eq(&'/').is_some() => Lex::LineComment,
            (Lex::Code, '/') if chars.next_if_eq(&'*').is_some() => Lex::BlockComment,
            (Lex::Code, '(' | '[' | '{') => {
                depth += 1;
                Lex::Code
            }
            (Lex::Code, ')' | ']' | '}') => {
                depth = depth.saturating_sub(1);
                Lex::Code
            }
            (Lex::Quote(_), '\\') => {
                chars.next();
                state
            }
            (Lex::Quote(quote), _) if quote == c => Lex::Code,
            (Lex::LineComment, '\n') => Lex::Code,
            (Lex::BlockComment, '*') if chars.next_if_eq(&'/').is_some() => Lex::Code,
            _ => state,
        };
    }
    if depth == 0 && matches!(state, Lex::Code | Lex::LineComment) {
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
    let root = markdown::to_mdast(source, &flavor.options()).map_err(Error::Mdx)?;
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

impl Builder<'_> {
    /// Visits `node`; `cell` is the scope of a table cell found below it.
    fn visit(&self, node: &Mdast, cell: Scope) -> Visited {
        match node {
            Mdast::Paragraph(_) => self.scoped(node, Scope::Paragraph, cell),
            Mdast::Heading(heading) => match Scope::heading(heading.depth) {
                Some(scope) => self.scoped(node, scope, cell),
                None => self.children(node, cell),
            },
            Mdast::Blockquote(_) => self.scoped(node, Scope::Blockquote, cell),
            Mdast::ListItem(_) => self.scoped(node, Scope::List, cell),
            Mdast::TableRow(_) => self.scoped(node, Scope::TableRow, cell),
            Mdast::TableCell(_) => self.scoped(node, cell, cell),
            Mdast::Link(_) | Mdast::LinkReference(_) => self.scoped(node, Scope::Link, cell),
            Mdast::InlineCode(_) => self.scoped(node, Scope::Code, cell),
            Mdast::Strong(_) => self.scoped(node, Scope::Strong, cell),
            Mdast::Emphasis(_) => self.scoped(node, Scope::Emphasis, cell),
            Mdast::Image(_) | Mdast::ImageReference(_) => self.alt(node),
            Mdast::Table(table) => table.children.iter().enumerate().fold(
                Visited::default(),
                |visited, (row, node)| {
                    let cell = if row == 0 {
                        Scope::TableHeader
                    } else {
                        Scope::TableCell
                    };
                    visited.append(self.visit(node, cell))
                },
            ),
            Mdast::MdxJsxFlowElement(_) | Mdast::MdxJsxTextElement(_) => self.jsx(node, cell),
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
            | Mdast::Definition(_) => Visited::cut(span_of(node)),
            Mdast::Root(_) | Mdast::FootnoteDefinition(_) | Mdast::List(_) | Mdast::Delete(_) => {
                self.children(node, cell)
            }
            Mdast::Break(_)
            | Mdast::FootnoteReference(_)
            | Mdast::Text(_)
            | Mdast::ThematicBreak(_) => Visited::default(),
        }
    }

    fn children(&self, node: &Mdast, cell: Scope) -> Visited {
        node.children()
            .into_iter()
            .flatten()
            .fold(Visited::default(), |visited, child| {
                visited.append(self.visit(child, cell))
            })
    }

    /// A node of `scope` over `node`, holding the nodes below it.
    ///
    /// Syntax that is not prose and sits inside the node is a gap between its
    /// segments.
    /// A table row has the segments of its cells.
    fn scoped(&self, node: &Mdast, scope: Scope, cell: Scope) -> Visited {
        let inner = self.children(node, cell);
        let Some(span) = span_of(node).map(|span| self.trimmed(scope, span)) else {
            return inner;
        };
        let Visited {
            nodes: children,
            cuts,
        } = inner;
        let segments = if scope == Scope::TableRow {
            children
                .iter()
                .flat_map(|child| child.segments.clone())
                .collect()
        } else {
            self.segments(&span, &cuts)
        };
        Visited {
            nodes: Node::from_segments(scope, segments, children)
                .into_iter()
                .collect(),
            cuts,
        }
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

    /// A JSX element: its children stay prose and its tags are gaps.
    fn jsx(&self, node: &Mdast, cell: Scope) -> Visited {
        let mut visited = self.children(node, cell);
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
