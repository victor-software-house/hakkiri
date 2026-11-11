//! Checks: what each check type finds in one block of text.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use crate::alert::fill;
use crate::capitalization::evaluate;
use crate::metric::{Variables, graded, measure, variables};
use crate::pattern::{Captured, Pattern, SearchError};
use crate::rule::{
    Capitalization, Check, Conditional, Consistency, Exceptions, Existence, Metric, Occurrence,
    Readability, Repetition, Rule, Sequence, Substitution, Swap, term_pattern,
};
use crate::sequence::find;
use crate::spelling::run as run_spelling;
use crate::splitter::Model;
use crate::summary::{Elements, Summary};
use crate::term::recase_to_term;
use crate::text::Text;

/// One match a check found in a block of text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Hit {
    /// The bytes of the match in the block's text.
    pub(crate) range: Range<usize>,
    /// What the message's `%s` take, in order.
    pub(crate) values: Vec<String>,
    /// Whether the message reads `%d` as `%s`.
    pub(crate) numbered: bool,
    /// The message to use in place of the rule's, when the check has more
    /// detail than the rule's message does.
    pub(crate) message: Option<String>,
}

impl Hit {
    /// The message and the description of `rule`, filled with the values.
    pub(crate) fn messages(&self, rule: &Rule) -> (String, String) {
        let values: Vec<&str> = self.values.iter().map(String::as_str).collect();
        let read = |template: &str| {
            if self.numbered {
                fill(&template.replace("%d", "%s"), &values)
            } else {
                fill(template, &values)
            }
        };
        let message = self.message.clone().unwrap_or_else(|| read(&rule.message));
        (message, read(&rule.description))
    }
}

/// What one rule remembers from the blocks of one file it has read.
///
/// A rule starts each file with none, so two rules never share it.
#[derive(Debug, Default)]
pub(crate) struct Memory {
    /// The pairs of a `consistency` rule, by position, and for each the forms
    /// seen so far, as 0 or 1.
    seen: HashSet<(usize, usize)>,
    /// The terms a `conditional` rule has read defined so far, lowercased when
    /// the rule ignores case.
    terms: HashSet<String>,
}

/// The matches `check` finds in `text`, the next block of a file whose earlier
/// blocks `memory` holds and whose extension, without the dot, is `extension`.
///
/// `accepted` holds the exceptions of the vocabularies the file's globs name.
/// An `existence`, `repetition`, `spelling` or `substitution` rule holds back
/// the matches they name, besides its own exceptions, whatever it sets for
/// `vocab` and `nonword`.
///
/// # Errors
///
/// Returns a [`SearchError`] when a pattern gives up on the text.
pub(crate) fn run(
    check: &Check,
    text: &Text,
    memory: &mut Memory,
    extension: &str,
    accepted: &[&Exceptions],
) -> Result<Vec<Hit>, SearchError> {
    match check {
        Check::Existence(existence) => run_existence(existence, text, accepted),
        Check::Substitution(substitution) => run_substitution(substitution, text, accepted),
        Check::Occurrence(occurrence) => run_occurrence(occurrence, text),
        Check::Metric(metric) => run_metric(metric, text),
        Check::Consistency(consistency) => run_consistency(consistency, text, memory),
        Check::Conditional(conditional) => run_conditional(conditional, text, memory),
        Check::Repetition(repetition) => run_repetition(repetition, text, accepted),
        Check::Readability(readability) => Ok(run_readability(readability, text)),
        Check::Capitalization(capitalization) => Ok(run_capitalization(capitalization, text)),
        Check::Sequence(sequence) => run_sequence(sequence, text),
        Check::Spelling(spelling) => Ok(run_spelling(spelling, text, extension, accepted)),
    }
}

/// The spans of the multi-word exceptions in a text, ready to ask whether a
/// range lies inside one.
#[derive(Debug, Default)]
pub(crate) struct PhraseSpans {
    /// The start of each span, in order.
    starts: Vec<usize>,
    /// For each span, the furthest end among it and the spans before it.
    reach: Vec<usize>,
}

