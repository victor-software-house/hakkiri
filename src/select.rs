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

/// The scope names a node holds.
///
/// `text` marks the prose leaves, which a list item or a block quote is read
/// through.
/// A paragraph inside a list item or a block quote is not a `paragraph`, as in
/// Vale.
fn names(scope: Scope, enclosed: bool) -> Vec<&'static str> {
    match scope {
        Scope::Paragraph if enclosed => vec!["text"],
        Scope::Paragraph => vec!["paragraph", "text"],
        Scope::Heading1
        | Scope::Heading2
        | Scope::Heading3
        | Scope::Heading4
        | Scope::Heading5
        | Scope::Heading6
        | Scope::TableHeader
        | Scope::TableCell => vec![scope.name(), "text"],
        Scope::Blockquote
        | Scope::List
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

/// What the walk does next.
enum Step<'t> {
    Enter(&'t Node),
    Exit,
}

/// The blocks of `tree`, a file of `source` with `extension`, that `selector`
/// reads, in file order: a node before the nodes inside it.
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
    let mut path: Vec<Scope> = Vec::new();
    let mut enclosing = 0_usize;
    let mut steps: Vec<Step<'_>> = tree.roots.iter().rev().map(Step::Enter).collect();
    while let Some(step) = steps.pop() {
        let node = match step {
            Step::Enter(node) => node,
            Step::Exit => {
                if let Some(scope) = path.pop()
                    && matches!(scope, Scope::List | Scope::Blockquote)
                {
                    enclosing -= 1;
                }
                continue;
            }
        };
        let held = names(node.scope, enclosing > 0);
        let own = selector.reads(&held, &path, extension, false);
        let prose = held.contains(&"text");
        let sentences = if prose && model.is_some() {
            let mut with_sentence = vec!["sentence"];
            with_sentence.extend(&held);
            selector.reads(&with_sentence, &path, extension, true)
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
        if matches!(node.scope, Scope::List | Scope::Blockquote) {
            enclosing += 1;
        }
        path.push(node.scope);
        steps.push(Step::Exit);
        steps.extend(node.children.iter().rev().map(Step::Enter));
    }
    Ok(blocks)
}
