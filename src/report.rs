//! What a run gives back: one value that ctl-core prints as lines or as JSON.

use ctl_core::view::Outcome;
use ctl_core::{Document, Present};
use hakkiri::alert::Alert;
use hakkiri::rule::{Level, LoadError};
use serde::Serialize;

/// The result of a verb.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum Output {
    /// `hakkiri check`.
    Check(Check),
    /// `hakkiri instructions`.
    Instructions(Instructions),
}

impl Present for Output {
    fn present(&self) -> Document {
        match self {
            Self::Check(check) => check.present(),
            Self::Instructions(instructions) => instructions.present(),
        }
    }

    fn outcome(&self) -> Outcome {
        match self {
            Self::Check(check) => check.outcome(),
            Self::Instructions(instructions) => instructions.outcome(),
        }
    }

    fn exit_code(&self) -> u8 {
        match self {
            Self::Check(check) => check.exit_code(),
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
