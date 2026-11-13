//! Punkt's decision: does the end mark of a token end a sentence?
//!
//! The decision follows Kiss and Strunk, "Unsupervised Multilingual Sentence
//! Boundary Detection" (Computational Linguistics 32(4), 2006), and reads the
//! four tables of the [`Model`]:
//!
//! 1. abbreviations, which a period does not end a sentence after;
//! 2. collocations, pairs a period between them does not split;
//! 3. sentence starters, words that often follow a sentence break;
//! 4. orthography, how a word was capitalized inside and between sentences.

use super::model::Model;
use super::token::{Mark, NUMBER, Token};

/// Whether `current`, which is followed by `next`, ends a sentence.
///
/// `after_number` is whether a number comes right before `current`.
pub(super) fn ends_sentence(
    model: &Model,
    current: &Token<'_>,
    next: &Token<'_>,
    after_number: bool,
) -> bool {
    if current.internal_after {
        return false;
    }
    match current.mark {
        Mark::None => false,
        Mark::Strong => true,
        Mark::Period | Mark::Ellipsis => after_period(model, current, next, after_number),
    }
}

/// The decision for a token that ends in a period or an ellipsis.
fn after_period(model: &Model, current: &Token<'_>, next: &Token<'_>, after_number: bool) -> bool {
    // `vs.` sets two things against each other, so a capital or a number after
    // it goes on with the same sentence.
    if current.is_versus() {
        return false;
    }
    let first = current.first_look(model.abbreviations());
    let next_first = next.first_look(model.abbreviations());
    let kind = current.kind_without_period();
    let next_kind = next.kind_without_sentence_period(next_first.sentence_break);
    // A collocation: the period belongs to an abbreviation here.
    if model.is_collocation(&kind, &next_kind) {
        return false;
    }
    let initial = current.is_initial();
    let starts = starts_sentence(model, next, &next_kind);
    // The abbreviation of a unit after a number, as in `5 ms.`, ends the
    // sentence before a capitalized word, which no name follows.
    if first.abbreviation && after_number && current.is_unit() && next.first_upper() {
        return true;
    }
    // An abbreviation or an ellipsis ends the sentence only when the next
    // word is known to start one.
    if (first.abbreviation || first.ellipsis) && !initial {
        if starts == Some(true) {
            return true;
        }
        if next.first_upper() && model.is_starter(&next_kind) {
            return true;
        }
    }
    // An initial or a number ends the sentence unless the next word is known
    // not to start one.
    if initial || kind == NUMBER {
        // A chain of initials, such as `J. R. R. Tolkien`, goes on.
        if initial && next.is_initial() {
            return false;
        }
        match starts {
            Some(false) => return false,
            None if initial && next.first_upper() && !model.orthography(&next_kind).has_lower() => {
                return false;
            }
            Some(true) | None => {}
        }
    }
    first.sentence_break
}

/// Whether the capitalization of `next` shows that it starts a sentence, shows
/// that it does not, or shows nothing.
fn starts_sentence(model: &Model, next: &Token<'_>, next_kind: &str) -> Option<bool> {
    // Punctuation on its own never starts a sentence. Quotes and brackets on
    // their own say nothing.
    if next.word == "."
        || (next.word.is_empty() && (next.mark != Mark::None || next.internal_after))
    {
        return Some(false);
    }
    let seen = model.orthography(next_kind);
    // Capitalized, seen in lower case, and never capitalized inside a
    // sentence: a sentence starter.
    if next.first_upper() && seen.has_lower() && !seen.middle_upper() {
        return Some(true);
    }
    // In lower case, and either seen capitalized or never seen in lower case
    // after a break: not a sentence starter.
    if next.first_lower() && (seen.has_upper() || !seen.begins_lower()) {
        return Some(false);
    }
    None
}
