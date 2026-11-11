//! What a run gives back: one value that ctl-core prints as lines or as JSON.

use std::path::Path;

use ctl_core::view::Outcome;
use ctl_core::{Document, Present};
use hakkiri::alert::Alert;
use hakkiri::rule::{Level, LoadError, Unrunnable};
use serde::Serialize;

/// The result of a verb.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum Output {
    /// `hakkiri check`.
    Check(Check),
    /// `hakkiri test`.
    Test(Tests),
    /// `hakkiri instructions`.
    Instructions(Instructions),
}

impl Present for Output {
    fn present(&self) -> Document {
        match self {
            Self::Check(check) => check.present(),
            Self::Test(tests) => tests.present(),
            Self::Instructions(instructions) => instructions.present(),
        }
    }

    fn outcome(&self) -> Outcome {
        match self {
            Self::Check(check) => check.outcome(),
            Self::Test(tests) => tests.outcome(),
            Self::Instructions(instructions) => instructions.outcome(),
        }
    }

    fn exit_code(&self) -> u8 {
        match self {
            Self::Check(check) => check.exit_code(),
            Self::Test(tests) => tests.exit_code(),
            Self::Instructions(instructions) => instructions.exit_code(),
        }
    }
}

/// The operator contract built into the binary.
#[derive(Debug, Serialize)]
pub(crate) struct Instructions {
    pub(crate) markdown: String,
}

impl Present for Instructions {
    fn present(&self) -> Document {
        Document::new().verbatim(self.markdown.clone())
    }
}

/// One alert, placed in a file.
#[derive(Debug, Serialize)]
pub(crate) struct Row {
    path: String,
    line: usize,
    column: usize,
    end_column: usize,
    level: &'static str,
    rule: String,
    message: String,
    description: String,
    link: String,
    #[serde(rename = "match")]
    matched: String,
    #[serde(skip)]
    severity: Level,
}

impl Row {
    /// The row for `alert` in the file `path`.
    pub(crate) fn new(path: &str, alert: Alert) -> Self {
        Self {
            path: path.to_owned(),
            line: alert.line,
            column: alert.first_column,
            end_column: alert.last_column,
            level: alert.level.name(),
            rule: alert.rule,
            message: alert.message,
            description: alert.description,
            link: alert.link,
            matched: alert.matched,
            severity: alert.level,
        }
    }

    /// The row's place, for sorting.
    pub(crate) fn place(&self) -> (&str, usize, usize) {
        (&self.path, self.line, self.column)
    }
}

/// Something that could not run: a file, or the config.
#[derive(Debug, Serialize)]
pub(crate) struct Failure {
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<String>,
    message: String,
}

impl Failure {
    /// A file that could not be read, parsed or linted.
    pub(crate) fn file(path: &str, message: impl Into<String>) -> Self {
        Self {
            path: Some(path.to_owned()),
            config: None,
            message: message.into(),
        }
    }

    /// The faults of the config or of a rule file, each placed on its line.
    pub(crate) fn config(error: &LoadError) -> Vec<Self> {
        error
            .problems
            .iter()
            .map(|problem| {
                let line = problem
                    .line
                    .map(|line| format!("line {line}: "))
                    .unwrap_or_default();
                let field = if problem.field.is_empty() {
                    String::new()
                } else {
                    format!("{}: ", problem.field)
                };
                Self {
                    path: None,
                    config: Some(problem.file.clone()),
                    message: format!("{line}{field}{}", problem.message),
                }
            })
            .collect()
    }

    /// A rule hakkiri cannot run that the config lists: it holds no rule to run
    /// and fails the run until the config turns it `off`.
    pub(crate) fn unrunnable(rule: &Unrunnable) -> Self {
        Self {
            path: None,
            config: Some(rule.file.clone()),
            message: format!(
                "{}: {}: {}: set `{}: off` under `files` to leave it out",
                rule.name, rule.field, rule.message, rule.name
            ),
        }
    }

    /// A fault in the project root of the config file `config`.
    pub(crate) fn root(config: &Path, message: impl Into<String>) -> Self {
        Self {
            path: None,
            config: Some(config.display().to_string()),
            message: message.into(),
        }
    }

    /// The path or config file the failure is about.
    pub(crate) fn subject(&self) -> &str {
        self.path
            .as_deref()
            .or(self.config.as_deref())
            .unwrap_or_default()
    }
}

/// The result of `hakkiri check`.
#[derive(Debug, Serialize)]
pub(crate) struct Check {
    alerts: Vec<Row>,
    failures: Vec<Failure>,
    files: usize,
    /// Whether the run stopped before it linted a file.
    #[serde(skip)]
    stopped: bool,
    /// The lowest level of an alert that fails the run.
    #[serde(skip)]
    fail_level: Level,
}

impl Check {
    /// A run that linted `files` files.
    pub(crate) fn new(
        mut alerts: Vec<Row>,
        mut failures: Vec<Failure>,
        files: usize,
        fail_level: Level,
    ) -> Self {
        alerts.sort_by(|a, b| a.place().cmp(&b.place()));
        failures.sort_by(|a, b| a.subject().cmp(b.subject()));
        Self {
            alerts,
            failures,
            files,
            stopped: false,
            fail_level,
        }
    }

    /// A run the config or a rule stopped before any file was linted.
    pub(crate) fn stopped(error: &LoadError, fail_level: Level) -> Self {
        Self {
            alerts: Vec::new(),
            failures: Failure::config(error),
            files: 0,
            stopped: true,
            fail_level,
        }
    }

