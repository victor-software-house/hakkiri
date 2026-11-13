//! The errors a parse reports.

use std::{error, fmt};

use markdown::message::Message;

use crate::pattern::SearchError;

/// Why a file gave no tree.
#[derive(Debug)]
pub enum Error {
    /// The MDX parser refused the file.
    Mdx(Message),
    /// A tree-sitter grammar could not load.
    Grammar(tree_sitter::LanguageError),
    /// The comment query did not compile.
    Query(tree_sitter::QueryError),
    /// tree-sitter gave no tree for the file.
    Syntax,
    /// A block or token pattern gave up on the file.
    Ignore(SearchError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mdx(message) => write!(f, "MDX syntax error: {message}"),
            Self::Grammar(error) => write!(f, "cannot load the grammar: {error}"),
            Self::Query(error) => write!(f, "cannot compile the comment query: {error}"),
            Self::Syntax => f.write_str("tree-sitter gave no syntax tree"),
            Self::Ignore(error) => write!(f, "an ignore pattern failed: {error}"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Mdx(_) | Self::Syntax => None,
            Self::Grammar(error) => Some(error),
            Self::Query(error) => Some(error),
            Self::Ignore(error) => Some(error),
        }
    }
}

impl From<tree_sitter::LanguageError> for Error {
    fn from(error: tree_sitter::LanguageError) -> Self {
        Self::Grammar(error)
    }
}

impl From<tree_sitter::QueryError> for Error {
    fn from(error: tree_sitter::QueryError) -> Self {
        Self::Query(error)
    }
}