impl PhraseSpans {
    fn new(mut spans: Vec<Range<usize>>) -> Self {
        spans.sort_by_key(|span| span.start);
        let mut furthest = 0;
        let reach = spans
            .iter()
            .map(|span| {
                furthest = furthest.max(span.end);
                furthest
            })
            .collect();
        Self {
            starts: spans.iter().map(|span| span.start).collect(),
            reach,
        }
    }

    /// Whether `range` lies inside a span: one starts at or before it, and
    /// none of the spans that do ends past it.
    fn hold(&self, range: &Range<usize>) -> bool {
        let before = self.starts.partition_point(|start| *start <= range.start);
        before
            .checked_sub(1)
            .and_then(|last| self.reach.get(last))
            .is_some_and(|end| range.end <= *end)
    }
}

/// The spans of the multi-word exceptions in `content`, when `found` holds a
/// match to check them against.
///
/// Each exception is searched on its own, so two that overlap or nest both
/// give their spans.
fn phrase_spans(
    exceptions: &Exceptions,
    content: &str,
    found: bool,
) -> Result<PhraseSpans, SearchError> {
    let mut spans = Vec::new();
    if found {
        for phrase in &exceptions.phrases {
            spans.extend(phrase.find_overlapping(content)?);
        }
    }
    Ok(PhraseSpans::new(spans))
}

/// Whether an exception holds back the match `range`, which reads `observed`.
fn excepted(
    exceptions: &Exceptions,
    phrases: &PhraseSpans,
    range: &Range<usize>,
    observed: &str,
) -> Result<bool, SearchError> {
    if let Some(words) = &exceptions.words
        && words.is_match(observed)?
    {
        return Ok(true);
    }
    Ok(phrases.hold(range))
}

/// The spans of the multi-word terms of each of a file's accepted exceptions in
/// `content`, when `found` holds a match to check them against.
pub(crate) fn accepted_spans(
    accepted: &[&Exceptions],
    content: &str,
    found: bool,
) -> Result<Vec<PhraseSpans>, SearchError> {
    accepted
        .iter()
        .map(|exceptions| phrase_spans(exceptions, content, found))
        .collect()
}

/// Whether one of a file's accepted exceptions holds back the match `range`,
/// which reads `observed`.
pub(crate) fn accepted_hold(
    accepted: &[&Exceptions],
    spans: &[PhraseSpans],
    range: &Range<usize>,
    observed: &str,
) -> Result<bool, SearchError> {
    for (exceptions, phrases) in accepted.iter().zip(spans) {
        if excepted(exceptions, phrases, range, observed)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Each match of the tokens that no exception holds back and no code span
/// overlaps.
fn run_existence(
    existence: &Existence,
    text: &Text,
    accepted: &[&Exceptions],
) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let found = existence.pattern.find_all(content)?;
    let phrases = phrase_spans(&existence.exceptions, content, !found.is_empty())?;
    let held = accepted_spans(accepted, content, !found.is_empty())?;
    let mut hits = Vec::new();
    for range in found {
        if text.overlaps_code(&range) {
            continue;
        }
        let Some(matched) = content.get(range.clone()) else {
            continue;
        };
        if excepted(&existence.exceptions, &phrases, &range, matched.trim())?
            || accepted_hold(accepted, &held, &range, matched.trim())?
        {
            continue;
        }
        hits.push(Hit {
            message: None,
            values: vec![matched.to_owned()],
            numbered: false,
            range,
        });
    }
    Ok(hits)
}

/// Each match of a swap key that is not already in the form of its value.
fn run_substitution(
    substitution: &Substitution,
    text: &Text,
    accepted: &[&Exceptions],
) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let found = substitution.pattern.find_groups(content, Swap::GROUP)?;
    let phrases = phrase_spans(&substitution.exceptions, content, !found.is_empty())?;
    let held = accepted_spans(accepted, content, !found.is_empty())?;
    let mut forms = HashMap::new();
    let mut hits = Vec::new();
    for (index, range) in found {
        let (Some(swap), Some(observed)) =
            (substitution.swaps.get(index), content.get(range.clone()))
        else {
            continue;
        };
        if text.overlaps_code(&range) {
            continue;
        }
        let mut expected = swap.value.clone();
        if substitution.capitalize && starts_with_capital(observed) {
            expected = capitalized(&expected);
        }
        if let Some(Some(expander)) = substitution.expanders.get(index) {
            expected = expander.replace_all(observed, &expected)?;
        }
        let (form, expected) = if substitution.terms {
            (term_pattern(&expected), recase_to_term(&expected, observed))
        } else {
            (expected.clone(), expected)
        };
        if in_form(&mut forms, &form, observed)?
            || excepted(&substitution.exceptions, &phrases, &range, observed)?
            || accepted_hold(accepted, &held, &range, observed)?
        {
            continue;
        }
        hits.push(Hit {
            message: None,
            values: vec![expected, observed.to_owned()],
            numbered: false,
            range,
        });
    }
    Ok(hits)
}

