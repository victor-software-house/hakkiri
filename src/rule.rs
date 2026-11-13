//! Rules: loading a style folder into the rules the engine runs.
//!
//! A style folder holds one `.yml` file per rule.
//! [`load_style`] reads them all, and [`Rule::from_text`] reads one.
//! Loading refuses a rule it cannot run: a field its check does not define, a
//! value of the wrong kind, or a pattern that does not compile.
//! A rule that is valid Vale and that hakkiri does not run, a `script` rule or
//! a `doc()` scope, is not a fault of the file: [`read_style`] reports it
//! apart. A rule may extend another rule, as in Vale.
//! Every error names the file, and the field and line when it has them.

mod affix;
mod build;
mod cases;
mod field;
mod inherit;
mod package;
mod schema;

use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::{error, fmt, fs, io};

use ctl_core::input::{Input, InputError};
use serde::de::DeserializeOwned;

use self::build::Source;
pub(crate) use self::build::term_pattern;
pub use self::cases::{TestCase, alert_lines, cases_from_text};
use self::schema::{
    CapitalizationFields, ConditionalFields, ConsistencyFields, ExistenceFields, MetricFields,
    OccurrenceFields, Probe, ReadabilityFields, RepetitionFields, ScopeProbe, SequenceFields,
    SpellingFields, SubstitutionFields,
};
use crate::dictionary::Speller;
use crate::formula::Formula;
use crate::pattern::{Pattern, PatternError};
use crate::scope::Selector;
use crate::strcase::{Sentence, Title};

/// How serious an alert is.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Level {
    /// A hint that a person may ignore.
    Suggestion,
    /// A fault that a person should fix. A rule with no `level` has this one.
    #[default]
    Warning,
    /// A fault that fails a check.
    Error,
}

impl Level {
    /// The name a rule's `level` uses.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Suggestion => "suggestion",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The level a validated `level` field names; a rule with none is a warning.
fn level_of(name: &str) -> Level {
    match name {
        "suggestion" => Level::Suggestion,
        "error" => Level::Error,
        _ => Level::Warning,
    }
}

/// What a fix would do. It loads and rides on the rule; nothing applies it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Action {
    /// The kind of fix, such as `replace` or `remove`.
    pub name: String,
    /// The arguments of the fix.
    pub params: Vec<String>,
}

/// The value of a rule parameter: one scalar, as a config writes it.
#[derive(Clone, Debug, PartialEq)]
pub enum Param {
    /// `true` or `false`.
    Bool(bool),
    /// A whole number.
    Int(i64),
    /// A number with decimals.
    Float(f64),
    /// Text, which a field that wants a number or a flag reads as one.
    Text(String),
}

/// The fields of a rule that a parameter may not set: lists, mappings, the
/// rule's identity and its prose.
/// They are the fields that Vale 3.26.0 lists, and the `forms` of a
/// `sequence` rule, a list that Vale does not have.
/// Changing one is authoring, which a rule that extends another does.
const NOT_PARAMETERS: [&str; 15] = [
    "extends",
    "name",
    "path",
    "tests",
    "message",
    "description",
    "link",
    "tokens",
    "forms",
    "swap",
    "exceptions",
    "filters",
    "ignore",
    "raw",
    "either",
];

/// What a rule was read from, so it can be read again with parameters.
#[derive(Debug)]
enum Recipe {
    /// A rule file: its name in errors, its text, and the package that holds
    /// it.
    File {
        file: String,
        text: String,
        package: Option<PathBuf>,
    },
    /// A rule built from a vocabulary, which has no fields to set.
    Fixed,
}

/// One rule: a check, what it reports, and the text it reads.
#[derive(Debug)]
pub struct Rule {
    /// The style folder, a dot, and the file name without its extension, such
    /// as `Team.Filler`.
    pub name: String,
    /// The level of the rule's alerts.
    pub level: Level,
    /// The message of the rule's alerts, with `%s` for what the check gives.
    pub message: String,
    /// A longer message, empty when the rule has none.
    pub description: String,
    /// A link to more about the rule, empty when the rule has none.
    pub link: String,
    /// The most alerts the rule reports in one file, or 0 for no limit.
    pub limit: usize,
    /// The text the rule reads, from its `scope`.
    pub scope: Selector,
    /// The rule's `action`.
    pub action: Option<Action>,
    /// The test cases of the rule file, which `check` loads and never runs.
    pub tests: Vec<TestCase>,
    /// What the rule checks.
    pub check: Check,
    recipe: Recipe,
}

/// The check a rule runs, and the patterns it compiled.
#[derive(Debug)]
pub enum Check {
    /// Reports each match of its tokens.
    Existence(Existence),
    /// Names the word to use instead of each match.
    Substitution(Substitution),
    /// Counts matches against a limit.
    Occurrence(Occurrence),
    /// Computes a formula over counts of the text, and tests the result.
    Metric(Metric),
    /// Reports a file that uses both forms of a pair.
    Consistency(Consistency),
    /// Reports a match of one pattern that the file has not defined with
    /// another.
    Conditional(Conditional),
    /// Reports a match that repeats the one before it.
    Repetition(Repetition),
    /// Averages grade-level scores of the text, and tests the average.
    Readability(Readability),
    /// Reports a text that is not in the case it names.
    Capitalization(Capitalization),
    /// Reports a run of words that its tokens match in order.
    Sequence(Sequence),
    /// Reports each word that no dictionary holds.
    Spelling(Spelling),
}

