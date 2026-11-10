//! Rules: loading a style folder into the rules the engine runs.
//!
//! A style folder holds one `.yml` file per rule.
//! [`load_style`] reads them all, and [`Rule::from_text`] reads one.
//! Loading refuses a rule it cannot run: a field its check does not define, a
//! value of the wrong kind, a pattern that does not compile, or a Vale
//! feature hakkiri does not run.
//! Every error names the file, and the field and line when it has them.

mod build;
mod field;
mod schema;

use std::path::Path;
use std::{error, fmt, fs};

use ctl_core::input::{Input, InputError};
use serde::de::DeserializeOwned;

use self::build::Source;
use self::schema::{
    ExistenceFields, MetricFields, OccurrenceFields, Probe, ReadabilityFields, RepetitionFields,
    SubstitutionFields,
};
use crate::formula::Formula;
use crate::pattern::Pattern;
use crate::scope::Selector;

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
    /// What the rule checks.
    pub check: Check,
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
    /// Reports a match that repeats the one before it.
    Repetition(Repetition),
    /// Averages grade-level scores of the text, and tests the average.
    Readability(Readability),
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
    ///
    /// # Errors
    ///
    /// Returns a [`LoadError`] when the text is not YAML or names no check
    /// type. The same holds for a field its check does not define, a value of
    /// the wrong kind, and a feature hakkiri does not run. A pattern that does
    /// not compile is refused, and so is a pattern with nothing to match.
    pub fn from_text(name: &str, file: &str, text: &str) -> Result<Self, LoadError> {
        let input = Input::new(file, text);
        let probe: Probe = input.load().map_err(|error| at_tab(error, text))?;
        let source = Source { name, file };
        match probe.extends.value() {
            "existence" => build::existence_rule(&source, &read::<ExistenceFields>(&input)?),
            "substitution" => {
                build::substitution_rule(&source, &read::<SubstitutionFields>(&input)?)
            }
            "occurrence" => build::occurrence_rule(&source, &read::<OccurrenceFields>(&input)?),
            "repetition" => build::repetition_rule(&source, &read::<RepetitionFields>(&input)?),
            "readability" => build::readability_rule(&source, &read::<ReadabilityFields>(&input)?),
            _ => build::metric_rule(&source, &read::<MetricFields>(&input)?),
        }
    }
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

/// Loads every `.yml` file directly inside the style folder `folder`.
///
/// A rule is named by the folder, a dot, and its file name without the
/// extension. Every other file in the folder is ignored.
/// The rules come in file name order.
///
/// # Errors
///
/// Returns a [`LoadError`] with the problems of every rule file, or with the
/// failure to read the folder.
pub fn load_style(folder: &Path) -> Result<Vec<Rule>, LoadError> {
    let unread = |message: String| LoadError {
        problems: vec![LoadProblem {
            file: folder.display().to_string(),
            line: None,
            column: None,
            field: String::new(),
            message,
        }],
    };
    let style = folder
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| unread("the style folder has no name".to_owned()))?;
    let entries = fs::read_dir(folder).map_err(|error| unread(error.to_string()))?;
    let mut files = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| unread(error.to_string()))?.path();
        if path.is_file() && path.extension().is_some_and(|extension| extension == "yml") {
            files.push(path);
        }
    }
    files.sort();
    let mut rules = Vec::new();
    let mut problems = Vec::new();
    for path in files {
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default();
        let file = path.display().to_string();
        let rule = fs::read_to_string(&path)
            .map_err(|error| LoadError {
                problems: vec![LoadProblem {
                    file: file.clone(),
                    line: None,
                    column: None,
                    field: String::new(),
                    message: error.to_string(),
                }],
            })
            .and_then(|text| Rule::from_text(&format!("{style}.{stem}"), &file, &text));
        match rule {
            Ok(rule) => rules.push(rule),
            Err(error) => problems.extend(error.problems),
        }
    }
    if problems.is_empty() {
        Ok(rules)
    } else {
        Err(LoadError { problems })
    }
}