/// Whether `observed` is already `expected`: equal to it when it is a phrase
/// of letters, digits, spaces and hyphens, and matched by it as a pattern
/// otherwise.
///
/// `forms` keeps each pattern the first time it is needed.
fn in_form(
    forms: &mut HashMap<String, Option<Pattern>>,
    expected: &str,
    observed: &str,
) -> Result<bool, SearchError> {
    if expected
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-')
    {
        return Ok(expected == observed);
    }
    let form = forms
        .entry(expected.to_owned())
        .or_insert_with(|| Pattern::new(&format!("^(?:{expected})$")).ok());
    match form {
        Some(form) => form.is_match(observed),
        None => Ok(expected == observed),
    }
}

/// Whether `text` starts with a capital letter.
fn starts_with_capital(text: &str) -> bool {
    text.chars().next().is_some_and(char::is_uppercase)
}

/// `text` with its first character in capitals.
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// One hit when the matches of the token are over `max` or under `min`.
///
/// A match that overlaps a code span is no match: it is neither counted nor
/// pointed at.
/// The alert points at the first counted match that is not blank, and at the
/// first word when there is none.
fn run_occurrence(occurrence: &Occurrence, text: &Text) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let found: Vec<Range<usize>> = occurrence
        .pattern
        .find_all(content)?
        .into_iter()
        .filter(|range| !text.overlaps_code(range))
        .collect();
    let count = found.len();
    let over = occurrence.max > 0 && count > occurrence.max;
    let under = occurrence.min > 0 && count < occurrence.min;
    if !over && !under {
        return Ok(Vec::new());
    }
    let placed = found.into_iter().find(|range| {
        content
            .get(range.clone())
            .is_some_and(|matched| !matched.trim().is_empty())
    });
    let range = placed.or_else(|| first_word(content));
    Ok(range
        .map(|range| Hit {
            message: None,
            values: vec![count.to_string()],
            numbered: true,
            range,
        })
        .into_iter()
        .collect())
}

/// The bytes of the first run of characters that are not whitespace.
fn first_word(content: &str) -> Option<Range<usize>> {
    let start = content.find(|c: char| !c.is_whitespace())?;
    let length = content
        .get(start..)?
        .find(char::is_whitespace)
        .unwrap_or(content.len() - start);
    Some(start..start + length)
}

/// How many matches of the tokens `existence` holds in `content`, not
/// counting the ones an exception holds back. A match in a code span counts.
fn count_tokens(existence: &Existence, content: &str) -> Result<usize, SearchError> {
    let found = existence.pattern.find_all(content)?;
    let phrases = phrase_spans(&existence.exceptions, content, !found.is_empty())?;
    let mut count = 0;
    for range in found {
        let Some(matched) = content.get(range.clone()) else {
            continue;
        };
        if !excepted(&existence.exceptions, &phrases, &range, matched.trim())? {
            count += 1;
        }
    }
    Ok(count)
}

/// One hit at the start of `text` when the metric holds for it, measured
/// apart from the rest of the file.
fn run_metric(metric: &Metric, text: &Text) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let occurrences = metric
        .tokens
        .as_ref()
        .map(|tokens| count_tokens(tokens, content))
        .transpose()?;
    let Some(base) = variables(content, Elements::default()) else {
        return Ok(Vec::new());
    };
    Ok(measure(metric, &base, occurrences)
        .map(|result| Hit {
            message: None,
            range: 0..0,
            values: vec![format!("{result:.2}")],
            numbered: false,
        })
        .into_iter()
        .collect())
}

