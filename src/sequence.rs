//! The `sequence` check: runs of words that a rule's tokens match in order.
//!
//! The walk is Vale's ([sequence.go]).
//! It tags one sentence, picks the anchor step of the rule, and from each word
//! the anchor matches it walks left and right over the other steps.
//! A step of `skip` is optional: a word that does not match it is passed over.
//! A rule that lists several forms walks each one the same way and keeps the
//! matches together, an earlier match winning over a later one that it
//! overlaps.
//!
//! [sequence.go]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L343-L497

pub(crate) mod upos;

use std::collections::HashMap;
use std::ops::Range;

use crate::pattern::SearchError;
use crate::rule::{Form, Sequence, Token};
use crate::tagger::{Tagged, Tagger};
use crate::text::Text;

/// One run of words a rule's tokens matched.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Matched {
    /// The bytes of the alert in the sentence.
    pub(crate) range: Range<usize>,
    /// The words the match covers, in order.
    pub(crate) words: Vec<String>,
}

/// A word of the sentence and its tag.
struct Word<'a> {
    text: &'a str,
    tag: &'a str,
    range: Range<usize>,
    /// Whether the word reads a code span, a noun that a token reads as
    /// [`CODE_TEXT`].
    code: bool,
}

/// Where a walk found a match.
struct Found {
    /// The first and last words of the match.
    first: usize,
    last: usize,
    /// The word each step read, by step.
    at: HashMap<usize, usize>,
}

/// The text a pattern or a word list reads in place of a code span.
///
/// A code span names a thing, so a token reads it as a noun by its tag.
/// By its text it reads one capital letter: a pattern for any letter or word
/// takes it, as `ThingsSpeakNoun` does, and a pattern for a lowercase word
/// refuses it, as `NounStack` does.
const CODE_TEXT: &str = "X";

/// Whether `word` is one that `token` accepts.
///
/// A code span has the tag `NN` and reads as [`CODE_TEXT`].
fn accepts(token: &Token, word: &Word<'_>) -> Result<bool, SearchError> {
    let text = if word.code { CODE_TEXT } else { word.text };
    let tag_failed = match &token.tag {
        Some(tag) => tag.is_match(word.tag)?,
        None => true,
    } == token.negate;
    let word_failed = match &token.word {
        Some(pattern) => pattern.is_match(text)? == token.negate,
        None => false,
    };
    if let Some(words) = &token.words
        && words.is_match(text)? == token.negate
    {
        return Ok(false);
    }
    let has_word = token.word.is_some();
    let has_tag = token.tag.is_some();
    let refused = (!has_word && tag_failed)
        || (!has_tag && word_failed)
        || (has_tag && has_word && (tag_failed || word_failed));
    Ok(!refused)
}

/// Whether every one of `steps` is negated. A negated step holds where there
/// is no word at all, so it holds at the edge of the sentence.
fn negated(form: &Form, steps: Range<usize>) -> bool {
    if steps.is_empty() {
        return true;
    }
    (form.token_of(steps.start)..=form.token_of(steps.end - 1))
        .filter(|token| !form.block(*token).is_empty())
        .all(|token| form.tokens[token].negate)
}

