//! The `sequence` check: runs of words that a rule's tokens match in order.
//!
//! The walk is Vale's ([sequence.go]).
//! It tags one sentence, picks the anchor step of the rule, and from each word
//! the anchor matches it walks left and right over the other steps.
//! A step of `skip` is optional: a word that does not match it is passed over.
//!
//! [sequence.go]: https://github.com/vale-cli/vale/blob/66bbb85a1a78a5b5497331e04d4033357caced4d/internal/check/sequence.go#L343-L497

pub(crate) mod upos;

use std::collections::HashMap;
use std::ops::Range;

use crate::pattern::SearchError;
use crate::rule::{Sequence, Token};
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
    /// Whether the word reads a code span, which is code and no word of the
    /// prose.
    code: bool,
    /// For a code span, the noun that heads the phrase it belongs to.
    head: Option<Head<'a>>,
}

/// The noun that a code span stands for in a run.
struct Head<'a> {
    text: &'a str,
    tag: &'a str,
    range: Range<usize>,
}

/// Where a walk found a match.
struct Found {
    /// The first and last words of the match.
    first: usize,
    last: usize,
    /// The word each step read, by step.
    at: HashMap<usize, usize>,
}

/// Whether `word` is one that `token` accepts.
///
/// A code span is no word that a token can name, by its tag or its text.
/// A negated token holds at it.
/// A token that reads one word reads the noun that heads the phrase the span
/// belongs to, when there is one, as in the `notice` of `the notice for
/// `cmd-7` says`.
fn accepts(token: &Token, word: &Word<'_>) -> Result<bool, SearchError> {
    if word.code {
        if token.negate {
            return Ok(true);
        }
        let Some(head) = word.head.as_ref().filter(|_| token.copies <= 1) else {
            return Ok(false);
        };
        return accepts(
            token,
            &Word {
                text: head.text,
                tag: head.tag,
                range: head.range.clone(),
                code: false,
                head: None,
            },
        );
    }
    let tag_failed = match &token.tag {
        Some(tag) => tag.is_match(word.tag)?,
        None => true,
    } == token.negate;
    let word_failed = match &token.word {
        Some(pattern) => pattern.is_match(word.text)? == token.negate,
        None => false,
    };
    if let Some(words) = &token.words
        && words.is_match(word.text)? == token.negate
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
fn negated(sequence: &Sequence, steps: Range<usize>) -> bool {
    if steps.is_empty() {
        return true;
    }
    (sequence.token_of(steps.start)..=sequence.token_of(steps.end - 1))
        .filter(|token| !sequence.block(*token).is_empty())
        .all(|token| sequence.tokens[token].negate)
}

/// Walks the steps from the anchor step, which reads the word `at`.
fn walk(sequence: &Sequence, words: &[Word<'_>], at: usize) -> Result<Option<Found>, SearchError> {
    let anchor = sequence.anchor;
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
            if negated(sequence, 0..step) {
                break;
            }
            return Ok(None);
        };
        let current = sequence.step(step - 1);
        let token = &sequence.tokens[current.token];
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
    while step < sequence.len() {
        if word >= words.len() {
            if negated(sequence, step..sequence.len()) {
                break;
            }
            return Ok(None);
        }
        let current = sequence.step(step);
        first = Some(first.map_or(word, |first| first.min(word)));
        last = Some(word);
        read.insert(step, word);
        let hit = accepts(&sequence.tokens[current.token], &words[word])?;
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
    sequence: &Sequence,
    words: &[Word<'_>],
    cursor: &mut usize,
) -> Result<Option<usize>, SearchError> {
    let anchor = &sequence.tokens[sequence.token_of(sequence.anchor)];
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
fn targeted(sequence: &Sequence, found: &Found) -> Option<(usize, usize)> {
    let mut range: Option<(usize, usize)> = None;
    for (token, written) in sequence.tokens.iter().enumerate() {
        if !written.target {
            continue;
        }
        // A targeted step that read no word was passed over. The span cannot
        // be narrowed to it, so the match keeps its whole span.
        let block = sequence.block(token);
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

/// The byte where a match that starts at `word` begins: the head noun, for a
/// code span that stands for one.
fn start_of(word: &Word<'_>) -> usize {
    word.head
        .as_ref()
        .map_or(word.range.start, |head| head.range.start)
}

/// The tags that a determiner, an adjective or a number may have between a
/// preposition or a noun and the noun it belongs to.
const SKIPPED: [&str; 4] = ["DT", "PRP$", "JJ", "CD"];

/// The prepositions that attach a phrase to the noun before them.
const PREPOSITIONS: [&str; 13] = [
    "of", "in", "on", "at", "for", "with", "from", "by", "under", "within", "into", "over", "to",
];

/// The word before `at`, past the determiners, adjectives and numbers.
fn before_skipped(words: &[Word<'_>], at: usize) -> Option<usize> {
    words
        .get(..at)?
        .iter()
        .rposition(|word| !SKIPPED.contains(&word.tag))
}

/// The noun before `at`, past the determiners, adjectives and numbers, when it
/// is a word of the prose.
fn noun_before(words: &[Word<'_>], at: usize) -> Option<usize> {
    let index = before_skipped(words, at)?;
    let word = words.get(index)?;
    (word.tag.starts_with("NN") && !word.code).then_some(index)
}

/// The preposition before `at`, past the determiners, adjectives and numbers.
fn preposition_before(words: &[Word<'_>], at: usize) -> Option<usize> {
    let index = before_skipped(words, at)?;
    let word = words.get(index)?;
    PREPOSITIONS
        .iter()
        .any(|preposition| word.text.eq_ignore_ascii_case(preposition))
        .then_some(index)
}

/// The noun that heads the phrase the code span `at` belongs to.
///
/// It is the noun that a preposition before the span attaches it to, as in
/// `the notice for `cmd-7``.
/// The nouns that further prepositions attach that noun to come next, so the
/// head of `the lock of the instance at `paths`` is `lock`.
/// A noun right before the span is not one, since `the snapshot `request` asks
/// for` and `the scenario `A` says` read alike and only one is a name.
fn head_noun(words: &[Word<'_>], at: usize) -> Option<usize> {
    let mut head =
        preposition_before(words, at).and_then(|preposition| noun_before(words, preposition))?;
    while let Some(next) =
        preposition_before(words, head).and_then(|preposition| noun_before(words, preposition))
    {
        head = next;
    }
    Some(head)
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
    find_tagged(sequence, sentence, tagger.tag(sentence), |range| {
        text.overlaps_code(range)
    })
}

/// The runs of words in `sentence` that `sequence` matches, given the tokens of
/// the sentence and their tags.
///
/// `is_code` tells whether the bytes of a token read a code span.
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
    let mut words: Vec<Word<'_>> = tagged
        .into_iter()
        .map(|token| Word {
            text: sentence.get(token.range.clone()).unwrap_or_default(),
            tag: token.tag,
            code: is_code(&token.range),
            range: token.range,
            head: None,
        })
        .collect();
    let heads: Vec<Option<Head<'_>>> = (0..words.len())
        .map(|at| {
            let word = words.get(at)?;
            let head = words.get(head_noun(&words, at).filter(|_| word.code)?)?;
            Some(Head {
                text: head.text,
                tag: head.tag,
                range: head.range.clone(),
            })
        })
        .collect();
    for (word, head) in words.iter_mut().zip(heads) {
        word.head = head;
    }
    let tries = match &sequence.candidates {
        Some(candidates) => candidates.find_all(sentence)?.len(),
        None => words.len(),
    };
    let mut excluded: Option<Vec<Range<usize>>> = None;
    let mut cursor = 0;
    let mut matches = Vec::new();
    for _ in 0..tries {
        // A try with no word left to start from finds nothing.
        let Some(start) = next_start(sequence, &words, &mut cursor)? else {
            continue;
        };
        let Some(found) = walk(sequence, &words, start)? else {
            continue;
        };
        let (first, last) = targeted(sequence, &found).unwrap_or((found.first, found.last));
        let range = start_of(&words[first])..words[last].range.end;
        if excluded.is_none() {
            let mut spans = Vec::new();
            for exception in &sequence.exceptions {
                spans.extend(exception.find_all(sentence)?);
            }
            excluded = Some(spans);
        }
        if inside(excluded.as_deref().unwrap_or_default(), range.start) {
            continue;
        }
        matches.push(Matched {
            range,
            words: words[found.first..=found.last]
                .iter()
                .enumerate()
                .map(|(at, word)| match &word.head {
                    Some(head) if at == 0 => head.text.to_owned(),
                    _ => word.text.to_owned(),
                })
                .collect(),
        });
    }
    Ok(matches)
}

#[cfg(test)]
mod tests;
