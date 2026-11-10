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
                // markdown-rs reads an import or export block only when it has
                // a parser for it. hakkiri needs the block's extent, not its
                // syntax, so every block passes.
                mdx_esm_parse: Some(Box::new(|_| MdxSignal::Ok)),
                ..ParseOptions::default()
            },
        }
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
            Mdast::Image(_) => self.alt(node),
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
            Mdast::MdxjsEsm(_) | Mdast::MdxFlowExpression(_) | Mdast::MdxTextExpression(_) => {
                Visited::cut(span_of(node))
            }
            Mdast::Root(_) | Mdast::FootnoteDefinition(_) | Mdast::List(_) | Mdast::Delete(_) => {
                self.children(node, cell)
            }
            Mdast::Toml(_)
            | Mdast::Yaml(_)
            | Mdast::Break(_)
            | Mdast::InlineMath(_)
            | Mdast::FootnoteReference(_)
            | Mdast::Html(_)
            | Mdast::ImageReference(_)
            | Mdast::Text(_)
            | Mdast::Code(_)
            | Mdast::Math(_)
            | Mdast::ThematicBreak(_)
            | Mdast::Definition(_) => Visited::default(),
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
    /// Where MDX syntax cuts into the node's span, the span splits around it.
    /// One node of the scope stands for each piece that holds text.
    fn scoped(&self, node: &Mdast, scope: Scope, cell: Scope) -> Visited {
        let inner = self.children(node, cell);
        let Some(span) = span_of(node).map(|span| self.trimmed(scope, span)) else {
            return inner;
        };
        let Visited {
            nodes: children,
            cuts,
        } = inner;
        let mut pieces: Vec<Node> = self
            .pieces(&span, &cuts)
            .into_iter()
            .map(|span| Node::new(scope, span, Vec::new()))
            .collect();
        for child in children {
            if let Some(piece) = pieces
                .iter_mut()
                .find(|piece| piece.span.contains(&child.span.start))
            {
                piece.children.push(child);
            }
        }
        Visited {
            nodes: pieces,
            cuts,
        }
    }

    /// `span` without the line ending and padding that markdown-rs leaves at
    /// its end.
    ///
    /// A table row or cell also loses its pipes and padding.
    /// markdown-rs starts a cell at its leading pipe and ends it after the
    /// padding or the closing pipe.
    fn trimmed(&self, scope: Scope, span: Range<usize>) -> Range<usize> {
        let Some(text) = self.source.get(span.clone()) else {
            return span;
        };
        let text = text.trim_end();
        if !matches!(
            scope,
            Scope::TableRow | Scope::TableHeader | Scope::TableCell
        ) {
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

    /// The parts of `span` that no cut covers and that hold more than
    /// whitespace, without the whitespace around them.
    fn pieces(&self, span: &Range<usize>, cuts: &[Range<usize>]) -> Vec<Range<usize>> {
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
        parts
            .into_iter()
            .filter_map(|part| {
                let text = self.source.get(part.clone())?;
                let start = part.start + (text.len() - text.trim_start().len());
                let trimmed = text.trim();
                (!trimmed.is_empty()).then(|| start..start + trimmed.len())
            })
            .collect()
    }

    /// A JSX element: its children stay prose and its tags are cut out.
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

    /// The `alt` node of an image: the label between `![` and its closing
    /// bracket.
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
