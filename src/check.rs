//! Checks: what each check type finds in one block of text.

use std::collections::HashMap;
use std::ops::Range;

use crate::alert::fill;
use crate::metric::measure;
use crate::pattern::{Pattern, SearchError};
use crate::rule::{Check, Exceptions, Existence, Metric, Occurrence, Rule, Substitution, Swap};
use crate::summary::{Elements, Summary};
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
        (read(&rule.message), read(&rule.description))
    }
}

/// The matches `check` finds in `text`.
///
/// # Errors
///
/// Returns a [`SearchError`] when a pattern gives up on the text.
pub(crate) fn run(check: &Check, text: &Text) -> Result<Vec<Hit>, SearchError> {
    match check {
        Check::Existence(existence) => run_existence(existence, text),
        Check::Substitution(substitution) => run_substitution(substitution, text),
        Check::Occurrence(occurrence) => run_occurrence(occurrence, text),
        Check::Metric(metric) => run_metric(metric, text),
    }
}

/// The spans of the multi-word exceptions in `content`, when `found` holds a
/// match to check them against.
fn phrase_spans(
    exceptions: &Exceptions,
    content: &str,
    found: bool,
) -> Result<Vec<Range<usize>>, SearchError> {
    match &exceptions.phrases {
        Some(phrases) if found => phrases.find_overlapping(content),
        _ => Ok(Vec::new()),
    }
}

/// Whether an exception holds back the match `range`, which reads `observed`.
fn excepted(
    exceptions: &Exceptions,
    phrases: &[Range<usize>],
    range: &Range<usize>,
    observed: &str,
) -> Result<bool, SearchError> {
    if let Some(words) = &exceptions.words
        && words.is_match(observed)?
    {
        return Ok(true);
    }
    Ok(phrases
        .iter()
        .any(|phrase| phrase.start <= range.start && range.end <= phrase.end))
}

/// Each match of the tokens that no exception holds back and no code span
/// overlaps.
fn run_existence(existence: &Existence, text: &Text) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let found = existence.pattern.find_all(content)?;
    let phrases = phrase_spans(&existence.exceptions, content, !found.is_empty())?;
    let mut hits = Vec::new();
    for range in found {
        if text.overlaps_code(&range) {
            continue;
        }
        let Some(matched) = content.get(range.clone()) else {
            continue;
        };
        if excepted(&existence.exceptions, &phrases, &range, matched.trim())? {
            continue;
        }
        hits.push(Hit {
            values: vec![matched.to_owned()],
            numbered: false,
            range,
        });
    }
    Ok(hits)
}

/// Each match of a swap key that is not already in the form of its value.
fn run_substitution(substitution: &Substitution, text: &Text) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let found = substitution.pattern.find_groups(content, Swap::GROUP)?;
    let phrases = phrase_spans(&substitution.exceptions, content, !found.is_empty())?;
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
        if in_form(&mut forms, &expected, observed)?
            || excepted(&substitution.exceptions, &phrases, &range, observed)?
        {
            continue;
        }
        hits.push(Hit {
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
fn run_occurrence(occurrence: &Occurrence, text: &Text) -> Result<Vec<Hit>, SearchError> {
    let content = text.as_str();
    let found = occurrence.pattern.find_all(content)?;
    let count = found.len();
    let over = occurrence.max > 0 && count > occurrence.max;
    let under = occurrence.min > 0 && count < occurrence.min;
    if !over && !under {
        return Ok(Vec::new());
    }
    let range = if count == 0 {
        first_word(content)
    } else {
        found.into_iter().find(|range| {
            !text.overlaps_code(range)
                && content
                    .get(range.clone())
                    .is_some_and(|matched| !matched.trim().is_empty())
        })
    };
    Ok(range
        .map(|range| Hit {
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
    Ok(measure(metric, content, Elements::default(), occurrences)
        .map(|result| Hit {
            range: 0..0,
            values: vec![format!("{result:.2}")],
            numbered: false,
        })
        .into_iter()
        .collect())
}

/// What the metric reports for the prose of the whole file.
pub(crate) fn run_summary(metric: &Metric, summary: &Summary) -> Result<Option<Hit>, SearchError> {
    let occurrences = metric
        .tokens
        .as_ref()
        .map(|tokens| count_tokens(tokens, &summary.content))
        .transpose()?;
    Ok(
        measure(metric, &summary.content, summary.elements, occurrences).map(|result| Hit {
            range: 0..0,
            values: vec![format!("{result:.2}")],
            numbered: false,
        }),
    )
}
