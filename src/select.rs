//! Selection: the blocks of text a rule's scope reads in one file.

use std::ops::Range;
use std::{error, fmt};

use crate::scope::{Inline, Selector};
use crate::splitter::{FormatError, Model};
use crate::text::{Replace, Text};
use crate::tree::{Node, Scope, Tree};

/// One piece of text a rule reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    /// The scope of the node the text comes from.
    pub scope: Scope,
    /// Whether the text is one sentence of that node.
    pub sentence: bool,
    /// The text, and where each byte of it lies in the file.
    pub text: Text,
}

/// Why a selection failed.
#[derive(Clone, Copy, Debug)]
pub enum SelectError {
    /// The embedded sentence model did not load.
    Model(&'static FormatError),
    /// A segment of the tree is not a slice of the source: the tree belongs
    /// to another text.
    Segment(Scope),
}

impl fmt::Display for SelectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(error) => write!(f, "cannot load the sentence model: {error}"),
            Self::Segment(scope) => {
                write!(f, "a `{scope}` node is not a slice of the source text")
            }
        }
    }
}

impl error::Error for SelectError {}

/// The scope a node has when it is the one element a block of text is read
/// under: a list item, a quote, a heading or a table cell.
const fn named_scope(scope: Scope) -> Option<&'static str> {
    match scope {
        Scope::List
        | Scope::Blockquote
        | Scope::Heading1
        | Scope::Heading2
        | Scope::Heading3
        | Scope::Heading4
        | Scope::Heading5
        | Scope::Heading6
        | Scope::TableHeader
        | Scope::TableCell => Some(scope.name()),
        Scope::Paragraph
        | Scope::TableRow
        | Scope::Link
        | Scope::Code
        | Scope::Strong
        | Scope::Emphasis
        | Scope::Alt
        | Scope::Raw
        | Scope::CommentLine
        | Scope::CommentBlock
        | Scope::CommentDocLine
        | Scope::CommentDocBlock => None,
    }
}

/// The text of `line` after `depth` quote markers, or `None` when it holds
/// fewer than that.
fn after_markers(line: &str, depth: usize) -> Option<&str> {
    let mut rest = line;
    for _ in 0..depth {
        rest = rest.trim_start().strip_prefix('>')?;
    }
    Some(rest)
}

/// Whether nothing but blank quote lines lies between the start of the quote
/// that holds the node at byte `start`, and the line of that node.
///
/// A quote repeats its marker on every line, so the line of the node holds a
/// marker whether or not something came before it.
/// What came before is on the lines above that carry as many markers as the
/// node's line does, and a line with fewer is outside the quote.
fn first_in_quote(source: &str, line_start: usize, depth: usize) -> bool {
    let mut end = line_start;
    while end > 0 {
        let above = source.get(..end - 1).unwrap_or_default();
        let start = above.rfind('\n').map_or(0, |at| at + 1);
        match source
            .get(start..end - 1)
            .and_then(|line| after_markers(line, depth))
        {
            None => return true,
            Some(rest) if !rest.trim().is_empty() => return false,
            Some(_) => end = start,
        }
    }
    true
}

/// Whether `child` is the first thing inside `parent`.
///
/// The tree keeps only the prose of a document, so a fenced block or another
/// element with none can sit before the first child.
/// The line a first child starts on then holds no marker of its parent: the
/// bullet of a list item, or the pipe of a table row.
/// A quote repeats its marker on every line, so its first child is the one
/// with nothing but blank quote lines above it.
/// A table row holds no block, so a fence cannot come before its first cell.
fn opens(source: &str, parent: &Node, child: &Node) -> bool {
    let markers = match parent.scope {
        Scope::List => "-*+.)",
        Scope::Blockquote => ">",
        Scope::TableRow => "|",
        Scope::Paragraph
        | Scope::Heading1
        | Scope::Heading2
        | Scope::Heading3
        | Scope::Heading4
        | Scope::Heading5
        | Scope::Heading6
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
        | Scope::CommentDocBlock => return true,
    };
    let start = child.span.start;
    let line = source
        .get(..start)
        .map_or(0, |before| before.rfind('\n').map_or(0, |at| at + 1));
    let marked = source.get(line..start).is_some_and(|prefix| {
        prefix.contains(|c| markers.contains(c))
            && prefix
                .chars()
                .all(|c| c.is_whitespace() || c.is_ascii_digit() || "-*+>|.)#[]xX".contains(c))
    });
    marked
        && (parent.scope != Scope::Blockquote
            || source
                .get(line..start)
                .is_some_and(|prefix| first_in_quote(source, line, prefix.matches('>').count())))
}