/// The term a text of `conditional` stands for: the text, in lower case when
/// the rule ignores case.
fn term(conditional: &Conditional, text: &str) -> String {
    if conditional.ignorecase {
        text.to_lowercase()
    } else {
        text.to_owned()
    }
}

/// Each match of `first` in `text` that the file has not defined, and no
/// exception holds back.
///
/// With no group in `second`, a match of `second` in `text` defines every
/// `first` in it. With one, the texts its groups capture in `text` are
/// defined from here to the end of the file, and a `first` is defined when it
/// is one of them. A match that overlaps a code span is no match of either.
fn run_conditional(
    conditional: &Conditional,
    text: &Text,
    memory: &mut Memory,
) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let seconds: Vec<Captured> = conditional
        .second
        .find_captured(content)?
        .into_iter()
        .filter(|found| !text.overlaps_code(&found.whole))
        .collect();
    if !conditional.defines && !seconds.is_empty() {
        return Ok(Vec::new());
    }
    for found in &seconds {
        for group in &found.groups {
            if let Some(defined) = content.get(group.clone()) {
                memory.terms.insert(term(conditional, defined));
            }
        }
    }
    let firsts = conditional.first.find_all(content)?;
    let phrases = phrase_spans(&conditional.exceptions, content, !firsts.is_empty())?;
    let mut hits = Vec::new();
    for range in firsts {
        let Some(observed) = content.get(range.clone()) else {
            continue;
        };
        if text.overlaps_code(&range)
            || (conditional.defines && memory.terms.contains(&term(conditional, observed)))
            || excepted(&conditional.exceptions, &phrases, &range, observed)?
        {
            continue;
        }
        hits.push(Hit {
            message: None,
            values: vec![observed.to_owned()],
            numbered: false,
            range,
        });
    }
    Ok(hits)
}

/// One hit for each pair that matched in `text` once both of its forms have
/// matched in the file so far, `text` included.
///
/// The hit is at the last match of the pair in `text`, and holds its text. A
/// match that overlaps a code span is no match.
fn run_consistency(
    consistency: &Consistency,
    text: &Text,
    memory: &mut Memory,
) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let mut hits = Vec::new();
    for (pair, pattern) in consistency.pairs.iter().enumerate() {
        let mut last = None;
        for (form, range) in pattern.find_groups(content, Consistency::FORM)? {
            if text.overlaps_code(&range) {
                continue;
            }
            memory.seen.insert((pair, form));
            last = Some(range);
        }
        if let Some(range) = last
            && memory.seen.contains(&(pair, 0))
            && memory.seen.contains(&(pair, 1))
            && let Some(matched) = content.get(range.clone())
        {
            hits.push(Hit {
                message: None,
                values: vec![matched.to_owned()],
                numbered: false,
                range,
            });
        }
    }
    Ok(hits)
}

/// Whether the matches `before` and `current` are the same word.
fn same(repetition: &Repetition, before: &str, current: &str) -> bool {
    if repetition.ignorecase {
        before.to_lowercase() == current.to_lowercase()
    } else {
        before == current
    }
}

