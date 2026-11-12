//! Comments of Rust, TypeScript and JavaScript files into nodes, through
//! tree-sitter.
//!
//! Only comments enter the tree.
//! The markers of a comment are stripped, and its text is read as Markdown.
//! Every span the Markdown parser gives maps back to the bytes of the file.

use std::ops::Range;

use tree_sitter::{Language as Grammar, Parser, Query, QueryCursor, StreamingIterator};

use crate::error::Error;
use crate::markdown::{self, Flavor};
use crate::pattern::Pattern;
use crate::tree::{Node, Scope, remap_nodes};
use crate::{ignore, jsdoc};

/// The grammar that finds the comments of a file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Dialect {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
}

impl Dialect {
    fn grammar(self) -> Grammar {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
        }
    }

    /// The query that captures each comment node.
    const fn query(self) -> &'static str {
        match self {
            Self::Rust => "[(line_comment) (block_comment)] @comment",
            Self::TypeScript | Self::Tsx | Self::JavaScript => "(comment) @comment",
        }
    }
}

/// How a comment opens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Marker {
    /// `//`
    Line,
    /// Rust `///`
    OuterDocLine,
    /// Rust `//!`
    InnerDocLine,
    /// `/*`
    Block,
    /// `/**`, and Rust `/*!`
    DocBlock,
}

impl Marker {
    fn of(dialect: Dialect, text: &str) -> Option<Self> {
        let is_rust = dialect == Dialect::Rust;
        if let Some(after) = text.strip_prefix("//") {
            Some(if is_rust && after.starts_with('!') {
                Self::InnerDocLine
            } else if is_rust && after.starts_with('/') && !after.starts_with("//") {
                Self::OuterDocLine
            } else {
                Self::Line
            })
        } else {
            let rest = text.strip_prefix("/*")?;
            // `/**/` and `/***` are plain comments.
            let javadoc =
                rest.starts_with('*') && !rest.starts_with("*/") && !rest.starts_with("**");
            Some(if javadoc || (is_rust && rest.starts_with('!')) {
                Self::DocBlock
            } else {
                Self::Block
            })
        }
    }

    const fn scope(self) -> Scope {
        match self {
            Self::Line => Scope::CommentLine,
            Self::OuterDocLine | Self::InnerDocLine => Scope::CommentDocLine,
            Self::Block => Scope::CommentBlock,
            Self::DocBlock => Scope::CommentDocBlock,
        }
    }

    /// The bytes of the opening marker.
    const fn opener(self) -> usize {
        match self {
            Self::Line | Self::Block => 2,
            Self::OuterDocLine | Self::InnerDocLine | Self::DocBlock => 3,
        }
    }

    const fn is_line(self) -> bool {
        matches!(self, Self::Line | Self::OuterDocLine | Self::InnerDocLine)
    }

    /// Whether rustdoc reads a run of these across blank lines as one text.
    const fn is_doc_line(self) -> bool {
        matches!(self, Self::OuterDocLine | Self::InnerDocLine)
    }
}

/// One comment, or several adjacent line comments of the same marker.
#[derive(Debug)]
struct Group {
    marker: Marker,
    /// The file bytes of each comment, markers included.
    comments: Vec<Range<usize>>,
}

/// The `raw` roots of a source file: one per comment group.
///
/// The Markdown of each comment is rewritten under `blocks` and `tokens`, as
/// [`crate::ignore`] explains for a file.
pub(crate) fn roots(
    dialect: Dialect,
    source: &str,
    blocks: &[&Pattern],
    tokens: &[&Pattern],
) -> Result<Vec<Node>, Error> {
    groups(source, &find(dialect, source)?)
        .iter()
        .map(|group| root(dialect, source, group, blocks, tokens))
        .collect()
}

