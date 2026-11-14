//! The languages hakkiri reads.
//!
//! A reader is one of the seven parsers hakkiri has, and each reads one
//! language of [langdex], the table of languages with their names, aliases,
//! extensions, file names and interpreters.

use std::path::Path;

/// A language hakkiri builds a tree for.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Language {
    /// Markdown with GFM and front matter.
    Markdown,
    /// MDX.
    Mdx,
    /// Rust.
    Rust,
    /// TypeScript.
    TypeScript,
    /// TypeScript with JSX.
    Tsx,
    /// JavaScript with JSX.
    JavaScript,
    /// Plain text: the file is prose, read as written, in paragraphs set apart
    /// by blank lines. A file reads this way when its language is Text, when
    /// nothing else reads it, or when the config maps it here.
    Text,
}

impl Language {
    /// Every reader, in the order the `formats` error names them.
    pub const ALL: [Self; 7] = [
        Self::Markdown,
        Self::Mdx,
        Self::Rust,
        Self::TypeScript,
        Self::Tsx,
        Self::JavaScript,
        Self::Text,
    ];

    /// The name of the langdex language this reader reads.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown",
            Self::Mdx => "MDX",
            Self::Rust => "Rust",
            Self::TypeScript => "TypeScript",
            Self::Tsx => "TSX",
            Self::JavaScript => "JavaScript",
            Self::Text => "Text",
        }
    }

    /// The reader of `language`, or `None` when hakkiri has none for it.
    #[must_use]
    pub fn of(language: &langdex::Language) -> Option<Self> {
        match language.name {
            "Markdown" => Some(Self::Markdown),
            "MDX" => Some(Self::Mdx),
            "Rust" => Some(Self::Rust),
            "TypeScript" => Some(Self::TypeScript),
            "TSX" => Some(Self::Tsx),
            "JavaScript" => Some(Self::JavaScript),
            "Text" => Some(Self::Text),
            _ => None,
        }
    }

    /// The langdex language this reader reads.
    #[must_use]
    pub fn langdex(self) -> Option<&'static langdex::Language> {
        langdex::all()
            .iter()
            .find(|language| language.name == self.name())
    }

    /// The reader of the file at `path`, chosen by its langdex language: its
    /// file name, then its extension. `None` when no language has a reader.
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        langdex::by_path(path.to_str()?).and_then(Self::of)
    }

    /// The reader a `formats` name gives: any name, alias or extension of the
    /// language it reads, such as `md`, `markdown` or `Markdown`.
    #[must_use]
    pub fn from_reader(name: &str) -> Option<Self> {
        langdex::by_name(name).and_then(Self::of)
    }

    /// The extension of a file read as this language, without its dot: the
    /// first extension of its langdex language.
    #[must_use]
    pub fn extension(self) -> &'static str {
        self.langdex()
            .and_then(|language| language.extensions.first())
            .map_or("", |extension| extension.trim_start_matches('.'))
    }
}

#[cfg(test)]
mod tests;