/// Each match of the tokens that repeats the one before it, once the repeats
/// of the text are over `max`.
///
/// A match is compared, trimmed of spaces, with the match before it. The
/// count of repeats goes back to 0 only after a hit, as Vale counts it. A
/// repeat that spans more than one sentence, or that an exception holds back,
/// is no hit. A match that overlaps a code span is no match, and it breaks the
/// chain: the match after it has nothing before it to repeat.
fn run_repetition(
    repetition: &Repetition,
    text: &Text,
    accepted: &[&Exceptions],
) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let found = repetition.pattern.find_all(content)?;
    // The sentence model is embedded and loads; without it no span could be
    // told from two sentences, and a repeat is not reported.
    let Ok(model) = Model::english() else {
        return Ok(Vec::new());
    };
    let mut phrases: Option<PhraseSpans> = None;
    let mut accepted_held: Option<Vec<PhraseSpans>> = None;
    let mut hits = Vec::new();
    let mut before: Option<(usize, &str)> = None;
    let mut count = 0_usize;
    for range in found {
        let Some(matched) = content.get(range.clone()) else {
            continue;
        };
        if text.overlaps_code(&range) {
            before = None;
            continue;
        }
        let current = matched.trim();
        let repeated = match before {
            Some((_, earlier)) => !current.is_empty() && same(repetition, earlier, current),
            None => false,
        };
        let spelled = match &repetition.letters {
            Some(letters) => letters.is_match(current)?,
            None => true,
        };
        if let Some((start, _)) = before
            && repeated
            && spelled
        {
            count += 1;
            let span = start..range.end;
            if count > repetition.max
                && let Some(spanned) = content.get(span.clone())
                && model.split(spanned).len() == 1
            {
                let held = match &phrases {
                    Some(held) => held,
                    None => phrases.insert(phrase_spans(&repetition.exceptions, content, true)?),
                };
                let accepted_phrases = match &accepted_held {
                    Some(held) => held,
                    None => accepted_held.insert(accepted_spans(accepted, content, true)?),
                };
                if !excepted(&repetition.exceptions, held, &span, spanned)?
                    && !accepted_hold(accepted, accepted_phrases, &span, spanned)?
                {
                    hits.push(Hit {
                        message: None,
                        range: span,
                        values: vec![current.to_owned()],
                        numbered: false,
                    });
                    count = 0;
                }
            }
        }
        before = Some((range.start, current));
    }
    Ok(hits)
}

/// One hit that spans `text` when it is not in the case the rule names.
///
/// The message reads the text, and then the text in the case the rule
/// requires.
fn run_capitalization(capitalization: &Capitalization, text: &Text) -> Vec<Hit> {
    let content = text.as_str();
    let (expected, matched) = evaluate(capitalization, content);
    if matched {
        return Vec::new();
    }
    vec![Hit {
        message: None,
        range: 0..content.len(),
        values: vec![content.to_owned(), expected],
        numbered: false,
    }]
}

/// One hit for each run of words in `text`, a sentence, that the rule's tokens
/// match. The message's `%s` read the words of the run.
fn run_sequence(sequence: &Sequence, text: &Text) -> Result<Vec<Hit>, SearchError> {
    Ok(find(sequence, text.as_str())?
        .into_iter()
        .map(|matched| Hit {
            message: None,
            range: matched.range,
            values: matched.words,
            numbered: false,
        })
        .collect())
}

/// One hit at the start of `text` when the average grade of the text is over
/// the rule's grade, measured apart from the rest of the file.
fn run_readability(readability: &Readability, text: &Text) -> Vec<Hit> {
    let Some(base) = variables(text.as_str(), Elements::default()) else {
        return Vec::new();
    };
    graded(readability, &base)
        .map(grade_hit)
        .into_iter()
        .collect()
}

/// The hit of an average grade, at the start of the text.
fn grade_hit(average: f64) -> Hit {
    Hit {
        message: None,
        range: 0..0,
        values: vec![format!("{average:.2}")],
        numbered: false,
    }
}

/// What the check reports for the prose of the whole file, whose variables
/// are `base`: a `metric` or a `readability` rule that measures the file.
pub(crate) fn run_summary(
    check: &Check,
    summary: &Summary,
    base: &Variables,
) -> Result<Option<Hit>, SearchError> {
    match check {
        Check::Metric(metric) => {
            let occurrences = metric
                .tokens
                .as_ref()
                .map(|tokens| count_tokens(tokens, &summary.content))
                .transpose()?;
            Ok(measure(metric, base, occurrences).map(|result| Hit {
                message: None,
                range: 0..0,
                values: vec![format!("{result:.2}")],
                numbered: false,
            }))
        }
        Check::Readability(readability) => Ok(graded(readability, base).map(grade_hit)),
        Check::Existence(_)
        | Check::Substitution(_)
        | Check::Occurrence(_)
        | Check::Consistency(_)
        | Check::Conditional(_)
        | Check::Repetition(_)
        | Check::Capitalization(_)
        | Check::Sequence(_)
        | Check::Spelling(_) => Ok(None),
    }
}