/// The patterns that hold a match back.
#[derive(Debug, Default)]
pub struct Exceptions {
    /// The `exceptions` words: a match that holds one as a whole word is held
    /// back.
    pub words: Option<Pattern>,
    /// The `exceptions` of more than one word, each as a pattern of its own: a
    /// match inside a match of one is held back.
    pub phrases: Vec<Pattern>,
}

impl Exceptions {
    /// The exceptions that hold back a match of one of `terms`, each a plain
    /// term or a pattern, as a rule's `exceptions` do.
    ///
    /// # Errors
    ///
    /// Returns a [`PatternError`] when a term is not a pattern.
    pub fn of(terms: &[String]) -> Result<Self, PatternError> {
        build::exceptions_of(terms)
    }
}

/// An `existence` rule.
#[derive(Debug)]
pub struct Existence {
    /// The `raw` patterns and the `tokens`, joined.
    pub pattern: Pattern,
    /// What holds a match back.
    pub exceptions: Exceptions,
}

/// One swap of a `substitution` rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Swap {
    /// The text to find, as a pattern.
    pub key: String,
    /// The text to use instead.
    pub value: String,
}

impl Swap {
    /// The name of the group that holds swap number `n` in the rule's
    /// pattern is this, then the number.
    pub const GROUP: &'static str = "swap_";
}

/// A `substitution` rule.
#[derive(Debug)]
pub struct Substitution {
    /// The keys of the swap, joined, each in a group of its own.
    pub pattern: Pattern,
    /// The swaps, longest key first: swap number `n` is the group
    /// [`Swap::GROUP`] and `n`.
    pub swaps: Vec<Swap>,
    /// For each swap, in the same order, its key as a pattern of its own when
    /// the value holds `$1` or another group reference, to read the groups
    /// from the matched text.
    pub expanders: Vec<Option<Pattern>>,
    /// What holds a match back.
    pub exceptions: Exceptions,
    /// Whether the value takes a capital when the match does.
    pub capitalize: bool,
    /// Whether the keys match without regard to case.
    pub ignorecase: bool,
    /// Whether the rule is built from the accepted terms of a vocabulary: each
    /// value is a term, which may be a pattern. A match is in form when the
    /// term as a pattern matches it whole, and the term is shown in the case of
    /// the text it replaced.
    pub terms: bool,
}

/// An `occurrence` rule.
#[derive(Debug)]
pub struct Occurrence {
    /// The `token`, as written.
    pub pattern: Pattern,
    /// The most matches a text may hold, or 0 for no limit.
    pub max: usize,
    /// The fewest matches a text may hold, or 0 for no limit.
    pub min: usize,
}

/// A `metric` rule.
#[derive(Debug)]
pub struct Metric {
    /// The `formula`.
    pub formula: Formula,
    /// The `condition`, which reads as a comparison of the formula's result,
    /// such as `> 4`.
    pub condition: Formula,
    /// What the formula's `occurrences` counts, when the rule has `tokens` or
    /// `raw`.
    pub tokens: Option<Existence>,
}

/// A `consistency` rule.
#[derive(Debug)]
pub struct Consistency {
    /// One pattern for each pair of `either`, by key: the pair's two forms are
    /// the groups [`Consistency::FORM`] and 0, and [`Consistency::FORM`] and 1.
    pub pairs: Vec<Pattern>,
}

impl Consistency {
    /// The name of the group that holds a pair's form number `n` is this,
    /// then the number.
    pub const FORM: &'static str = "form_";
}

/// A `conditional` rule.
#[derive(Debug)]
pub struct Conditional {
    /// The `first` pattern, as written.
    pub first: Pattern,
    /// The `second` pattern, as written.
    pub second: Pattern,
    /// Whether `second` has a capture group, which makes the texts it captures
    /// the terms `first` may use; without one, `second` must match in the same
    /// block.
    pub defines: bool,
    /// Whether both patterns, and a term, match without regard to case.
    pub ignorecase: bool,
    /// What holds a match of `first` back.
    pub exceptions: Exceptions,
}

/// A `repetition` rule.
#[derive(Debug)]
pub struct Repetition {
    /// The `tokens`, joined into one group.
    pub pattern: Pattern,
    /// The most repeats a node may hold before one is reported.
    pub max: usize,
    /// Whether two matches are equal without regard to case.
    pub ignorecase: bool,
    /// With `alpha`, the pattern a match must satisfy whole: letters only.
    pub letters: Option<Pattern>,
    /// What holds a repeat back.
    pub exceptions: Exceptions,
}

