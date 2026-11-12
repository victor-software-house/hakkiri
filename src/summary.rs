//! The summary of a file: the prose a whole-file metric measures,
//! and the counts of its elements.

use std::ops::Range;

use crate::select::{SelectError, replacements};
use crate::text::{Decode, Parts, Text};
use crate::tree::{Node, Scope, Tree};

/// The numbers of elements in a file, as a formula reads them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Elements {
    /// The paragraphs outside list items and block quotes.
    pub paragraphs: usize,
    /// The paragraphs inside a list item, not counting those in a block quote
    /// that holds the list.
    pub list: usize,
    /// The paragraphs inside a block quote.
    pub blockquote: usize,
    /// The fenced and indented code blocks.
    pub pre: usize,
    /// The headings of each depth, from 1 to 6.
    pub headings: [usize; 6],
}

impl Elements {
    /// The count the formula variable `name` reads, or `None` for a name that
    /// is no element count.
    #[must_use]
    pub fn count(&self, name: &str) -> Option<usize> {
        let depth = name
            .strip_prefix("heading_h")
            .and_then(|depth| depth.parse::<usize>().ok())
            .and_then(|depth| depth.checked_sub(1));
        match name {
            "paragraphs" => Some(self.paragraphs),
            "list" => Some(self.list),
            "blockquote" => Some(self.blockquote),
            "pre" => Some(self.pre),
            _ => depth.and_then(|depth| self.headings.get(depth).copied()),
        }
    }
}

/// The prose of a file, and its element counts.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Summary {
    /// The paragraphs, list items and block quotes of the file, each followed
    /// by a blank line. Headings and table cells are not in it.
    pub content: String,
    /// The bytes of `content` that read a code span, in order.
    pub code: Vec<Range<usize>>,
    /// The element counts.
    pub elements: Elements,
}

/// What a walk of the tree does next.
enum Step<'t> {
    Enter(&'t Node),
    Exit,
}

/// The summary of `tree`, the tree of `source`: its paragraphs, which in a code
/// file are the paragraphs of its comments.
///
/// Each code span counts as `code_span_size` words.
///
/// # Errors
///
/// Returns a [`SelectError`] when a segment of `tree` is not a slice of
/// `source`.
pub fn summarize(tree: &Tree, source: &str, code_span_size: u8) -> Result<Summary, SelectError> {
    let mut summary = Summary {
        elements: Elements {
            pre: tree.code_blocks,
            ..Elements::default()
        },
        ..Summary::default()
    };
    let mut path: Vec<Scope> = Vec::new();
    let mut steps: Vec<Step<'_>> = tree.roots.iter().rev().map(Step::Enter).collect();
    while let Some(step) = steps.pop() {
        let Step::Enter(node) = step else {
            path.pop();
            continue;
        };
        match node.scope {
            Scope::Paragraph => {
                let text = Text::build(
                    source,
                    &Parts {
                        segments: &node.segments,
                        reads: &node.reads,
                        rows: &node.rows(),
                        replacements: &replacements(node, &[]),
                        decode: Decode::Text,
                    },
                )
                .ok_or(SelectError::Segment(node.scope))?;
                let start = summary.content.len();
                summary.code.extend(
                    text.code_sized_ranges(code_span_size)
                        .iter()
                        .map(|range| start + range.start..start + range.end),
                );
                summary.content.push_str(&text.code_sized(code_span_size));
                summary.content.push_str("\n\n");
                let enclosing = path
                    .iter()
                    .find(|scope| matches!(scope, Scope::List | Scope::Blockquote));
                match enclosing {
                    Some(Scope::List) => summary.elements.list += 1,
                    Some(_) => summary.elements.blockquote += 1,
                    None => summary.elements.paragraphs += 1,
                }
            }
            Scope::Heading1 => summary.elements.headings[0] += 1,
            Scope::Heading2 => summary.elements.headings[1] += 1,
            Scope::Heading3 => summary.elements.headings[2] += 1,
            Scope::Heading4 => summary.elements.headings[3] += 1,
            Scope::Heading5 => summary.elements.headings[4] += 1,
            Scope::Heading6 => summary.elements.headings[5] += 1,
            Scope::Heading0
            | Scope::Heading7
            | Scope::Heading8
            | Scope::Heading9
            | Scope::Blockquote
            | Scope::List
            | Scope::TableRow
            | Scope::TableHeader
            | Scope::TableCell
            | Scope::TableCaption
            | Scope::FigureCaption
            | Scope::Link
            | Scope::Code
            | Scope::Strong
            | Scope::Emphasis
            | Scope::Alt
            | Scope::Title
            | Scope::Image
            | Scope::Description
            | Scope::Meta
            | Scope::FrontMatter
            | Scope::Raw
            | Scope::CommentLine
            | Scope::CommentBlock
            | Scope::CommentDocLine
            | Scope::CommentDocBlock => {}
        }
        path.push(node.scope);
        steps.push(Step::Exit);
        steps.extend(node.children.iter().rev().map(Step::Enter));
    }
    Ok(summary)
}
