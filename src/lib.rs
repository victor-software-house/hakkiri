//! hakkiri, a prose linter for Markdown, MDX and source comments.
//!
//! [`parse`] turns one file into a [`Tree`].
//! Each [`Node`] holds a [`Scope`], the byte span of its text in the file, and
//! its children. Rules select text by scope, and alerts point back into the
//! file through the spans.

pub mod alert;
mod capitalization;
mod case;
mod check;
mod comments;
pub mod control;
pub mod dictionary;
pub mod engine;
mod entity;
mod error;
pub mod formula;
mod html;
mod html_text;
mod ignore;
mod jsdoc;
mod language;
mod markdown;
mod metric;
pub mod pattern;
pub mod readability;
pub mod rule;
pub mod scope;
pub mod select;
mod sequence;
mod spelling;
pub mod splitter;
pub mod strcase;
pub mod summary;
pub mod tagger;
mod term;
pub mod text;
mod tree;
pub mod words;

pub use error::Error;
pub use language::Language;
use pattern::Pattern;
pub use tree::{Comment, Join, Node, Reading, Scope, Tree};

/// Parses `source`, the text of a file in `language`, into its document tree.
///
/// A Markdown or MDX file gives one `raw` root that spans the whole file, and
/// the tree counts its code blocks.
/// A source file gives one `raw` root per comment group, so no string literal
/// reaches the tree.
/// A group is one comment, or adjacent line comments that share a marker.
///
/// # Errors
///
/// Returns an [`Error`] when an MDX file has a syntax error, or when the
/// grammar of a source language cannot load or cannot parse the file.
pub fn parse(language: Language, source: &str) -> Result<Tree, Error> {
    parse_ignoring(language, source, &[], &[])
}

/// Parses `source` as [`parse`] does, with the text that the block and token
/// patterns match left out of the prose.
///
/// A Markdown or MDX file is read as Vale reads it under `BlockIgnores` and
/// `TokenIgnores`.
/// Each match of a `blocks` pattern becomes a fenced code block that holds its
/// first group, and each match of a `tokens` pattern becomes a code span that
/// holds it.
/// The patterns run in order, block patterns first, each over the text the one
/// before it left.
/// Every span of the tree is in `source`.
/// The Markdown of each comment of a source file is rewritten the same way.
///
/// # Errors
///
/// Returns an [`Error`] when an MDX file has a syntax error, when the grammar
/// of a source language cannot load or cannot parse the file, or when a
/// pattern that backtracks gives up on the file.
pub fn parse_ignoring(
    language: Language,
    source: &str,
    blocks: &[&Pattern],
    tokens: &[&Pattern],
) -> Result<Tree, Error> {
    let (roots, code_blocks, comments) = match language {
        Language::Markdown => ignore::document(source, markdown::Flavor::Markdown, blocks, tokens)?,
        Language::Mdx => ignore::document(source, markdown::Flavor::Mdx, blocks, tokens)?,
        Language::Rust => (
            comments::roots(comments::Dialect::Rust, source, blocks, tokens)?,
            0,
            Vec::new(),
        ),
        Language::TypeScript => (
            comments::roots(comments::Dialect::TypeScript, source, blocks, tokens)?,
            0,
            Vec::new(),
        ),
        Language::Tsx => (
            comments::roots(comments::Dialect::Tsx, source, blocks, tokens)?,
            0,
            Vec::new(),
        ),
        Language::JavaScript => (
            comments::roots(comments::Dialect::JavaScript, source, blocks, tokens)?,
            0,
            Vec::new(),
        ),
    };
    let mut roots = roots;
    // The text after a comment that turns every rule off is no one's prose.
    if let Some(root) = roots.first_mut() {
        control::cut(&mut root.children, &comments, source.len());
    }
    Ok(Tree {
        roots,
        code_blocks,
        comments,
    })
}
