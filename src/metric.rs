//! Metrics: the variables a formula reads, and the test of a measured text.

use std::collections::HashMap;
use std::ops::Range;

use crate::formula::{Formula, Value};
use crate::readability::{Counts, quoted_words};
use crate::rule::{Metric, Readability};
use crate::splitter::Model;
use crate::summary::Elements;

/// The name the value of a formula takes in the text of a condition, which
/// reads `> 4` as `<result> > 4`.
pub(crate) const LEFT: &str = "__result__";

/// The names of the counts a formula reads.
const COUNTS: [&str; 9] = [
    "words",
    "sentences",
    "characters",
    "syllables",
    "complex_words",
    "long_words",
    "polysyllabic_words",
    "quote_words",
    "sentence_length_sd",
];

/// The names of the scores a formula reads.
const SCORES: [&str; 8] = [
    "automated_readability",
    "coleman_liau",
    "dale_chall",
    "flesch_kincaid",
    "flesch_reading_ease",
    "gunning_fog",
    "lix",
    "smog",
];

/// The names of the element counts a formula reads.
const ELEMENTS: [&str; 10] = [
    "paragraphs",
    "list",
    "blockquote",
    "pre",
    "heading_h1",
    "heading_h2",
    "heading_h3",
    "heading_h4",
    "heading_h5",
    "heading_h6",
];

/// Whether `name` is a variable of a formula. `counted` says the rule has
/// `tokens` or `raw`, which give `occurrences`.
pub(crate) fn is_variable(name: &str, counted: bool) -> bool {
    COUNTS.contains(&name)
        || SCORES.contains(&name)
        || ELEMENTS.contains(&name)
        || (counted && name == "occurrences")
}

/// A float from a count, which stays far below 2^32.
fn float(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

/// The variables of a text, by name.
pub(crate) type Variables = HashMap<&'static str, f64>;

/// The variables of the text `content` that do not depend on the rule: the
/// counts, the scores and the element counts.
///
/// Returns `None` when the sentence model does not load.
pub(crate) fn variables(
    content: &str,
    code: &[Range<usize>],
    elements: Elements,
) -> Option<Variables> {
    let model = Model::english().ok()?;
    let counts = Counts::with_code(content, code, model);
    let mut found = HashMap::new();
    for (name, value) in [
        ("words", counts.words),
        ("sentences", counts.sentences),
        ("characters", counts.characters),
        ("syllables", counts.syllables),
        ("complex_words", counts.complex_words),
        ("long_words", counts.long_words),
        ("polysyllabic_words", counts.polysyllabic_words),
        ("quote_words", quoted_words(content)),
    ] {
        found.insert(name, float(value));
    }
    found.insert("sentence_length_sd", counts.sentence_length_sd());
    found.extend(counts.scores());
    for name in ELEMENTS {
        found.insert(name, elements.count(name).map_or(0.0, float));
    }
    Some(found)
}

/// The result of the formula over a text with the variables `base`, when the
/// condition holds for it.
///
/// A text with no words, a formula that gives no number, and a condition that
/// does not give a truth value hold nothing.
pub(crate) fn measure(
    metric: &Metric,
    base: &Variables,
    occurrences: Option<usize>,
) -> Option<f64> {
    let mut found = base.clone();
    if let Some(count) = occurrences {
        found.insert("occurrences", float(count));
    }
    if found.get("words").is_none_or(|words| *words == 0.0) {
        return None;
    }
    let result = metric
        .formula
        .eval(&|name| found.get(name).copied())
        .ok()?
        .number()
        .filter(|result| result.is_finite())?;
    found.insert("result", result);
    found.insert(LEFT, format!("{result:.6}").parse().ok()?);
    let holds = metric
        .condition
        .eval(&|name| found.get(name).copied())
        .ok()?;
    (holds == Value::Bool(true)).then_some(result)
}

/// The average of the scores of `readability` over a text with the variables
/// `base`, when it is over the rule's grade.
///
/// A text with no words or no sentence holds nothing: each score divides by
/// one of those counts.
pub(crate) fn graded(readability: &Readability, base: &Variables) -> Option<f64> {
    let has = |name: &str| base.get(name).is_some_and(|count| *count > 0.0);
    if !has("words") || !has("sentences") {
        return None;
    }
    let sum = readability
        .scores
        .iter()
        .map(|score| base.get(score.variable()).copied())
        .sum::<Option<f64>>()?;
    let average = sum / float(readability.scores.len());
    (average.is_finite() && average > readability.grade).then_some(average)
}

/// Whether `formula` is one that reads `result`; the name is a variable of a
/// condition only.
pub(crate) fn is_condition_variable(name: &str, counted: bool) -> bool {
    name == "result" || name == LEFT || is_variable(name, counted)
}

/// A formula, for the code that refuses a rule to describe.
pub(crate) fn describe(formula: &Formula) -> &'static str {
    if formula.is_truth() {
        "a truth value"
    } else {
        "a number"
    }
}