/// Walks the steps from the anchor step, which reads the word `at`.
fn walk(form: &Form, words: &[Word<'_>], at: usize) -> Result<Option<Found>, SearchError> {
    let anchor = form.anchor;
    let mut first: Option<usize> = None;
    let mut last: Option<usize> = None;
    let mut read = HashMap::new();
    // The steps before the anchor, read from the anchor leftwards. A step of
    // `skip` is written for a walk to the right, so the left walk turns it
    // around: the required step becomes optional, and the first optional one
    // required.
    let (mut step, mut word) = (anchor, at);
    while step > 0 {
        let Some(before) = word.checked_sub(1) else {
            if negated(form, 0..step) {
                break;
            }
            return Ok(None);
        };
        let current = form.step(step - 1);
        let token = &form.tokens[current.token];
        first = Some(before);
        read.insert(step - 1, before);
        let optional = if token.skip > 0 {
            (current.optional || current.end) && !current.start
        } else {
            current.optional
        };
        let hit = accepts(token, &words[before])?;
        match (hit, optional) {
            (false, false) => return Ok(None),
            (true, true) => {
                // The word sat inside its window: the spare steps of the
                // window are done with, and the steps past them still hold.
                step = current.first;
                word = before;
            }
            _ => {
                step -= 1;
                word = before;
            }
        }
    }
    let (mut step, mut word) = (anchor, at);
    while step < form.len() {
        if word >= words.len() {
            if negated(form, step..form.len()) {
                break;
            }
            return Ok(None);
        }
        let current = form.step(step);
        first = Some(first.map_or(word, |first| first.min(word)));
        last = Some(word);
        read.insert(step, word);
        let hit = accepts(&form.tokens[current.token], &words[word])?;
        match (hit, current.optional) {
            (false, false) => return Ok(None),
            (true, true) => {
                step = current.after;
                word += 1;
            }
            _ => {
                step += 1;
                word += 1;
            }
        }
    }
    Ok(match (first, last) {
        (Some(first), Some(last)) if last >= first => Some(Found {
            first,
            last,
            at: read,
        }),
        _ => None,
    })
}

/// The next word from `cursor` that the anchor step accepts, which no earlier
/// walk started from.
///
/// Walks start from the words the anchor accepts in order, so the words before
/// `cursor` are the ones already taken or refused. Vale looks each start up
/// from the first word and in the list of earlier starts, which takes time
/// that grows with the cube of the words when the anchor matches most of them.
fn next_start(
    form: &Form,
    words: &[Word<'_>],
    cursor: &mut usize,
) -> Result<Option<usize>, SearchError> {
    let anchor = &form.tokens[form.token_of(form.anchor)];
    while let Some(word) = words.get(*cursor) {
        let index = *cursor;
        *cursor += 1;
        if accepts(anchor, word)? {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

/// The words from the first to the last of the `target` tokens, when every
/// step of such a token read a word.
fn targeted(form: &Form, found: &Found) -> Option<(usize, usize)> {
    let mut range: Option<(usize, usize)> = None;
    for (token, written) in form.tokens.iter().enumerate() {
        if !written.target {
            continue;
        }
        // A targeted step that read no word was passed over. The span cannot
        // be narrowed to it, so the match keeps its whole span.
        let block = form.block(token);
        let read: Vec<usize> = found
            .at
            .iter()
            .filter(|(step, _)| block.contains(step))
            .map(|(_, word)| *word)
            .collect();
        if read.len() != block.len() {
            return None;
        }
        for word in read {
            range = Some(range.map_or((word, word), |(lo, hi)| (lo.min(word), hi.max(word))));
        }
    }
    range
}

/// The scope items a `sequence` rule reads, narrowed to the sentences within
/// what it declared.
///
/// The rule reads tags a sentence at a time, so it always reads sentences, and
/// its `scope` says which. No scope means every sentence, and `list` means the
/// sentences of list items. An item that already names sentences stays as it
/// is. One that only leaves something out, such as `~list`, is joined to
/// `sentence`, and `paragraph` is `sentence`, since a sentence is always in a
/// paragraph ([`sentenceScope`][v-scope] in Vale).
///
/// [v-scope]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L593-L641
pub(crate) fn sentence_scope(declared: &[String]) -> Vec<String> {
    if declared.is_empty() {
        return vec!["sentence".to_owned()];
    }
    declared
        .iter()
        .map(|item| {
            if item == "sentence" || item.starts_with("sentence.") {
                item.clone()
            } else if item.starts_with('~') {
                format!("sentence&{item}")
            } else if let Some(rest) = item.strip_prefix("paragraph")
                && (rest.is_empty() || rest.starts_with('.'))
            {
                format!("sentence{rest}")
            } else {
                format!("sentence & {item}")
            }
        })
        .collect()
}

/// Whether `at`, the byte where a match begins, is inside one of `spans`.
fn inside(spans: &[Range<usize>], at: usize) -> bool {
    spans.iter().any(|span| span.contains(&at))
}

/// The runs of words in `sentence` that `sequence` matches.
///
/// # Errors
///
/// Returns a [`SearchError`] when a pattern gives up on a word or the
/// sentence.
pub(crate) fn find(sequence: &Sequence, text: &Text) -> Result<Vec<Matched>, SearchError> {
    // The model is embedded and loads; without it no word has a tag, and no
    // run is reported.
    let Ok(tagger) = Tagger::english() else {
        return Ok(Vec::new());
    };
    let sentence = text.as_str();
    let is_code = |range: &Range<usize>| text.overlaps_code(range);
    find_tagged(
        sequence,
        sentence,
        tagger.tag_code(sentence, is_code),
        is_code,
    )
}

/// The runs of words in `sentence` that `sequence` matches, given the tokens of
/// the sentence and their tags.
///
/// `is_code` tells whether the bytes of a token read a code span.
///
/// A rule with several forms matches where any form does. When two forms
/// match overlapping words, the match that starts first stays and the other
/// is dropped, so one stretch of words gives one alert. Matches of the same
/// form never drop each other, so a rule of one list gives Vale's matches.
///
/// # Errors
///
/// Returns a [`SearchError`] when a pattern gives up on a word or the
/// sentence.
fn find_tagged(
    sequence: &Sequence,
    sentence: &str,
    tagged: Vec<Tagged<'_>>,
    is_code: impl Fn(&Range<usize>) -> bool,
) -> Result<Vec<Matched>, SearchError> {
    let words: Vec<Word<'_>> = tagged
        .into_iter()
        .map(|token| Word {
            text: sentence.get(token.range.clone()).unwrap_or_default(),
            tag: token.tag,
            code: is_code(&token.range),
            range: token.range,
        })
        .collect();
    let mut excluded: Option<Vec<Range<usize>>> = None;
    let mut found: Vec<(usize, Matched)> = Vec::new();
    for (index, form) in sequence.forms.iter().enumerate() {
        for matched in find_form(sequence, form, sentence, &words, &mut excluded)? {
            found.push((index, matched));
        }
    }
    Ok(merged(found, sequence.forms.len()))
}

/// The runs of `words` that one form matches, with the exceptions of the rule
/// taken out. `excluded` holds the spans of the exceptions once a match asks
/// for them, and is shared by the forms of the rule.
fn find_form(
    sequence: &Sequence,
    form: &Form,
    sentence: &str,
    words: &[Word<'_>],
    excluded: &mut Option<Vec<Range<usize>>>,
) -> Result<Vec<Matched>, SearchError> {
    let tries = match &form.candidates {
        Some(candidates) => candidates.find_all(sentence)?.len(),
        None => words.len(),
    };
    let mut cursor = 0;
    let mut matches = Vec::new();
    for _ in 0..tries {
        // A try with no word left to start from finds nothing.
        let Some(start) = next_start(form, words, &mut cursor)? else {
            continue;
        };
        let Some(found) = walk(form, words, start)? else {
            continue;
        };
        let (first, last) = targeted(form, &found).unwrap_or((found.first, found.last));
        let range = words[first].range.start..words[last].range.end;
        if excluded.is_none() {
            let mut spans = Vec::new();
            for exception in &sequence.exceptions {
                spans.extend(exception.find_all(sentence)?);
            }
            *excluded = Some(spans);
        }
        if inside(excluded.as_deref().unwrap_or_default(), range.start) {
            continue;
        }
        matches.push(Matched {
            range,
            words: words[found.first..=found.last]
                .iter()
                .map(|word| word.text.to_owned())
                .collect(),
        });
    }
    Ok(matches)
}

/// The matches of every form as one list. One form's matches stay as they
/// were found. Several forms' matches go in order of where they start, the
/// earlier form first at the same place, and a match that overlaps one kept
/// from another form is dropped.
fn merged(mut found: Vec<(usize, Matched)>, forms: usize) -> Vec<Matched> {
    if forms < 2 {
        return found.into_iter().map(|(_, matched)| matched).collect();
    }
    found.sort_by_key(|(form, matched)| (matched.range.start, *form));
    let mut kept: Vec<(usize, Matched)> = Vec::new();
    for (form, matched) in found {
        let overlaps = kept.iter().any(|(other, earlier)| {
            *other != form
                && earlier.range.start < matched.range.end
                && matched.range.start < earlier.range.end
        });
        if !overlaps {
            kept.push((form, matched));
        }
    }
    kept.into_iter().map(|(_, matched)| matched).collect()
}

#[cfg(test)]
mod tests;
