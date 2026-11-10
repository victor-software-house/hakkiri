//! Sentence splitting: where a sentence ends in the text of one paragraph,
//! heading or table cell.
//!
//! The splitter is Punkt over a trained [`Model`], then a few override rules
//! for the cases Punkt is known to miss.
//! [`train()`] builds a model from text.

mod model;
mod punkt;
mod rules;
mod token;
mod train;

#[cfg(test)]
mod tests;

use std::ops::Range;

pub use model::{FormatError, Model};
pub use train::{Summary, Trained, train};

impl Model {
    /// Cuts `text` into sentences, and gives the byte range of each.
    ///
    /// A range has no whitespace at its ends. A closing quote or bracket right
    /// after an end mark belongs to the sentence the mark ends.
    /// Text with no word gives no range.
    ///
    /// The cost is linear in the length of `text`.
    #[must_use]
    pub fn split(&self, text: &str) -> Vec<Range<usize>> {
        let tokens = token::tokens(text);
        let brackets = rules::Brackets::new(text);
        let mut ends = vec![false; tokens.len()];
        // Whether the token being read opens a sentence, or follows a colon.
        let mut opens = true;
        for (index, pair) in tokens.windows(2).enumerate() {
            let [current, next] = pair else { continue };
            let numbers_an_item = opens && current.is_list_marker();
            let ends_here = !numbers_an_item
                && punkt::ends_sentence(self, current, next)
                && !brackets.encloses(current.span.end)
                && !rules::continues_after_ellipsis(current, next);
            if let Some(slot) = ends.get_mut(index) {
                *slot = ends_here;
            }
            opens = ends_here || current.colon_after || (opens && current.is_symbol());
        }
        let mut sentences = Vec::new();
        let mut first = 0;
        let mut index = 0;
        while index < tokens.len() {
            if !ends.get(index).is_some_and(|&ends_here| ends_here) {
                index += 1;
                continue;
            }
            let mut last = index;
            while tokens
                .get(last + 1)
                .is_some_and(|token| token.is_closing_only(text))
            {
                last += 1;
            }
            if let (Some(start), Some(end)) = (tokens.get(first), tokens.get(last)) {
                sentences.push(start.span.start..end.span.end);
            }
            first = last + 1;
            index = first;
        }
        if let (Some(start), Some(end)) = (tokens.get(first), tokens.last()) {
            sentences.push(start.span.start..end.span.end);
        }
        sentences
    }
}