/// A grade-level score a `readability` rule may average.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Score {
    /// `Automated Readability`.
    AutomatedReadability,
    /// `Coleman-Liau`.
    ColemanLiau,
    /// `Flesch-Kincaid`.
    FleschKincaid,
    /// `Gunning Fog`.
    GunningFog,
    /// `SMOG`.
    Smog,
}

impl Score {
    /// Every score, in the order the error for an unknown one lists them.
    pub const ALL: [Self; 5] = [
        Self::AutomatedReadability,
        Self::ColemanLiau,
        Self::FleschKincaid,
        Self::GunningFog,
        Self::Smog,
    ];

    /// The name a rule's `metrics` list uses.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AutomatedReadability => "Automated Readability",
            Self::ColemanLiau => "Coleman-Liau",
            Self::FleschKincaid => "Flesch-Kincaid",
            Self::GunningFog => "Gunning Fog",
            Self::Smog => "SMOG",
        }
    }

    /// The score named `name`, written as a rule's `metrics` list writes it.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|score| score.name() == name)
    }

    /// The metric variable that holds the score.
    pub(crate) const fn variable(self) -> &'static str {
        match self {
            Self::AutomatedReadability => "automated_readability",
            Self::ColemanLiau => "coleman_liau",
            Self::FleschKincaid => "flesch_kincaid",
            Self::GunningFog => "gunning_fog",
            Self::Smog => "smog",
        }
    }
}

/// A `readability` rule.
#[derive(Debug)]
pub struct Readability {
    /// The scores to average, at least one.
    pub scores: Vec<Score>,
    /// The highest average a text may have.
    pub grade: f64,
}

/// The case a `capitalization` rule requires.
#[derive(Debug)]
pub enum Case {
    /// `$title`, in one style.
    Title(Title),
    /// `$sentence`.
    Sentence(Sentence),
    /// `$lower`.
    Lower,
    /// `$upper`.
    Upper,
    /// Any other `match`: a pattern the text must match.
    Pattern(Pattern),
}

/// A `capitalization` rule.
#[derive(Debug)]
pub struct Capitalization {
    /// The `match`.
    pub case: Case,
    /// The share of words that must agree, 0.8 when the rule sets none.
    pub threshold: f64,
    /// The `exceptions` as one pattern: a text, or a word of one, that it
    /// matches passes.
    pub exceptions: Option<Pattern>,
    /// The pattern that cuts a text into the words the share counts, the
    /// exceptions first.
    pub words: Pattern,
}

/// One token of a `sequence` rule.
#[derive(Debug)]
pub struct Token {
    /// The `pattern`, matched against a whole word.
    pub word: Option<Pattern>,
    /// The `tag`, or the Penn tags a `upos` stands for, matched anywhere in
    /// the word's tag.
    pub tag: Option<Pattern>,
    /// The words a `upos` that Penn cannot tell apart is limited to, matched
    /// against a whole word.
    pub words: Option<Pattern>,
    /// Whether the token matches a word its pattern and tag do not match.
    pub negate: bool,
    /// Whether the alert spans this token's word alone.
    pub target: bool,
    /// The `skip`: how many words may come before the word.
    pub skip: usize,
    /// How many times the token repeats: its `min`, or 1.
    pub copies: usize,
    /// Whether the token has a step of its own after its optional ones: it
    /// has a `pattern`, a tag or a `upos`.
    pub required: bool,
}

impl Token {
    /// How many steps one copy of the token makes: its `skip` optional steps
    /// and the required one.
    fn width(&self) -> usize {
        self.skip + usize::from(self.required)
    }
}

/// One position a `sequence` walk checks: a token, or one copy of it.
///
/// A token with `skip: N` has N optional steps and the required one, and a
/// token with `min: N` repeats that N times.
/// The steps are counted, not stored, so that a large `skip` or `min` costs
/// nothing until a walk reaches them.
#[derive(Clone, Copy, Debug)]
pub struct Step {
    /// The index of the token in [`Form::tokens`].
    pub token: usize,
    /// Whether a word that does not match is passed over.
    pub optional: bool,
    /// Whether it is the first of the optional steps of a `skip`.
    pub start: bool,
    /// Whether it is the required step that the optional ones pad.
    pub end: bool,
    /// The first step that one occurrence of a token made.
    pub first: usize,
    /// The step after the last one that the occurrence made.
    pub after: usize,
}

/// A `sequence` rule: one or more forms, and the exceptions that apply to every
/// one.
///
/// A rule with `tokens` has one form. A rule with `forms` has one for each,
/// and it matches where any form does.
#[derive(Debug)]
pub struct Sequence {
    /// The forms, in the order the rule wrote them.
    pub forms: Vec<Form>,
    /// The `exceptions`: a match that begins inside a match of one is dropped.
    pub exceptions: Vec<Pattern>,
}

/// One list of tokens that a `sequence` rule matches.
#[derive(Debug)]
pub struct Form {
    /// The tokens as written.
    pub tokens: Vec<Token>,
    /// The number of steps up to the end of each token.
    pub ends: Vec<usize>,
    /// The step the walk starts from: the first with a `pattern`, or else the
    /// first with a tag, that is not negated.
    pub anchor: usize,
    /// Where the anchor's pattern occurs in a sentence, which bounds how many
    /// times the walk starts. A tag has no text to search for.
    pub candidates: Option<Pattern>,
}

