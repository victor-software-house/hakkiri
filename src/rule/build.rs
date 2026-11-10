//! Rule files into rules: the patterns each check assembles from its fields,
//! compiled once.
//!
//! The assembly follows Vale 3.26.0: a word template or a plain one, the
//! `raw` text before or after it, and one flag for case.

use std::cmp::Reverse;

use super::schema::{
    ConditionalFields, ConsistencyFields, ExistenceFields, MetricFields, OccurrenceFields,
    ReadabilityFields, RepetitionFields, SubstitutionFields,
};
use super::{
    Action, Check, Conditional, Consistency, Exceptions, Existence, LoadError, LoadProblem, Metric,
    Occurrence, Readability, Repetition, Rule, Score, Substitution, Swap, level_of,
};
use crate::formula::Formula;
use crate::metric;
use crate::pattern::Pattern;
use crate::scope::Selector;

/// Where a rule came from, for the errors that name it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Source<'a> {
    /// The rule's name, such as `Team.Filler`.
    pub(super) name: &'a str,
    /// The rule's file.
    pub(super) file: &'a str,
}

impl Source<'_> {
    fn problem(&self, field: &str, message: String) -> LoadError {
        LoadError {
            problems: vec![LoadProblem {
                file: self.file.to_owned(),
                line: None,
                column: None,
                field: field.to_owned(),
                message,
            }],
        }
    }

    /// Parses the formula `text` of `field`, whose variables are those that
    /// `known` accepts.
    fn formula(
        &self,
        field: &str,
        text: &str,
        known: impl Fn(&str) -> bool,
    ) -> Result<Formula, LoadError> {
        Formula::parse(text, &known)
            .map_err(|error| self.problem(field, format!("rule `{}`: {error}", self.name)))
    }

    /// Compiles `pattern`, assembled from the `parts` of `field`.
    ///
    /// When it does not compile, the first part that does not compile alone
    /// is the one the error names. When every part does, the fault is in how
    /// they combine, and the error names the whole pattern.
    fn compile(&self, field: &str, pattern: &str, parts: &[&str]) -> Result<Pattern, LoadError> {
        Pattern::new(pattern).map_err(|combined| {
            let blamed = parts
                .iter()
                .filter(|part| !part.trim().is_empty())
                .find_map(|part| Pattern::new(part).err())
                .unwrap_or(combined);
            self.problem(field, format!("rule `{}`: {blamed}", self.name))
        })
    }
}

/// The text a flag-and-word template wraps its alternatives in.
fn template(word: bool, ignorecase: bool, body: &str) -> String {
    let wrapped = if word {
        format!(r"(?m)\b(?:{body})\b")
    } else {
        format!("(?m)(?:{body})")
    };
    if ignorecase {
        format!("(?i){wrapped}")
    } else {
        wrapped
    }
}