/// The byte range and marker of each comment, in file order.
fn find(dialect: Dialect, source: &str) -> Result<Vec<(Range<usize>, Marker)>, Error> {
    let grammar = dialect.grammar();
    let mut parser = Parser::new();
    parser.set_language(&grammar)?;
    let tree = parser.parse(source, None).ok_or(Error::Syntax)?;
    let query = Query::new(&grammar, dialect.query())?;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let mut found = Vec::new();
    while let Some(found_match) = matches.next() {
        for node in found_match.nodes_for_capture_index(0) {
            let range = node.byte_range();
            let text = source.get(range.clone()).unwrap_or_default();
            // A line comment ends before its line ending.
            let end = range.start + text.trim_end_matches(['\n', '\r']).len();
            if let Some(marker) = Marker::of(dialect, text) {
                found.push((range.start..end, marker));
            }
        }
    }
    found.sort_by_key(|(range, _)| range.start);
    Ok(found)
}

/// Joins each run of adjacent line comments that share a marker, as rustdoc
/// does. Doc line comments join across blank lines too, since rustdoc reads
/// the lines of one item's doc comments as one text, so a fence opened before
/// a blank line stays open after it.
fn groups(source: &str, comments: &[(Range<usize>, Marker)]) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for (range, marker) in comments {
        let joins = groups.last().is_some_and(|group| {
            group.marker == *marker
                && marker.is_line()
                && group.comments.last().is_some_and(|previous| {
                    let gap = source.get(previous.end..range.start).unwrap_or_default();
                    gap.trim().is_empty()
                        && (marker.is_doc_line() || gap.matches('\n').count() == 1)
                        && starts_line(source, previous.start)
                })
        });
        match groups.last_mut() {
            Some(group) if joins => group.comments.push(range.clone()),
            _ => groups.push(Group {
                marker: *marker,
                comments: vec![range.clone()],
            }),
        }
    }
    groups
}

/// Whether only whitespace sits before `offset` on its line.
fn starts_line(source: &str, offset: usize) -> bool {
    source
        .get(..offset)
        .and_then(|before| before.rsplit('\n').next())
        .is_some_and(|line| line.trim().is_empty())
}

/// The `raw` root of one group: the comment node over the Markdown of its text.
fn root(
    dialect: Dialect,
    source: &str,
    group: &Group,
    blocks: &[&Pattern],
    tokens: &[&Pattern],
) -> Result<Node, Error> {
    let stripped = strip(source, &lines(source, group));
    let segments = if dialect != Dialect::Rust && group.marker == Marker::DocBlock {
        jsdoc::prose(&stripped.text)
    } else {
        std::iter::once(0..stripped.text.len()).collect()
    };
    let mut children = Vec::new();
    for segment in segments {
        let text = stripped.text.get(segment.clone()).unwrap_or_default();
        if blocks.is_empty() && tokens.is_empty() {
            let mut nodes = markdown::blocks(text, Flavor::Comment)?;
            for node in &mut nodes {
                node.map_segments(&|span| {
                    stripped.segments(span.start + segment.start..span.end + segment.start)
                });
            }
            children.extend(nodes);
        } else {
            let changed = ignore::rewrite(text, blocks, tokens).map_err(Error::Ignore)?;
            let mut nodes = markdown::blocks(changed.text(), Flavor::Comment)?;
            remap_nodes(
                &mut nodes,
                &|span| {
                    changed
                        .origin(span)
                        .into_iter()
                        .flat_map(|place| {
                            stripped
                                .segments(place.start + segment.start..place.end + segment.start)
                        })
                        .collect()
                },
                &Some,
            );
            children.extend(nodes);
        }
    }
    let start = group.comments.first().map_or(0, |comment| comment.start);
    let end = group.comments.last().map_or(0, |comment| comment.end);
    let comment = Node {
        scope: group.marker.scope(),
        span: start..end,
        segments: group.comments.clone(),
        reads: Vec::new(),
        children,
        first: false,
        closes: end,
        label: None,
    };
    Ok(Node {
        scope: Scope::Raw,
        span: start..end,
        segments: group.comments.clone(),
        reads: Vec::new(),
        children: vec![comment],
        first: false,
        closes: end,
        label: None,
    })
}

