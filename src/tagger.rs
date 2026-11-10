//! Part-of-speech tags for the words of one sentence.
//!
//! The tagger is an averaged perceptron over the Penn Treebank tags, a port of
//! the `tag` and `tokenize` packages of [prose] at `b33e60d`, with the English
//! model it ships.
//! [`Tagger::english()`] reads the model once per process, and
//! [`Tagger::tag()`] tokenizes one sentence and tags each token.
//!
//! The model is trained on UD English EWT and is under CC BY-SA 4.0.
//! Its notice is in `data/tagger/NOTICE.md`.
//!
//! [prose]: https://github.com/jdkato/prose/tree/b33e60dbff0492289b227f3681120ac057d8131b

mod features;
mod model;
mod tokens;

#[cfg(test)]
mod tests;

use std::ops::Range;
use std::sync::LazyLock;

pub use model::FormatError;
use model::Model;
use regex::Regex;

/// The English model, in prose's flat format.
const ENGLISH: &[u8] = include_bytes!("../data/tagger/en.bin");

/// The English tagger, read once per process.
static ENGLISH_TAGGER: LazyLock<Result<Tagger, FormatError>> =
    LazyLock::new(|| Tagger::from_bytes(ENGLISH));

/// Corpus marks for traces and elisions, such as `*-1` and `*T*-2`, which the
/// model never scores.
#[expect(
    clippy::expect_used,
    reason = "The pattern is a constant that compiles."
)]
static TRACE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:0|\*[0-9A-Za-z_?]\*|\*-[0-9]{1,3}|\*[A-Z]+\*-[0-9]{1,3}|\*)$")
        .expect("the trace pattern compiles")
});

/// Corpus brackets such as `-LRB-`, which are their own tag.
#[expect(
    clippy::expect_used,
    reason = "The pattern is a constant that compiles."
)]
static BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-[A-Z]{3}-$").expect("the bracket pattern compiles"));

/// A token of a sentence and its tag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tagged<'a> {
    /// The byte range of the token in the sentence.
    pub range: Range<usize>,
    /// The Penn Treebank tag, such as `NN` or `MD`.
    pub tag: &'a str,
}

/// An averaged perceptron over the Penn Treebank tags.
#[derive(Debug)]
pub struct Tagger {
    model: Model,
}

impl Tagger {
    /// The English tagger that ships in the binary.
    ///
    /// The model is read once per process.
    ///
    /// # Errors
    ///
    /// Returns the [`FormatError`] of the embedded model. A test reads it, so
    /// a released binary does not return one.
    pub fn english() -> Result<&'static Self, &'static FormatError> {
        LazyLock::force(&ENGLISH_TAGGER).as_ref()
    }

    /// Reads a tagger from the bytes of a model file.
    ///
    /// # Errors
    ///
    /// Returns a [`FormatError`] when the file is not a model, or the model is
    /// not a Penn Treebank model of the `runes` feature template.
    pub fn from_bytes(data: &[u8]) -> Result<Self, FormatError> {
        let model = Model::parse(data)?;
        if model.meta("tagset").is_some_and(|tagset| tagset != "penn") {
            return Err(FormatError::Unsupported("the tagset is not `penn`"));
        }
        if model.meta("template") != Some("1") {
            return Err(FormatError::Unsupported(
                "the feature template is not `runes`",
            ));
        }
        Ok(Self { model })
    }

    /// The tokens of `sentence`, each with its tag.
    ///
    /// The tags of the two words before a word are part of what the model
    /// reads, so `sentence` should be one sentence.
    #[must_use]
    pub fn tag<'a>(&'a self, sentence: &'a str) -> Vec<Tagged<'a>> {
        let ranges = tokens::tokens(sentence);
        let words: Vec<&str> = ranges
            .iter()
            .map(|range| sentence.get(range.clone()).unwrap_or_default())
            .collect();
        let mut context = vec![features::START.to_owned(), features::START2.to_owned()];
        context.extend(words.iter().map(|word| features::normalize(word)));
        context.push(features::END.to_owned());
        context.push(features::END2.to_owned());
        let mut previous = features::START;
        let mut before = features::START2;
        let mut tagged = Vec::with_capacity(ranges.len());
        for (index, (range, word)) in ranges.into_iter().zip(words).enumerate() {
            let tag = self.tag_one(&context, index + 2, word, previous, before);
            tagged.push(Tagged { range, tag });
            before = previous;
            previous = tag;
        }
        tagged
    }

    /// The tag of one word.
    fn tag_one<'a>(
        &'a self,
        context: &[String],
        at: usize,
        word: &'a str,
        previous: &str,
        before: &str,
    ) -> &'a str {
        if let Some(fixed) = Self::fixed(word) {
            return fixed;
        }
        if let Some(listed) = self.model.tag_for(word) {
            return listed;
        }
        let mut scores = vec![0.0_f32; self.model.class_count()];
        for feature in features::build(context, at, word, previous, before) {
            self.model.accumulate(feature.as_bytes(), &mut scores);
        }
        self.model.best(&scores).unwrap_or("")
    }

    /// The tag of a word the model never decides: a hyphen, an emoticon, a
    /// handle, a trace mark or a bracket.
    fn fixed(word: &str) -> Option<&str> {
        if word == "-" {
            Some("-")
        } else if tokens::is_emoticon(word) {
            Some("SYM")
        } else if word.starts_with('@') {
            Some("NN")
        } else if TRACE.is_match(word) {
            Some("-NONE-")
        } else if BRACKET.is_match(word) {
            Some(word)
        } else {
            None
        }
    }
}