/// The scope names a node holds.
///
/// `text` marks the prose leaves, which a list item or a block quote is read
/// through: the two hold no scope of their own.
/// A block of text has one scope, as in Vale ([`select`] explains which).
/// `chain` is that scope when an element around the node gives it, and a
/// paragraph with none is a `paragraph`.
fn names(scope: Scope, chain: Option<&'static str>) -> Vec<&'static str> {
    match scope {
        Scope::Paragraph => vec![chain.unwrap_or("paragraph"), "text"],
        Scope::Blockquote | Scope::List => Vec::new(),
        Scope::Heading1
        | Scope::Heading2
        | Scope::Heading3
        | Scope::Heading4
        | Scope::Heading5
        | Scope::Heading6
        | Scope::TableHeader
        | Scope::TableCell => vec![chain.unwrap_or_else(|| scope.name()), "text"],
        Scope::TableRow
        | Scope::Link
        | Scope::Code
        | Scope::Strong
        | Scope::Emphasis
        | Scope::Alt
        | Scope::Raw
        | Scope::CommentLine
        | Scope::CommentBlock
        | Scope::CommentDocLine
        | Scope::CommentDocBlock => vec![scope.name()],
    }
}

/// The segments of the text a node is read as.
///
/// A comment reads as the Markdown inside it, without its markers.
fn segments(node: &Node) -> Vec<Range<usize>> {
    match node.scope {
        Scope::CommentLine
        | Scope::CommentBlock
        | Scope::CommentDocLine
        | Scope::CommentDocBlock => node
            .children
            .iter()
            .flat_map(|child| child.segments.clone())
            .collect(),
        Scope::Paragraph
        | Scope::Heading1
        | Scope::Heading2
        | Scope::Heading3
        | Scope::Heading4
        | Scope::Heading5
        | Scope::Heading6
        | Scope::Blockquote
        | Scope::List
        | Scope::TableRow
        | Scope::TableHeader
        | Scope::TableCell
        | Scope::Link
        | Scope::Code
        | Scope::Strong
        | Scope::Emphasis
        | Scope::Alt
        | Scope::Raw => node.segments.clone(),
    }
}

/// The spans below `node` that read otherwise: a code span reads as `x`, and
/// an inline element the scope excludes reads as spaces.
pub(crate) fn replacements(node: &Node, blank: &[Inline]) -> Vec<(Range<usize>, Replace)> {
    let mut found: Vec<(Range<usize>, Replace)> = Vec::new();
    let mut pending: Vec<&Node> = node.children.iter().collect();
    while let Some(child) = pending.pop() {
        let excluded = blank.iter().any(|inline| inline.scope() == child.scope);
        if excluded {
            found.push((child.span.clone(), Replace::Blank));
        } else if child.scope == Scope::Code {
            found.push((child.span.clone(), Replace::Code));
        }
        pending.extend(child.children.iter());
    }
    found.sort_by_key(|(range, _)| (range.start, std::cmp::Reverse(range.end)));
    let mut kept: Vec<(Range<usize>, Replace)> = Vec::new();
    for (range, kind) in found {
        if kept.last().is_none_or(|(last, _)| last.end <= range.start) {
            kept.push((range, kind));
        }
    }
    kept
}

/// A node the walk still has to enter, with the scope its parent gives it.
struct Step<'t> {
    node: &'t Node,
    chain: Option<&'static str>,
}

/// The blocks of `tree`, a file of `source` with `extension`, that `selector`
/// reads, in file order: a node before the nodes inside it.
///
/// A block of text has the scope of one element, as in Vale, and not of every
/// element around it.
/// Vale reads that element off the tags opened since the last element closed,
/// and takes the outermost of a list item, a quote, a heading and a table
/// cell.
/// So a node that is the first child of an element gets that element's scope
/// when the element had one, and a later child gets its own.
/// The first paragraph of an item is `list` and the second is a `paragraph`.
/// A heading in the middle of a quote is a `heading`.
/// A quote that opens an item is `list`.
///
/// # Errors
///
/// Returns a [`SelectError`] when the sentence model does not load, or when a
/// segment of `tree` is not a slice of `source`.
pub fn select(
    selector: &Selector,
    tree: &Tree,
    source: &str,
    extension: &str,
) -> Result<Vec<Block>, SelectError> {
    let model = if selector.wants_sentences() {
        Some(Model::english().map_err(SelectError::Model)?)
    } else {
        None
    };
    let mut blocks = Vec::new();
    let mut steps: Vec<Step<'_>> = tree
        .roots
        .iter()
        .rev()
        .map(|node| Step { node, chain: None })
        .collect();
    while let Some(Step { node, chain }) = steps.pop() {
        let chain = chain.or_else(|| named_scope(node.scope));
        let held = names(node.scope, chain);
        let own = selector.reads(&held, extension, false);
        let prose = held.contains(&"text");
        let sentences = if prose && model.is_some() {
            let mut with_sentence = vec!["sentence"];
            with_sentence.extend(&held);
            selector.reads(&with_sentence, extension, true)
        } else {
            None
        };
        if own.is_some() || sentences.is_some() {
            let rows = node.rows();
            let text_for = |blank: &[Inline]| {
                if node.scope == Scope::Raw {
                    Text::build(source, &segments(node), &rows, &[])
                } else {
                    Text::build(source, &segments(node), &rows, &replacements(node, blank))
                }
                .ok_or(SelectError::Segment(node.scope))
            };
            let own_text = match &own {
                Some(blank) => Some(text_for(blank)?),
                None => None,
            };
            if let Some(text) = &own_text
                && !text.as_str().is_empty()
            {
                blocks.push(Block {
                    scope: node.scope,
                    sentence: false,
                    text: text.clone(),
                });
            }
            if let (Some(blank), Some(model)) = (&sentences, model) {
                let built = match (&own, &own_text) {
                    (Some(own), Some(text)) if own == blank => text.clone(),
                    _ => text_for(blank)?,
                };
                for range in model.split(built.as_str()) {
                    if let Some(text) = built.slice(range) {
                        blocks.push(Block {
                            scope: node.scope,
                            sentence: true,
                            text,
                        });
                    }
                }
            }
        }
        steps.extend(
            node.children
                .iter()
                .enumerate()
                .rev()
                .map(|(index, child)| Step {
                    node: child,
                    chain: if index == 0 && opens(source, node, child) {
                        chain
                    } else {
                        None
                    },
                }),
        );
    }
    Ok(blocks)
}
