//! Block and token ignores: text that a pattern hides from every rule that
//! reads prose.
//!
//! A port of `applyBlockPatterns` and `applyInlinePatterns` in `html.go` of
//! [Vale] at `66bbb85`, for Markdown and MDX.
//! Vale rewrites the file before it parses it. Each match of a block pattern
//! becomes a fenced code block that holds the first group of the match, and
//! each match of a token pattern becomes a code span that holds it.
//! The patterns run one after the other, each over the text the one before it
//! left, block patterns first.
//! hakkiri parses the rewritten text, and then gives every node of the tree
//! the bytes of the file the text came from, so an alert is still placed in the
//! file as written.
//!
//! [Vale]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/lint/html.go

#[cfg(test)]
mod tests;

use std::ops::Range;

use crate::error::Error;
use crate::markdown::{self, Flavor};
use crate::pattern::{FirstGroup, Pattern, SearchError};
use crate::tree::{Comment, Node, Scope, remap_nodes};

/// What a block pattern's match turns into, on each side of its first group.
const BLOCK: (&str, &str) = ("\n```\n", "\n```\n");

/// What a token pattern's match turns into, on each side of its first group.
const TOKEN: (&str, &str) = ("`", "`");

/// What stands in for the first group of a pattern that has none: Vale hands
/// the replacement `$1` to an engine that leaves it as written.
const NO_GROUP: &str = "$1";

/// A stretch of the rewritten text that holds bytes of the file.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Run {
    /// Where the stretch starts in the rewritten text.
    new: usize,
    /// Where it starts in the file.
    old: usize,
    /// How many bytes it holds.
    len: usize,
}

/// The text of a file after the patterns ran, and where each part of it came
/// from.
///
/// A byte that no run holds is one the rewrite added.
#[derive(Debug)]
pub(crate) struct Changed {
    text: String,
    runs: Vec<Run>,
}

impl Changed {
    /// `source` as it is, every byte from itself.
    fn new(source: &str) -> Self {
        let mut runs = Vec::new();
        push(&mut runs, 0, 0, source.len());
        Self {
            text: source.to_owned(),
            runs,
        }
    }

    /// The rewritten text.
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// This text with each match of each pattern of `patterns` turned into its
    /// first group between `around`, a pattern after the one before it.
    ///
    /// A match of no bytes changes nothing.
    fn apply(self, patterns: &[&Pattern], around: (&str, &str)) -> Result<Self, SearchError> {
        let mut changed = self;
        for pattern in patterns {
            changed = changed.replace(pattern, around)?;
        }
        Ok(changed)
    }

    fn replace(
        self,
        pattern: &Pattern,
        (before, after): (&str, &str),
    ) -> Result<Self, SearchError> {
        let found = pattern.find_replaceable(&self.text)?;
        if found.is_empty() {
            return Ok(self);
        }
        let mut text = String::with_capacity(self.text.len());
        let mut runs = Vec::new();
        let mut at = 0;
        for hit in found {
            self.copy(at..hit.whole.start, &mut text, &mut runs);
            text.push_str(before);
            match hit.first {
                FirstGroup::Missing => text.push_str(NO_GROUP),
                FirstGroup::Skipped => {}
                FirstGroup::At(group) => self.copy(group, &mut text, &mut runs),
            }
            text.push_str(after);
            at = hit.whole.end;
        }
        self.copy(at..self.text.len(), &mut text, &mut runs);
        Ok(Self { text, runs })
    }

    /// Appends the bytes `range` of this text to `text`, and the runs they
    /// hold to `runs`.
    fn copy(&self, range: Range<usize>, text: &mut String, runs: &mut Vec<Run>) {
        let start = text.len();
        text.push_str(self.text.get(range.clone()).unwrap_or_default());
        for (new, old) in self.overlap(&range) {
            push(runs, start + new.start - range.start, old, new.len());
        }
    }

    /// The part of each run that lies in `range`: its bytes in the rewritten
    /// text, and where the first one is in the file.
    fn overlap(&self, range: &Range<usize>) -> impl Iterator<Item = (Range<usize>, usize)> {
        let first = self
            .runs
            .partition_point(|run| run.new + run.len <= range.start);
        self.runs
            .get(first..)
            .unwrap_or_default()
            .iter()
            .take_while(|run| run.new < range.end)
            .map(|run| {
                let start = run.new.max(range.start);
                let end = (run.new + run.len).min(range.end);
                (start..end, run.old + start - run.new)
            })
    }

    /// The bytes of the file that the bytes `range` of the rewritten text came
    /// from, in file order. A byte the rewrite added gives none, and an empty
    /// range gives the empty range at the place it stands.
    pub(crate) fn origin(&self, range: Range<usize>) -> Vec<Range<usize>> {
        if range.is_empty() {
            let first = self
                .runs
                .partition_point(|run| run.new + run.len < range.start);
            return self
                .runs
                .get(first)
                .filter(|run| run.new <= range.start)
                .map(|run| {
                    let at = run.old + range.start - run.new;
                    at..at
                })
                .into_iter()
                .collect();
        }
        self.overlap(&range)
            .map(|(new, old)| old..old + new.len())
            .collect()
    }
}

/// Appends a run, joined to the last one when it continues it.
fn push(runs: &mut Vec<Run>, new: usize, old: usize, len: usize) {
    if len == 0 {
        return;
    }
    if let Some(last) = runs.last_mut()
        && last.new + last.len == new
        && last.old + last.len == old
    {
        last.len += len;
        return;
    }
    runs.push(Run { new, old, len });
}

/// `source` after the block patterns and then the token patterns ran.
///
/// # Errors
///
/// Returns a [`SearchError`] when a pattern that backtracks gives up.
pub(crate) fn rewrite(
    source: &str,
    blocks: &[&Pattern],
    tokens: &[&Pattern],
) -> Result<Changed, SearchError> {
    Changed::new(source)
        .apply(blocks, BLOCK)?
        .apply(tokens, TOKEN)
}

/// Replaces the place of each segment of `nodes`, and of the nodes below them,
/// by the bytes of the file it came from.
///
/// A node whose text the rewrite added gives way to its children.
pub(crate) fn restore(nodes: &mut Vec<Node>, changed: &Changed) {
    remap_nodes(nodes, &|span| changed.origin(span), &|at| {
        changed.origin(at..at).first().map(|place| place.start)
    });
}

/// The tree of a whole Markdown or MDX file that the patterns rewrote: one
/// `raw` root over the prose nodes, and the number of its code blocks.
///
/// # Errors
///
/// Returns an [`Error`] when a pattern gives up, or the MDX parser refuses the
/// rewritten text.
pub(crate) fn document(
    source: &str,
    flavor: Flavor,
    blocks: &[&Pattern],
    tokens: &[&Pattern],
) -> Result<(Vec<Node>, usize, Vec<Comment>), Error> {
    if blocks.is_empty() && tokens.is_empty() {
        return markdown::document(source, flavor);
    }
    let changed = rewrite(source, blocks, tokens).map_err(Error::Ignore)?;
    let mut built = markdown::build(changed.text(), flavor)?;
    restore(&mut built.nodes, &changed);
    // A comment the rewrite added or hid is no comment of the file.
    let comments = built
        .comments
        .into_iter()
        .filter_map(|comment| {
            let at = changed.origin(comment.at..comment.at + 1).first()?.start;
            Some(Comment { at, ..comment })
        })
        .collect();
    Ok((
        vec![Node::new(Scope::Raw, 0..source.len(), built.nodes)],
        built.code_blocks,
        comments,
    ))
}