impl Form {
    /// How many steps the tokens make.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ends.last().copied().unwrap_or(0)
    }

    /// Whether the tokens make no step.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The steps of token `token`.
    #[must_use]
    pub fn block(&self, token: usize) -> Range<usize> {
        let begin = token.checked_sub(1).map_or(0, |before| self.ends[before]);
        begin..self.ends[token]
    }

    /// The token that made step `index`.
    #[must_use]
    pub fn token_of(&self, index: usize) -> usize {
        self.ends.partition_point(|end| *end <= index)
    }

    /// Step `index`, which must be below [`Form::len`].
    #[must_use]
    pub fn step(&self, index: usize) -> Step {
        let token = self.token_of(index);
        let width = self.tokens[token].width();
        let offset = index - self.block(token).start;
        let within = offset % width;
        let first = index - within;
        Step {
            token,
            optional: within < self.tokens[token].skip,
            start: within == 0 && self.tokens[token].skip > 0,
            end: within == self.tokens[token].skip,
            first,
            after: first + width,
        }
    }
}

/// A `spelling` rule.
#[derive(Debug)]
pub struct Spelling {
    /// The dictionaries, and the words the rule's lists accept besides.
    pub speller: Speller,
    /// The `filters`: a word that one matches is skipped.
    pub filters: Vec<Pattern>,
    /// The accepted terms of the project's vocabularies as whole words matched
    /// without regard to case: a word that matches is skipped.
    pub accepted: Option<Pattern>,
    /// The multi-word accepted terms, which are masked before the words are
    /// read.
    pub masks: Option<Pattern>,
    /// Whether the built-in filters skip names and codes: the rule does not set
    /// `custom`.
    pub standard: bool,
    /// Whether an identifier is checked part by part, and each misspelt part
    /// is reported at its own place.
    pub split: bool,
}

/// One thing wrong with a rule file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadProblem {
    /// The rule file, or the style folder when it could not be read.
    pub file: String,
    /// The 1-based line, when known.
    pub line: Option<usize>,
    /// The 1-based column, when known.
    pub column: Option<usize>,
    /// The field it concerns, empty for a problem with the whole file.
    pub field: String,
    /// What a person changes to fix it.
    pub message: String,
    /// Whether the rule is valid Vale that hakkiri cannot run, such as a
    /// `script` rule. A config turns such a rule `off` to leave it out.
    pub unrunnable: bool,
}

/// Why rules did not load: every problem found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadError {
    /// What is wrong, in the order found.
    pub problems: Vec<LoadProblem>,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, problem) in self.problems.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", problem.file)?;
            if let Some(line) = problem.line {
                write!(f, ":{line}")?;
                if let Some(column) = problem.column {
                    write!(f, ":{column}")?;
                }
            }
            if problem.field.is_empty() {
                write!(f, ": {}", problem.message)?;
            } else {
                write!(f, ": {}: {}", problem.field, problem.message)?;
            }
        }
        Ok(())
    }
}

impl error::Error for LoadError {}

impl LoadError {
    /// Whether every problem is a rule that is valid Vale and that hakkiri
    /// cannot run.
    #[must_use]
    pub fn is_unrunnable(&self) -> bool {
        !self.problems.is_empty() && self.problems.iter().all(|problem| problem.unrunnable)
    }
}

impl From<InputError> for LoadError {
    fn from(error: InputError) -> Self {
        Self {
            problems: error
                .problems
                .into_iter()
                .map(|problem| LoadProblem {
                    file: error.name.clone(),
                    line: problem.line,
                    column: problem.column,
                    field: problem.field,
                    message: problem.message,
                    unrunnable: false,
                })
                .collect(),
        }
    }
}

impl Rule {
    /// Reads one rule from the text of its file.
    ///
    /// `name` is the rule's name, such as `Team.Filler`, and `file` names the
    /// file in the errors.
    /// A rule read this way holds no package, so a `spelling` rule that names
    /// a dictionary or a word list is refused.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] when the text is not YAML or names no check
    /// type. The same holds for a field its check does not define, a value of
    /// the wrong kind, and a feature hakkiri does not run. A pattern that does
    /// not compile is refused, and so is a pattern with nothing to match.
    pub fn from_text(name: &str, file: &str, text: &str) -> Result<Self, LoadError> {
        Self::read(name, file, text, None, &[], &[], &[])
    }

    /// Reads one rule from the text of its file, in the package `package`.
    ///
    /// The package is the folder that holds the rule's style folder.
    /// A `spelling` rule reads its dictionaries and word lists from it, and
    /// from no other place.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] for each fault [`Rule::from_text`] refuses.
    /// A dictionary or word list that is missing or unreadable is refused,
    /// and so is a path that leaves the package.
    pub fn from_text_in(
        name: &str,
        file: &str,
        text: &str,
        package: &Path,
    ) -> Result<Self, LoadError> {
        Self::read(
            name,
            file,
            text,
            Some(package),
            &[package.to_path_buf()],
            &[],
            &[],
        )
    }