    /// `1 alert in 3 files`.
    fn count(&self) -> String {
        let alerts = self.alerts.len();
        let files = self.files;
        format!(
            "{alerts} alert{} in {files} file{}",
            if alerts == 1 { "" } else { "s" },
            if files == 1 { "" } else { "s" },
        )
    }
}

impl Present for Check {
    /// A line per alert, then a line per failure, then the count: plain text,
    /// because tools read these lines and a frame or a colour would break them.
    fn present(&self) -> Document {
        let mut lines: Vec<String> = self
            .alerts
            .iter()
            .map(|row| {
                format!(
                    "{}:{}:{}: {} {}: {}",
                    row.path, row.line, row.column, row.level, row.rule, row.message
                )
            })
            .collect();
        lines.extend(
            self.failures
                .iter()
                .map(|failure| format!("failed {}: {}", failure.subject(), failure.message)),
        );
        if !self.stopped {
            lines.push(self.count());
        }
        Document::new().verbatim(lines.join("\n"))
    }

    fn outcome(&self) -> Outcome {
        if self.stopped {
            Outcome::Error
        } else {
            Outcome::Success
        }
    }

    fn exit_code(&self) -> u8 {
        if self.stopped || !self.failures.is_empty() {
            2
        } else {
            u8::from(
                self.alerts
                    .iter()
                    .any(|row| row.severity >= self.fail_level),
            )
        }
    }
}

/// What running one case gave.
#[derive(Debug, Serialize)]
pub(crate) struct CaseResult {
    pub(crate) file: String,
    pub(crate) name: String,
    pub(crate) passed: bool,
    /// Why the case failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reason: Option<String>,
    /// Why the case could not run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
    /// The alerts the case got, one per line, when it failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) got: Option<String>,
    /// The output the case wanted, when it failed against one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) want: Option<String>,
    /// The case's note for the reader.
    #[serde(skip)]
    pub(crate) about: String,
}

/// A rule no case makes flag anything.
#[derive(Debug, Serialize)]
pub(crate) struct Uncovered {
    /// The rule name.
    pub(crate) rule: String,
    /// The rule files that give the name, only when more than one does.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) files: Vec<String>,
}

/// The result of `hakkiri test`.
#[derive(Debug, Serialize)]
pub(crate) struct Tests {
    passed: usize,
    failed: usize,
    files: usize,
    results: Vec<CaseResult>,
    uncovered: Vec<Uncovered>,
    failures: Vec<Failure>,
    /// Whether the run checked coverage.
    #[serde(skip)]
    coverage: bool,
}

impl Tests {
    /// A run over `files` files with the `results` of its cases.
    pub(crate) fn new(
        files: usize,
        results: Vec<CaseResult>,
        uncovered: Vec<Uncovered>,
        mut failures: Vec<Failure>,
        coverage: bool,
    ) -> Self {
        failures.sort_by(|a, b| a.subject().cmp(b.subject()));
        let failed = results.iter().filter(|result| !result.passed).count();
        Self {
            passed: results.len() - failed,
            failed,
            files,
            results,
            uncovered,
            failures,
            coverage,
        }
    }

    /// `3 files, 41 passed, 1 failed`, and the uncovered rules when coverage
    /// ran.
    fn count(&self) -> String {
        let files = self.files;
        let uncovered = if self.coverage {
            format!(", {} uncovered", self.uncovered.len())
        } else {
            String::new()
        };
        format!(
            "{files} file{}, {} passed, {} failed{uncovered}",
            if files == 1 { "" } else { "s" },
            self.passed,
            self.failed
        )
    }
}

impl CaseResult {
    /// The lines that say why the case failed or could not run.
    fn lines(&self) -> Vec<String> {
        if let Some(error) = &self.error {
            return vec![format!(
                "could not run {}: {}: {error}",
                self.file, self.name
            )];
        }
        let Some(reason) = &self.reason else {
            return Vec::new();
        };
        let mut lines = vec![
            format!("failed {}: {}", self.file, self.name),
            format!("  {reason}"),
        ];
        if !self.about.is_empty() {
            lines.push(format!("  about: {}", self.about));
        }
        let got = self.got.as_deref().unwrap_or_default().trim();
        match &self.want {
            Some(want) => {
                lines.extend(want.trim().lines().map(|line| format!("  - {line}")));
                lines.extend(got.lines().map(|line| format!("  + {line}")));
            }
            None if got.is_empty() => lines.push("  got: no alerts".to_owned()),
            None => {
                lines.push("  got:".to_owned());
                lines.extend(got.lines().map(|line| format!("    {line}")));
            }
        }
        lines
    }
}

impl Present for Tests {
    /// A block per failed case, a line per rule no case fires, and the count:
    /// plain text, as the lines of `check` are.
    fn present(&self) -> Document {
        let mut lines: Vec<String> = self.results.iter().flat_map(CaseResult::lines).collect();
        lines.extend(
            self.failures
                .iter()
                .map(|failure| format!("failed {}: {}", failure.subject(), failure.message)),
        );
        lines.extend(self.uncovered.iter().map(|entry| {
            let rule = &entry.rule;
            if entry.files.is_empty() {
                format!("uncovered {rule}: no case makes it flag anything")
            } else {
                let files = entry.files.join(", ");
                format!("uncovered {rule} (named by {files}): no case makes it flag anything")
            }
        }));
        lines.push(self.count());
        Document::new().verbatim(lines.join("\n"))
    }

    fn exit_code(&self) -> u8 {
        if !self.failures.is_empty() || self.results.iter().any(|result| result.error.is_some()) {
            2
        } else {
            u8::from(self.failed > 0 || !self.uncovered.is_empty())
        }
    }
}