/// The file bytes of each line of comment text, with the markers and the `*` of
/// each line removed.
///
/// A plain comment also loses the run of `/` or `*` that follows its opener, so
/// a banner such as `//////` or `/*****` leaves no punctuation in the text.
fn lines(source: &str, group: &Group) -> Vec<Range<usize>> {
    if group.marker.is_line() {
        return group
            .comments
            .iter()
            .map(|comment| {
                let start = comment.start + group.marker.opener();
                let banner = if group.marker == Marker::Line {
                    leading_run(source.get(start..comment.end).unwrap_or_default(), '/')
                } else {
                    0
                };
                start + banner..comment.end
            })
            .map(|line| trim_carriage_return(source, line))
            .collect();
    }
    let Some(comment) = group.comments.first() else {
        return Vec::new();
    };
    let mut start = comment.start + group.marker.opener();
    let text = source.get(comment.clone()).unwrap_or_default();
    let mut end = if text.len() >= group.marker.opener() + 2 && text.ends_with("*/") {
        comment.end - 2
    } else {
        comment.end
    };
    if group.marker == Marker::Block {
        start += leading_run(source.get(start..end).unwrap_or_default(), '*');
        let body = source.get(start..end).unwrap_or_default();
        end -= body.len() - body.trim_end_matches('*').len();
    }
    let mut lines = Vec::new();
    let mut at = start;
    for line in source.get(start..end).unwrap_or_default().split('\n') {
        lines.push(trim_carriage_return(source, at..at + line.len()));
        at += line.len() + 1;
    }
    // rustdoc removes the `*` that starts the lines of a comment only when
    // every line has one.
    let starred = lines.iter().skip(1).all(|line| {
        let text = source.get(line.clone()).unwrap_or_default().trim_start();
        text.is_empty() || text.starts_with('*')
    });
    if starred {
        for line in lines.iter_mut().skip(1) {
            let text = source.get(line.clone()).unwrap_or_default();
            if text.trim_start().starts_with('*') {
                line.start += text.len() - text.trim_start().len() + 1;
            }
        }
    }
    lines
}

/// The bytes of the run of `c` that starts `text`.
fn leading_run(text: &str, c: char) -> usize {
    text.len() - text.trim_start_matches(c).len()
}

fn trim_carriage_return(source: &str, line: Range<usize>) -> Range<usize> {
    let text = source.get(line.clone()).unwrap_or_default();
    line.start..line.start + text.trim_end_matches('\r').len()
}

/// Comment text with markers stripped, and the file byte each of its bytes came
/// from.
struct Stripped {
    text: String,
    /// `map[i]` is the file byte of `text` byte `i`; one more entry ends the
    /// last line.
    map: Vec<usize>,
}

impl Stripped {
    /// The file ranges of a span of `text`, one for each line of it.
    ///
    /// A line with no text gives an empty range, so the ranges joined by a line
    /// feed read as the span.
    fn segments(&self, span: Range<usize>) -> Vec<Range<usize>> {
        let at = |index: usize| self.map.get(index).copied().unwrap_or_default();
        let mut start = span.start;
        self.text
            .get(span)
            .unwrap_or_default()
            .split('\n')
            .map(|line| {
                let end = start + line.len();
                let first = at(start);
                let last = if line.is_empty() {
                    first
                } else {
                    at(end - 1) + 1
                };
                start = end + 1;
                first..last
            })
            .collect()
    }
}

/// Joins `lines` into one text and removes the indentation they all share, as
/// rustdoc does.
fn strip(source: &str, lines: &[Range<usize>]) -> Stripped {
    let text_of = |line: &Range<usize>| source.get(line.clone()).unwrap_or_default();
    let indent = lines
        .iter()
        .map(text_of)
        .filter(|text| !text.trim().is_empty())
        .map(|text| text.len() - text.trim_start_matches([' ', '\t']).len())
        .min()
        .unwrap_or_default();
    let mut text = String::new();
    let mut map = Vec::new();
    let mut previous_end = lines.first().map_or(0, |line| line.start);
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            text.push('\n');
            map.push(previous_end);
        }
        let start = if text_of(line).trim().is_empty() {
            line.end
        } else {
            line.start + indent
        };
        // The padding before a closing `*/` is not text.
        let end = if index + 1 == lines.len() {
            start
                + source
                    .get(start..line.end)
                    .unwrap_or_default()
                    .trim_end()
                    .len()
        } else {
            line.end
        };
        text.push_str(source.get(start..end).unwrap_or_default());
        map.extend(start..end);
        previous_end = end;
    }
    map.push(previous_end);
    Stripped { text, map }
}