    /// Reads one rule from the text of its file, in the package `package`,
    /// with a parent named by `extends` looked up in the package folders
    /// `search`.
    ///
    /// A rule whose `extends` holds a dot starts from the rule it names and
    /// lays its own fields on top, as Vale does: a bare key replaces, `key+`
    /// adds to a list or a mapping, and `key-` removes from one.
    /// The parent's `tests` do not carry over.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] for each fault [`Rule::from_text_in`] refuses.
    /// It also returns each fault Vale refuses in a chain of rules.
    /// They are a parent that is missing, a cycle, and a key written both bare
    /// and with `+` or `-`.
    /// They are also an edit of a field the parent lacks, and a `Vale.` parent.
    /// A rule read through a parent names no line, since the merged text has
    /// none.
    /// `accepted` holds the accepted terms of the project's vocabularies, which
    /// the checks that read them take as exceptions.
    pub fn from_text_searching(
        name: &str,
        file: &str,
        text: &str,
        package: &Path,
        search: &[PathBuf],
        accepted: &[String],
    ) -> Result<Self, LoadError> {
        Self::read(name, file, text, Some(package), search, accepted, &[])
    }

    /// Reads this rule again with `params` laid over its fields, a bare key
    /// replacing a field as in a rule that extends another.
    ///
    /// `search` and `accepted` are what the rule was first read with.
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] with a problem for each parameter that is a
    /// level, which the rule's own setting carries, and for each that is not
    /// adjustable.
    /// The parameters that are not adjustable are `extends`, `name`, `path`,
    /// `tests`, `message`, `description`, `link`, `tokens`, `forms`, `swap`,
    /// `exceptions`, `filters`, `ignore`, `raw` and `either`.
    /// A parameter the rule's check does not define, or whose value is of the
    /// wrong kind, is refused as the same field of a rule file is, naming the
    /// field and no line.
    /// A rule built from a vocabulary has no field to set, and refuses every
    /// parameter.
    pub fn with_params(
        &self,
        search: &[PathBuf],
        accepted: &[String],
        params: &[(String, Param)],
    ) -> Result<Self, LoadError> {
        let refuse = |field: &str, message: String| LoadProblem {
            file: self.name.clone(),
            line: None,
            column: None,
            field: field.to_owned(),
            message,
            unrunnable: false,
        };
        let problems: Vec<LoadProblem> = params
            .iter()
            .filter_map(|(field, _)| {
                if field == "level" {
                    Some(refuse(
                        field,
                        format!(
                            "is not a parameter: set the level with the rule's own setting, such as `{}: warning`",
                            self.name
                        ),
                    ))
                } else if NOT_PARAMETERS.contains(&field.as_str()) {
                    Some(refuse(
                        field,
                        format!(
                            "is not adjustable as a parameter: extend `{}` in a style instead",
                            self.name
                        ),
                    ))
                } else {
                    None
                }
            })
            .collect();
        if !problems.is_empty() {
            return Err(LoadError { problems });
        }
        match &self.recipe {
            Recipe::File {
                file,
                text,
                package,
            } => Self::read(
                &self.name,
                file,
                text,
                package.as_deref(),
                search,
                accepted,
                params,
            ),
            Recipe::Fixed => Err(LoadError {
                problems: params
                    .iter()
                    .map(|(field, _)| {
                        refuse(
                            field,
                            "is not a parameter: this rule is built from a vocabulary and takes none"
                                .to_owned(),
                        )
                    })
                    .collect(),
            }),
        }
    }

    fn read(
        name: &str,
        file: &str,
        text: &str,
        package: Option<&Path>,
        search: &[PathBuf],
        accepted: &[String],
        params: &[(String, Param)],
    ) -> Result<Self, LoadError> {
        let input = Input::new(file, text);
        let probe: Probe = input.load().map_err(|error| at_tab(error, text))?;
        let mut rule = if inherit::is_rule_ref(probe.extends.value()) || !params.is_empty() {
            let merged = inherit::merged(file, text, search, params)?;
            let input = Input::new(file, merged);
            let probe: Probe = input
                .load()
                .map_err(|error| unplaced(LoadError::from(error)))?;
            Self::build(name, file, package, search, accepted, &probe, &input).map_err(unplaced)?
        } else {
            Self::build(name, file, package, search, accepted, &probe, &input)?
        };
        rule.recipe = Recipe::File {
            file: file.to_owned(),
            text: text.to_owned(),
            package: package.map(Path::to_path_buf),
        };
        Ok(rule)
    }

    /// The rule `probe` and `input` hold, once its `extends` is a check type.
    fn build(
        name: &str,
        file: &str,
        package: Option<&Path>,
        folders: &[PathBuf],
        accepted: &[String],
        probe: &Probe,
        input: &Input,
    ) -> Result<Self, LoadError> {
        let source = Source {
            name,
            file,
            package,
            folders,
            accepted,
        };
        if let Some(reason) = unrunnable(probe, input)? {
            return Err(LoadError {
                problems: vec![LoadProblem {
                    file: file.to_owned(),
                    line: None,
                    column: None,
                    field: reason.0.to_owned(),
                    message: reason.1,
                    unrunnable: true,
                }],
            });
        }
        match probe.extends.value() {
            "existence" => build::existence_rule(&source, &read::<ExistenceFields>(input)?),
            "substitution" => {
                build::substitution_rule(&source, &read::<SubstitutionFields>(input)?)
            }
            "occurrence" => build::occurrence_rule(&source, &read::<OccurrenceFields>(input)?),
            "conditional" => build::conditional_rule(&source, &read::<ConditionalFields>(input)?),
            "consistency" => build::consistency_rule(&source, &read::<ConsistencyFields>(input)?),
            "repetition" => build::repetition_rule(&source, &read::<RepetitionFields>(input)?),
            "readability" => build::readability_rule(&source, &read::<ReadabilityFields>(input)?),
            "capitalization" => {
                build::capitalization_rule(&source, &read::<CapitalizationFields>(input)?)
            }
            "sequence" => build::sequence_rule(&source, &read::<SequenceFields>(input)?),
            "spelling" => build::spelling_rule(&source, &read::<SpellingFields>(input)?),
            _ => build::metric_rule(&source, &read::<MetricFields>(input)?),
        }
    }
}

