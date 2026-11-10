//! hakkiri, a prose linter for Markdown, MDX and source comments.
//!
//! [`parse`] turns one file into a [`Tree`].
//! Each [`Node`] holds a [`Scope`], the byte span of its text in the file, and
//! its children. Rules select text by scope, and alerts point back into the
//! file through the spans.

mod comments;
mod error;
mod jsdoc;
mod language;
mod markdown;
mod tree;

pub use error::Error;
pub use language::Language;
pub use tree::{Node, Scope, Tree};

/// Parses `source`, the text of a file in `language`, into its document tree.
///
/// A Markdown or MDX file gives one `raw` root that spans the whole file.
/// A source file gives one `raw` root per comment group, so no string literal
/// reaches the tree.
/// A group is one comment, or adjacent line comments that share a marker.
///
/// # Errors
///
/// Returns an [`Error`] when an MDX file has a syntax error, or when the
/// grammar of a source language cannot load or cannot parse the file.
pub fn parse(language: Language, source: &str) -> Result<Tree, Error> {
    let roots = match language {
        Language::Markdown => markdown::document(source, markdown::Flavor::Markdown)?,
        Language::Mdx => markdown::document(source, markdown::Flavor::Mdx)?,
        Language::Rust => comments::roots(comments::Dialect::Rust, source)?,
        Language::TypeScript => comments::roots(comments::Dialect::TypeScript, source)?,
        Language::Tsx => comments::roots(comments::Dialect::Tsx, source)?,
        Language::JavaScript => comments::roots(comments::Dialect::JavaScript, source)?,
    };
    Ok(Tree { roots })
}
