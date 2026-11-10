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
pub mod dictionary;
pub mod engine;
mod error;
pub mod formula;
mod html;
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
pub mod text;
mod tree;
pub mod words;

pub use error::Error;
pub use language::Language;
pub use tree::{Node, Scope, Tree};

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
    let (roots, code_blocks) = match language {
        Language::Markdown => markdown::document(source, markdown::Flavor::Markdown)?,
        Language::Mdx => markdown::document(source, markdown::Flavor::Mdx)?,
        Language::Rust => (comments::roots(comments::Dialect::Rust, source)?, 0),
        Language::TypeScript => (comments::roots(comments::Dialect::TypeScript, source)?, 0),
        Language::Tsx => (comments::roots(comments::Dialect::Tsx, source)?, 0),
        Language::JavaScript => (comments::roots(comments::Dialect::JavaScript, source)?, 0),
    };
    Ok(Tree { roots, code_blocks })
}