/// Why a rule that is valid Vale cannot run here: the field it concerns and
/// what hakkiri does not run.
fn unrunnable(probe: &Probe, input: &Input) -> Result<Option<(&'static str, String)>, LoadError> {
    if probe.extends.value() == "script" {
        return Ok(Some((
            "extends",
            "`script` rules run a Tengo program, and hakkiri runs none".to_owned(),
        )));
    }
    if input.text().contains("doc(") {
        let scopes: ScopeProbe = read(input)?;
        if let Some(scope) = scopes
            .scope
            .values()
            .iter()
            .find(|scope| scope.trim_start_matches('~').starts_with("doc("))
        {
            return Ok(Some((
                "scope",
                format!(
                    "`{scope}` is a doc selector: it selects by CSS over rendered HTML, and hakkiri runs none"
                ),
            )));
        }
    }
    Ok(None)
}

/// `error` with the lines and columns it names taken out, for a rule read
/// from merged text, whose lines nobody wrote.
fn unplaced(mut error: LoadError) -> LoadError {
    for problem in &mut error.problems {
        problem.line = None;
        problem.column = None;
    }
    error
}

/// The fields of one rule file type, parsed and validated once.
fn read<T>(input: &Input) -> Result<T, LoadError>
where
    T: DeserializeOwned + garde::Validate<Context = ()>,
{
    input
        .load::<T>()
        .map_err(|error| at_tab(error, input.text()))
}

/// `error` with a tab refusal placed on the line of the tab.
///
/// The YAML parser finds a tab in an indentation only after it has read the
/// scalar above it, and reports the line of that scalar.
/// The tab is on the first line after it that is indented with one.
fn at_tab(mut error: InputError, text: &str) -> LoadError {
    for problem in &mut error.problems {
        if problem.field.is_empty()
            && problem.message.contains("found a tab")
            && let Some(first) = problem.line
            && let Some((index, line)) = text
                .lines()
                .enumerate()
                .skip(first.saturating_sub(1))
                .find(|(_, line)| {
                    line.chars()
                        .take_while(|c| c.is_whitespace())
                        .any(|c| c == '\t')
                })
        {
            problem.line = Some(index + 1);
            problem.column = line.find('\t').map(|at| at + 1);
        }
    }
    LoadError::from(error)
}

/// The name the rule in `file` has in the style folder `style`: the folder's
/// name, then each folder between it and the file, then the file name up to
/// its first dot, all joined by dots.
/// So `Team/Foo.yml` is `Team.Foo`, `Team/Terms.custom.yml` is `Team.Terms`,
/// and `Team/Sub/Bar.yml` is `Team.Sub.Bar`, as Vale names them.
///
/// [`load_style`] names its rules this way, and so does every caller that
/// must agree with it.
///
/// # Errors
///
/// Returns why `file` has no name: it is not in `style`, a name is not
/// UTF-8, or the name holds a bracket, which Vale keeps for the settings of a
/// rule.
pub fn rule_name(style: &Path, file: &Path) -> Result<String, String> {
    let folder = style
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("the style folder has no name")?;
    let relative = file
        .strip_prefix(style)
        .ok()
        .ok_or("the rule is not in the style folder")?;
    let mut parts = Vec::new();
    for part in relative.components() {
        parts.push(part.as_os_str().to_str().ok_or("a name is not UTF-8")?);
    }
    let last = parts.pop().ok_or("the rule has no file name")?;
    parts.push(last.split('.').next().unwrap_or(last));
    let name = format!("{folder}.{}", parts.join("."));
    if name.contains(['[', ']']) {
        return Err(format!(
            "`{name}` has a bracket, which a rule name cannot hold"
        ));
    }
    Ok(name)
}

