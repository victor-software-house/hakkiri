//! Rule files into rules: the patterns each check assembles from its fields,
//! compiled once.
//!
//! The assembly follows Vale 3.26.0: a word template or a plain one, the
//! `raw` text before or after it, and one flag for case.

use std::cmp::Reverse;

use super::schema::{ExistenceFields, MetricFields, OccurrenceFields, SubstitutionFields};
use super::{
    Action, Check, Exceptions, Existence, LoadError, LoadProblem, Metric, Occurrence, Rule,
    Substitution, Swap, level_of,
};
use crate::pattern::Pattern;

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
    let phrases: Vec<String> = terms
        .iter()
        .filter(|term| term.contains([' ', '\t']) || term.contains(r"\s"))
        .map(|term| term_pattern(term))
        .collect();
    let phrases = if phrases.is_empty() {
        None
    } else {
        Pattern::new(&format!(r"(?i)\b(?:{})\b", phrases.join("|"))).ok()
    };
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
    Ok(Substitution {
        pattern: source.compile("swap", &pattern, &keys)?,
        swaps,
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
    let counts = !fields.tokens.values().is_empty() || !fields.raw.values().is_empty();
    let tokens = if counts {
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
    Ok(Metric {
        formula: fields.formula.value().to_owned(),
        condition: fields.condition.value().to_owned(),
        tokens,
    })
}

/// The fields every check type has, and the check built from the rest.
macro_rules! rule_of {
    ($name:expr, $fields:expr, $check:expr) => {{
        let fields = $fields;
        Rule {
            name: $name.to_owned(),
            level: level_of(fields.level.value()),
            message: fields.message.value().to_owned(),
            description: fields.description.value().to_owned(),
            link: fields.link.value().to_owned(),
            limit: fields.limit.value(),
            scope: fields.scope.values().to_vec(),
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
        source.name,
        fields,
        Check::Existence(existence(source, fields)?)
    ))
}

pub(super) fn substitution_rule(
    source: &Source<'_>,
    fields: &SubstitutionFields,
) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source.name,
        fields,
        Check::Substitution(substitution(source, fields)?)
    ))
}

pub(super) fn occurrence_rule(
    source: &Source<'_>,
    fields: &OccurrenceFields,
) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source.name,
        fields,
        Check::Occurrence(occurrence(source, fields)?)
    ))
}

pub(super) fn metric_rule(source: &Source<'_>, fields: &MetricFields) -> Result<Rule, LoadError> {
    Ok(rule_of!(
        source.name,
        fields,
        Check::Metric(metric(source, fields)?)
    ))
}
