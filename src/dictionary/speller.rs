//! The dictionaries a `spelling` rule reads, and the words it accepts besides.
//!
//! A port of `Checker` in [Vale]'s `spell` package at `66bbb85`, which queries
//! each dictionary in turn.
//!
//! [Vale]: https://github.com/vale-cli/vale/tree/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/spell

use std::collections::HashSet;

use super::Dictionary;

/// One dictionary of a [`Speller`]: the built-in one, which every rule shares,
/// or one that the rule read from its package.
#[derive(Debug)]
pub enum Book {
    /// Vale's `en_US-web`, from the binary.
    Builtin(&'static Dictionary),
    /// A dictionary read from the rule's package.
    Package(Box<Dictionary>),
}

impl Book {
    fn dictionary(&self) -> &Dictionary {
        match self {
            Self::Builtin(dictionary) => dictionary,
            Self::Package(dictionary) => dictionary,
        }
    }
}

/// The dictionaries of a `spelling` rule and its lists of accepted words.
#[derive(Debug)]
pub struct Speller {
    books: Vec<Book>,
    listed: HashSet<String>,
}

impl Speller {
    /// A speller over `books`, in the order a rule loads them, that also
    /// accepts the words in `listed`.
    #[must_use]
    pub const fn new(books: Vec<Book>, listed: HashSet<String>) -> Self {
        Self { books, listed }
    }

    /// `word` as each dictionary's `ICONV` pairs convert it, one after the
    /// other.
    #[must_use]
    pub fn convert(&self, word: &str) -> String {
        self.books.iter().fold(word.to_owned(), |text, book| {
            book.dictionary().convert(&text)
        })
    }

    /// Whether a dictionary accepts `word`, or a list holds it.
    #[must_use]
    pub fn spell(&self, word: &str) -> bool {
        self.books
            .iter()
            .any(|book| book.dictionary().spell(word, &self.listed))
    }

    /// The one spelling the first dictionary that has one gives for `word` in
    /// another case, such as `GitHub` for `github`.
    #[must_use]
    pub fn recase(&self, word: &str) -> Option<String> {
        self.books
            .iter()
            .find_map(|book| book.dictionary().recase(word, &self.listed))
    }
}
