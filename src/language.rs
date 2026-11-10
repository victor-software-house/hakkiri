//! The languages hakkiri reads.

use std::ffi::OsStr;
use std::path::Path;

/// A language hakkiri builds a tree for.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Language {
    /// Markdown with GFM and front matter, from `.md`.
    Markdown,
    /// MDX, from `.mdx`.
    Mdx,
    /// Rust, from `.rs`.
    Rust,
    /// TypeScript, from `.ts`.
    TypeScript,
    /// TypeScript with JSX, from `.tsx`.
    Tsx,
    /// JavaScript with JSX, from `.js`, `.jsx`, `.mjs` and `.cjs`.
    JavaScript,
}

impl Language {
    /// The language of the file at `path`, chosen by its extension.
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(OsStr::to_str)? {
            "md" => Some(Self::Markdown),
            "mdx" => Some(Self::Mdx),
            "rs" => Some(Self::Rust),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            _ => None,
        }
    }
}