/// Whether `path` is a rule file: a file with the extension `yml` that is no
/// `*.test.yml` file of cases.
fn is_rule_file(path: &Path) -> bool {
    let is_test = path
        .file_stem()
        .map(Path::new)
        .and_then(Path::extension)
        .is_some_and(|extension| extension == "test");
    path.extension().is_some_and(|extension| extension == "yml") && !is_test && path.is_file()
}

/// The rule files under `folder`, in the order Vale walks them: each folder's
/// entries by name, a subfolder where it sits, and no folder whose name starts
/// with `.` or `_`.
/// A `*.test.yml` file holds cases and is no rule.
fn rule_files(folder: &Path, files: &mut Vec<PathBuf>) -> Result<(), (PathBuf, io::Error)> {
    let fail = |error| (folder.to_path_buf(), error);
    let mut entries = fs::read_dir(folder)
        .map_err(fail)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(fail)?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type().map_err(fail)?.is_dir() {
            if !name.starts_with(['.', '_']) {
                rule_files(&path, files)?;
            }
        } else if is_rule_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

/// A style folder read: its rules, and the faults in the names of its files.
#[derive(Debug)]
pub struct Style {
    /// The rules, in the order of the walk.
    pub rules: Vec<Rule>,
    /// A file whose rule name holds a bracket, which gives no rule, or that
    /// gives the name of an earlier file, whose rule it replaces as Vale does.
    /// Vale refuses these only for a style a glob enables, so a caller decides
    /// whether they are errors.
    pub names: Vec<LoadProblem>,
    /// The rules that are valid Vale and that hakkiri cannot run, such as a
    /// `script` rule. They give no rule; a caller decides whether each is an
    /// error.
    pub unrunnable: Vec<Unrunnable>,
}

/// A rule that is valid Vale and that hakkiri cannot run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Unrunnable {
    /// The rule's name, such as `Readability.Polysyllables`.
    pub name: String,
    /// The rule file.
    pub file: String,
    /// The field that makes it so, such as `extends`.
    pub field: String,
    /// What hakkiri does not run.
    pub message: String,
}

fn problem(file: &str, message: String) -> LoadProblem {
    LoadProblem {
        file: file.to_owned(),
        line: None,
        column: None,
        field: String::new(),
        message,
        unrunnable: false,
    }
}

/// Reads every rule file under the style folder `folder`.
///
/// A rule is named as [`rule_name`] names it, so a file in a subfolder joins
/// the folders to its name.
/// A folder that starts with `.` or `_`, a `*.test.yml` file and every other
/// file are ignored.
/// The folder's parent is the rule's package.
/// A rule whose `extends` names another rule looks it up in the package
/// folders `search`.
/// The rules come in the order of the walk.
/// `accepted` holds the accepted terms of the project's vocabularies, which the
/// checks that read them take as exceptions.
///
/// # Errors
///
/// Returns a [`LoadError`] with the problems of every rule file, or with the
/// failure to read a folder.
/// A fault in a name is not one: [`Style::names`] holds it, and a rule that
/// hakkiri cannot run is not one: [`Style::unrunnable`] holds it.
pub fn read_style(
    folder: &Path,
    search: &[PathBuf],
    accepted: &[String],
) -> Result<Style, LoadError> {
    let unread = |file: &Path, message: String| LoadError {
        problems: vec![problem(&file.display().to_string(), message)],
    };
    if folder.file_name().and_then(|name| name.to_str()).is_none() {
        return Err(unread(folder, "the style folder has no name".to_owned()));
    }
    let package = folder
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut files = Vec::new();
    rule_files(folder, &mut files).map_err(|(at, error)| unread(&at, error.to_string()))?;
    let mut rules: Vec<Rule> = Vec::new();
    let mut names = Vec::new();
    let mut unrunnable = Vec::new();
    let mut problems = Vec::new();
    let mut by_name: HashMap<String, PathBuf> = HashMap::new();
    for path in files {
        let file = path.display().to_string();
        let name = match rule_name(folder, &path) {
            Ok(name) => name,
            Err(message) => {
                names.push(problem(&file, message));
                continue;
            }
        };
        if let Some(first) = by_name.get(&name) {
            names.push(problem(
                &file,
                format!(
                    "the rule `{name}` is also the name of `{}`: a rule name ends at the first dot of its file name, so rename one file",
                    first.display()
                ),
            ));
        } else {
            by_name.insert(name.clone(), path.clone());
        }
        let read = fs::read_to_string(&path)
            .map_err(|error| LoadError {
                problems: vec![problem(&file, error.to_string())],
            })
            .and_then(|text| {
                Rule::from_text_searching(&name, &file, &text, package, search, accepted)
            });
        match read {
            Ok(rule) => {
                // The later file wins, as in Vale: it replaces an earlier
                // rule of the name, runnable or not.
                unrunnable.retain(|earlier: &Unrunnable| earlier.name != rule.name);
                match rules.iter().position(|earlier| earlier.name == rule.name) {
                    Some(at) => rules[at] = rule,
                    None => rules.push(rule),
                }
            }
            Err(error) if error.is_unrunnable() => {
                rules.retain(|earlier| earlier.name != name);
                unrunnable.retain(|earlier| earlier.name != name);
                unrunnable.extend(error.problems.into_iter().map(|problem| Unrunnable {
                    name: name.clone(),
                    file: problem.file,
                    field: problem.field,
                    message: problem.message,
                }));
            }
            Err(error) => problems.extend(error.problems),
        }
    }
    if problems.is_empty() {
        Ok(Style {
            rules,
            names,
            unrunnable,
        })
    } else {
        Err(LoadError { problems })
    }
}