/// `pattern` with each run of spaces between words matching a line wrap too.
///
/// A space inside a character class, after a backslash or before a
/// quantifier keeps its meaning, as does every space in a free-spacing
/// pattern.
fn wrap_spaces(pattern: &str) -> String {
    if !pattern.contains([' ', '\t']) || pattern.contains("(?x") {
        return pattern.to_owned();
    }
    let mut out = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();
    let mut in_class = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                out.push(c);
                out.extend(chars.next());
            }
            _ if in_class => {
                in_class = c != ']';
                out.push(c);
            }
            '[' => {
                in_class = true;
                out.push(c);
            }
            ' ' | '\t' => {
                let mut run = String::from(c);
                while let Some(next) = chars.next_if(|next| matches!(next, ' ' | '\t')) {
                    run.push(next);
                }
                if chars
                    .peek()
                    .is_some_and(|next| matches!(next, '*' | '+' | '?' | '{'))
                {
                    out.push_str(&run);
                } else {
                    out.push_str(r"\s+");
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Whether `term` holds no regular expression syntax, a period apart.
fn literal_term(term: &str) -> bool {
    !term.chars().any(|c| {
        matches!(
            c,
            '\\' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$'
        )
    })
}

/// An exception as a pattern: a plain term has its periods escaped, and the
/// space between its words matches a line wrap too.
fn term_pattern(term: &str) -> String {
    let term = if literal_term(term) {
        term.replace('.', r"\.")
    } else {
        term.to_owned()
    };
    wrap_runs(&term)
}

/// `text` with each run of spaces and tabs as `\s+`.
fn wrap_runs(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_run = false;
    for c in text.chars() {
        if matches!(c, ' ' | '\t') {
            if !in_run {
                out.push_str(r"\s+");
            }
            in_run = true;
        } else {
            in_run = false;
            out.push(c);
        }
    }
    out
}

/// The patterns of a rule's `exceptions`.
///
/// The words come longest first, so an alternation prefers the longer term,
/// and each holds its own case.
/// The phrases are the terms of more than one word, matched without regard to
/// case, so a match inside one is held back too.
fn exceptions(source: &Source<'_>, terms: &[String]) -> Result<Exceptions, LoadError> {
    if terms.is_empty() {
        return Ok(Exceptions::default());
    }
    let mut longest_first: Vec<&String> = terms.iter().collect();
    longest_first.sort_by_key(|term| Reverse(term.len()));
    let alternatives: Vec<String> = longest_first
        .into_iter()
        .map(|term| {
            let pattern = term_pattern(term);
            if pattern.starts_with("(?i)") {
                pattern
            } else {
                format!("(?-i){pattern}")
            }
        })
        .collect();
    let parts: Vec<&str> = terms.iter().map(String::as_str).collect();
    let words = source.compile(
        "exceptions",
        &template(true, false, &alternatives.join("|")),
        &parts,
    )?;
    let phrases = terms
        .iter()
        .filter(|term| term.contains([' ', '\t']) || term.contains(r"\s"))
        .map(|term| {
            let pattern = format!(r"(?i)\b(?:{})\b", term_pattern(term));
            source.compile("exceptions", &pattern, &[term.as_str()])
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Exceptions {
        words: Some(words),
        phrases,
    })
}

/// The pattern an `existence` rule, or the tokens of a `metric` rule, match
/// with.
fn tokens_pattern(
    source: &Source<'_>,
    raw: &[String],
    tokens: &[String],
    append: bool,
    nonword: bool,
    ignorecase: bool,
) -> Result<Pattern, LoadError> {
    let parsed: Vec<String> = tokens
        .iter()
        .filter(|token| !token.trim().is_empty())
        .map(|token| wrap_spaces(token))
        .collect();
    let raw_text = raw.concat();
    let word = !nonword && !tokens.is_empty();
    let body = template(word, false, &parsed.join("|"));
    let joined = if append {
        format!("{body}{raw_text}")
    } else {
        format!("{raw_text}{body}")
    };
    let pattern = if ignorecase {
        format!("(?i){joined}")
    } else {
        joined
    };
    let parts: Vec<&str> = parsed
        .iter()
        .map(String::as_str)
        .chain(raw.iter().map(String::as_str))
        .collect();
    source.compile("tokens", &pattern, &parts)
}

fn existence(source: &Source<'_>, fields: &ExistenceFields) -> Result<Existence, LoadError> {
    let nonword = fields.nonword.value();
    Ok(Existence {
        pattern: tokens_pattern(
            source,
            fields.raw.values(),
            fields.tokens.values(),
            fields.append.value(),
            nonword,
            fields.ignorecase.value(),
        )?,
        exceptions: exceptions(source, fields.exceptions.values())?,
    })
}

/// Whether `value` holds a `$` and a digit, a reference to a group of the key.
fn has_group_reference(value: &str) -> bool {
    value.match_indices('$').any(|(index, _)| {
        value
            .get(index + 1..)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|c| c.is_ascii_digit())
    })
}

fn substitution(
    source: &Source<'_>,
    fields: &SubstitutionFields,
) -> Result<Substitution, LoadError> {
    let mut swaps: Vec<Swap> = fields
        .swap
        .entries()
        .iter()
        .map(|(key, value)| Swap {
            key: key.clone(),
            value: value.clone(),
        })
        .collect();
    swaps.sort_by_key(|swap| Reverse(swap.key.len()));
    let alternatives: Vec<String> = swaps
        .iter()
        .enumerate()
        .map(|(index, swap)| format!("(?P<{}{index}>{})", Swap::GROUP, wrap_spaces(&swap.key)))
        .collect();
    let pattern = template(
        !fields.nonword.value(),
        fields.ignorecase.value(),
        &alternatives.join("|"),
    );
    let keys: Vec<&str> = swaps.iter().map(|swap| swap.key.as_str()).collect();
    let expanders = swaps
        .iter()
        .map(|swap| {
            if !has_group_reference(&swap.value) {
                return Ok(None);
            }
            // The key as the main pattern reads it, spaces matching a line
            // wrap included, and held to the whole match.
            let flags = if fields.ignorecase.value() {
                "(?m)(?i)"
            } else {
                "(?m)"
            };
            let key = format!(r"{flags}\A(?:{})\z", wrap_spaces(&swap.key));
            source.compile("swap", &key, &[&swap.key]).map(Some)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Substitution {
        pattern: source.compile("swap", &pattern, &keys)?,
        swaps,
        expanders,
        exceptions: exceptions(source, fields.exceptions.values())?,
        capitalize: fields.capitalize.value(),
        ignorecase: fields.ignorecase.value(),
    })
}

fn occurrence(source: &Source<'_>, fields: &OccurrenceFields) -> Result<Occurrence, LoadError> {
    let token = fields.token.value();
    let pattern = if fields.ignorecase.value() {
        format!("(?i)(?:{token})")
    } else {
        format!("(?:{token})")
    };
    Ok(Occurrence {
        pattern: source.compile("token", &pattern, &[token])?,
        max: fields.max.value(),
        min: fields.min.value(),
    })
}

fn metric(source: &Source<'_>, fields: &MetricFields) -> Result<Metric, LoadError> {
    let counted = !fields.tokens.values().is_empty() || !fields.raw.values().is_empty();
    let tokens = if counted {
        Some(Existence {
            pattern: tokens_pattern(
                source,
                fields.raw.values(),
                fields.tokens.values(),
                fields.append.value(),
                fields.nonword.value(),
                fields.ignorecase.value(),
            )?,
            exceptions: exceptions(source, fields.exceptions.values())?,
        })
    } else {
        None
    };
    let formula = source.formula("formula", &read_headings(fields.formula.value()), |name| {
        metric::is_variable(name, counted)
    })?;
    if formula.is_truth() {
        return Err(source.problem(
            "formula",
            format!(
                "rule `{}`: the formula gives {}, and a metric needs a number",
                source.name,
                metric::describe(&formula)
            ),
        ));
    }
    let condition = source.formula(
        "condition",
        &format!(
            "{} {}",
            metric::LEFT,
            read_headings(fields.condition.value())
        ),
        |name| metric::is_condition_variable(name, counted),
    )?;
    if condition.is_number() {
        return Err(source.problem(
            "condition",
            format!(
                "rule `{}`: the condition must compare the result, such as `> 4`",
                source.name
            ),
        ));
    }
    Ok(Metric {
        formula,
        condition,
        tokens,
    })
}

/// Whether `pattern` holds a capture group: a `(` that is not escaped, not in
/// a character class and not the start of an extension such as `(?:`, or a
/// named group.
fn has_capture_group(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    let mut in_class = false;
    let mut at = 0;
    while let Some(&byte) = bytes.get(at) {
        match byte {
            b'\\' => at += 1,
            b'[' => in_class = true,
            b']' => in_class = false,
            b'(' if !in_class => {
                if bytes.get(at + 1) != Some(&b'?') {
                    return true;
                }
                let rest = pattern.get(at + 2..).unwrap_or_default();
                if rest.starts_with("P<")
                    || (rest.starts_with('<') && !rest.starts_with("<=") && !rest.starts_with("<!"))
                {
                    return true;
                }
            }
            _ => {}
        }
        at += 1;
    }
    false
}

fn conditional(source: &Source<'_>, fields: &ConditionalFields) -> Result<Conditional, LoadError> {
    if !fields.within.value().is_empty() {
        return Err(source.problem(
            "in",
            format!(
                "rule `{}`: `in` names a View scope, and hakkiri has no Views",
                source.name
            ),
        ));
    }
    let ignorecase = fields.ignorecase.value();
    let compile = |field: &str, pattern: &str| {
        let written = if ignorecase {
            format!("(?i){pattern}")
        } else {
            pattern.to_owned()
        };
        source.compile(field, &written, &[pattern])
    };
    let second = fields.second.value();
    Ok(Conditional {
        first: compile("first", fields.first.value())?,
        second: compile("second", second)?,
        defines: has_capture_group(second),
        ignorecase,
        exceptions: exceptions(source, fields.exceptions.values())?,
    })
}

/// One pattern for each pair of a `consistency` rule, by key.
///
/// A pair matches as `existence` tokens do, its two forms as groups.
fn consistency(source: &Source<'_>, fields: &ConsistencyFields) -> Result<Consistency, LoadError> {
    let mut entries: Vec<&(String, String)> = fields.either.entries().iter().collect();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let pairs = entries
        .into_iter()
        .map(|(first, second)| {
            let body = format!(
                "(?P<{form}0>{first})|(?P<{form}1>{second})",
                form = Consistency::FORM
            );
            let pattern = template(!fields.nonword.value(), fields.ignorecase.value(), &body);
            source.compile("either", &pattern, &[first, second])
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Consistency { pairs })
}

/// The pattern of a `repetition` rule: its tokens in one group, with no word
/// template.
fn repetition(source: &Source<'_>, fields: &RepetitionFields) -> Result<Repetition, LoadError> {
    let tokens = fields.tokens.values();
    let group = format!("({})", tokens.join("|"));
    let pattern = if fields.ignorecase.value() {
        format!("(?i){group}")
    } else {
        group
    };
    let parts: Vec<&str> = tokens.iter().map(String::as_str).collect();
    Ok(Repetition {
        pattern: source.compile("tokens", &pattern, &parts)?,
        max: fields.max.value(),
        ignorecase: fields.ignorecase.value(),
        letters: if fields.alpha.value() {
            Some(source.compile("alpha", r"\A\p{L}+\z", &[])?)
        } else {
            None
        },
        exceptions: exceptions(source, fields.exceptions.values())?,
    })
}

fn readability(source: &Source<'_>, fields: &ReadabilityFields) -> Result<Readability, LoadError> {
    let names = fields.metrics.values();
    if names.is_empty() {
        return Err(source.problem(
            "metrics",
            format!(
                "rule `{}`: `metrics` lists no score: use one or more of {}",
                source.name,
                score_names()
            ),
        ));
    }
    let mut scores = Vec::new();
    for name in names {
        let score = Score::named(name).ok_or_else(|| {
            source.problem(
                "metrics",
                format!(
                    "rule `{}`: `{name}` is not a score hakkiri knows: use one of {}",
                    source.name,
                    score_names()
                ),
            )
        })?;
        if !scores.contains(&score) {
            scores.push(score);
        }
    }
    Ok(Readability {
        scores,
        grade: fields.grade.value(),
    })
}

/// The names a `metrics` list accepts, as a list for an error.
fn score_names() -> String {
    Score::ALL.map(Score::name).join(", ")
}

/// `formula` with `heading.h2` written as the variable `heading_h2`.
fn read_headings(formula: &str) -> String {
    (1..=6).fold(formula.to_owned(), |text, depth| {
        text.replace(&format!("heading.h{depth}"), &format!("heading_h{depth}"))
    })
}

/// The fields every check type has, and the check built from the rest.
macro_rules! rule_of {
    ($source:expr, $fields:expr, $check:expr) => {{
        let fields = $fields;
        let scope = Selector::parse(fields.scope.values()).map_err(|refused| {
            $source.problem("scope", format!("rule `{}`: {refused}", $source.name))
        })?;
        Rule {
            name: $source.name.to_owned(),
            level: level_of(fields.level.value()),
            message: fields.message.value().to_owned(),
            description: fields.description.value().to_owned(),
            link: fields.link.value().to_owned(),
            limit: fields.limit.value(),
            scope,
            action: fields.action.as_ref().map(|action| Action {
                name: action.name.value().to_owned(),
                params: action.params.values().to_vec(),
            }),
            check: $check,
        }
    }};
}

pub(super) fn existence_rule(
    source: &Source<'_>,
    fields: &ExistenceFields,
) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source,
        fields,
        Check::Existence(existence(source, fields)?)
    ))
}

pub(super) fn substitution_rule(
    source: &Source<'_>,
    fields: &SubstitutionFields,
) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source,
        fields,
        Check::Substitution(substitution(source, fields)?)
    ))
}

pub(super) fn occurrence_rule(
    source: &Source<'_>,
    fields: &OccurrenceFields,
) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source,
        fields,
        Check::Occurrence(occurrence(source, fields)?)
    ))
}

pub(super) fn metric_rule(source: &Source<'_>, fields: &MetricFields) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source,
        fields,
        Check::Metric(metric(source, fields)?)
    ))
}

pub(super) fn readability_rule(
    source: &Source<'_>,
    fields: &ReadabilityFields,
) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source,
        fields,
        Check::Readability(readability(source, fields)?)
    ))
}

pub(super) fn repetition_rule(
    source: &Source<'_>,
    fields: &RepetitionFields,
) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source,
        fields,
        Check::Repetition(repetition(source, fields)?)
    ))
}

pub(super) fn consistency_rule(
    source: &Source<'_>,
    fields: &ConsistencyFields,
) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source,
        fields,
        Check::Consistency(consistency(source, fields)?)
    ))
}

pub(super) fn conditional_rule(
    source: &Source<'_>,
    fields: &ConditionalFields,
) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source,
        fields,
        Check::Conditional(conditional(source, fields)?)
    ))
}
