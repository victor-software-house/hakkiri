//! Checks: what each check type finds in one block of text.

use std::ops::Range;

use crate::pattern::SearchError;
use crate::rule::{Check, Existence};
use crate::text::Text;

/// One match a check found in a block of text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Hit {
    /// The bytes of the match in the block's text.
    pub(crate) range: Range<usize>,
    /// What the message's `%s` take, in order.
    pub(crate) values: Vec<String>,
}

/// The matches `check` finds in `text`.
///
/// # Errors
///
/// Returns a [`SearchError`] when a pattern gives up on the text.
pub(crate) fn run(check: &Check, text: &Text) -> Result<Vec<Hit>, SearchError> {
    match check {
        Check::Existence(existence) => run_existence(existence, text),
        Check::Substitution(_) | Check::Occurrence(_) | Check::Metric(_) => Ok(Vec::new()),
    }
}

/// Each match of the tokens that no exception holds back and no code span
/// overlaps.
fn run_existence(existence: &Existence, text: &Text) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let found = existence.pattern.find_all(content)?;
    let phrases = match &existence.exceptions.phrases {
        Some(phrases) if !found.is_empty() => phrases.find_overlapping(content)?,
        _ => Vec::new(),
    };
    let mut hits = Vec::new();
    for range in found {
        if text.overlaps_code(&range) {
            continue;
        }
        let Some(matched) = content.get(range.clone()) else {
            continue;
        };
        if let Some(words) = &existence.exceptions.words
            && words.is_match(matched.trim())?
        {
            continue;
        }
        if phrases
            .iter()
            .any(|phrase| phrase.start <= range.start && range.end <= phrase.end)
        {
            continue;
        }
        hits.push(Hit {
            values: vec![matched.to_owned()],
            range,
        });
    }
    Ok(hits)
}
