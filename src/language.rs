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
    /// Plain text: the file is prose, read as written, in paragraphs set apart
    /// by blank lines. No extension names it, so a file reads this way when
    /// nothing else reads it, or when the config maps it here.
    Text,
}

/// The reader names a config's `formats` takes, with the language each gives.
pub const READERS: [(&str, Language); 7] = [
    ("md", Language::Markdown),
    ("mdx", Language::Mdx),
    ("rs", Language::Rust),
    ("ts", Language::TypeScript),
    ("tsx", Language::Tsx),
    ("js", Language::JavaScript),
    ("text", Language::Text),
];

impl Language {
    /// The language of the file at `path`, chosen by its extension, or `None`
    /// when no extension names one.
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

    /// The language a reader name gives: `md`, `mdx`, `rs`, `ts`, `tsx`, `js`
    /// or `text`.
    #[must_use]
    pub fn from_reader(name: &str) -> Option<Self> {
        READERS
            .iter()
            .find_map(|(reader, language)| (*reader == name).then_some(*language))
    }

    /// The extension a file read as this language takes in the path a glob
    /// matches: the name of its reader, and `txt` for plain text.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Mdx => "mdx",
            Self::Rust => "rs",
            Self::TypeScript => "ts",
            Self::Tsx => "tsx",
            Self::JavaScript => "js",
            Self::Text => "txt",
        }
    }
}