/// Loads every rule file under the style folder `folder`, as [`read_style`]
/// reads them, and refuses a fault in a name.
///
/// # Errors
///
/// Returns a [`LoadError`] with the problems of every rule file, with the
/// failure to read a folder, or with the faults in the names.
pub fn load_style(folder: &Path) -> Result<Vec<Rule>, LoadError> {
    let package = folder
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let style = read_style(folder, &[package.to_path_buf()], &[])?;
    let mut problems = style.names;
    problems.extend(style.unrunnable.into_iter().map(|rule| LoadProblem {
        file: rule.file,
        line: None,
        column: None,
        field: rule.field,
        message: rule.message,
        unrunnable: true,
    }));
    if problems.is_empty() {
        Ok(style.rules)
    } else {
        Err(LoadError { problems })
    }
}

/// The rule file of `Vale.Repetition`: Vale's own definition, with the level
/// `error`, the scope `text`, the tokens `[^\s.!?]+`, `ignorecase` and `alpha`.
/// Its `vocab` is on, as in any rule that reads accepted terms by default.
const VALE_REPETITION: &str = r#"extends: repetition
message: "'%s' is repeated!"
level: error
scope: text
ignorecase: true
alpha: true
tokens: ['[^\s.!?]+']
"#;

/// The rule `Vale.Repetition`, which Vale builds into its style `Vale`: it
/// reports a word that repeats, such as `the the`, at the level `error`.
/// `project_accepted` holds the accepted terms of the project, which hold a
/// repeat back.
///
/// # Errors
///
/// Returns a [`LoadError`] when an accepted term is not a pattern.
pub fn vale_repetition(project_accepted: &[String]) -> Result<Rule, LoadError> {
    Rule::read(
        "Vale.Repetition",
        "the built-in style Vale",
        VALE_REPETITION,
        None,
        &[],
        project_accepted,
        &[],
    )
}

/// The rule file of `Vale.Spelling`: Vale's own definition, with the level
/// `error` and the scope `text`. The built-in dictionary, the dictionaries of
/// the packages' `config/dictionaries` folders, and the word lists of their
/// `config/ignore` folders come from [`vale_spelling`].
const VALE_SPELLING: &str = r#"extends: spelling
message: "Did you really mean '%s'?"
level: error
scope: text
"#;

/// The rule `Vale.Spelling`, which Vale builds into its style `Vale`: it
/// reports a word the built-in dictionary does not hold, at the level
/// `error`.
///
/// Vale adds to the built-in dictionary every dictionary in the
/// `config/dictionaries` folder of its styles path, and accepts every word of
/// the `.txt` files under its `config/ignore` folder. The rule does the same
/// for each folder of `folders`, the package folders of the project, in order.
/// `project_accepted` holds the accepted terms of the project, which hold a
/// word back.
///
/// # Errors
///
/// Returns a [`LoadError`] when an accepted term is not a pattern, or a
/// dictionary or word list of a folder cannot be read.
pub fn vale_spelling(folders: &[PathBuf], project_accepted: &[String]) -> Result<Rule, LoadError> {
    Rule::read(
        VALE_SPELLING_NAME,
        "the built-in style Vale",
        VALE_SPELLING,
        None,
        folders,
        project_accepted,
        &[],
    )
}

/// The name of the rule `Vale.Spelling`.
pub(crate) const VALE_SPELLING_NAME: &str = "Vale.Spelling";

/// The rules a vocabulary builds, named `{prefix}.Terms` and `{prefix}.Avoid`,
/// as Vale builds them.
///
/// `Terms` reports an accepted term written in another case, and is there only
/// when `accepted` holds a term. `Avoid` reports a rejected term, and is there
/// only when `rejected` holds one. Both have the level `error` and the scope
/// `text`, and a rule that is there runs on no file until a config lists it.
/// `project_accepted` holds the accepted terms of the project, which hold a
/// match of `Avoid` back.
///
/// # Errors
///
/// Returns a [`LoadError`] when a term is not a pattern.
pub fn vocabulary_rules(
    prefix: &str,
    accepted: &[String],
    rejected: &[String],
    project_accepted: &[String],
) -> Result<Vec<Rule>, LoadError> {
    let mut rules = Vec::new();
    if !accepted.is_empty() {
        rules.push(build::terms_rule(&format!("{prefix}.Terms"), accepted)?);
    }
    if !rejected.is_empty() {
        rules.push(build::avoid_rule(
            &format!("{prefix}.Avoid"),
            rejected,
            project_accepted,
        )?);
    }
    Ok(rules)
}
