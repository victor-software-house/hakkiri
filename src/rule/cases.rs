//! The test cases of a rule file and of a `*.test.yml` file.
//!
//! A case holds a document and what linting it must give. Both places read
//! the same fields, refuse a field a case does not define, and refuse a case
//! that asserts nothing, because that case passes whatever the rule does.

use std::collections::HashSet;

use ctl_core::input::Input;
use garde::Validate;
use serde::Deserialize;

use super::field::{Text, Texts, valid};
use super::{LoadError, at_tab};
use crate::alert::Alert;

/// One case, as a file writes it.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[garde(allow_unvalidated)]
#[garde(custom(asserts))]
pub(super) struct CaseFields {
    #[garde(custom(named))]
    name: Text,
    #[serde(default)]
    #[garde(custom(valid))]
    about: Text,
    #[serde(default)]
    #[garde(custom(valid))]
    input: Text,
    #[serde(default)]
    #[garde(custom(valid))]
    format: Text,
    #[serde(default)]
    #[garde(custom(valid))]
    rule: Text,
    #[serde(default)]
    #[garde(custom(wanted))]
    want: Option<Text>,
    #[serde(default)]
    #[garde(custom(valid))]
    contains: Texts,
    #[serde(default)]
    #[garde(custom(valid))]
    absent: Texts,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn named(value: &Text, ctx: &()) -> garde::Result {
    valid(value, ctx)?;
    if value.value().trim().is_empty() {
        Err(garde::Error::new("every case needs a name"))
    } else {
        Ok(())
    }
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    clippy::ref_option,
    reason = "garde calls a custom rule with a reference to the field and to its context."
)]
fn wanted(value: &Option<Text>, ctx: &()) -> garde::Result {
    value.as_ref().map_or(Ok(()), |text| valid(text, ctx))
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
fn asserts(case: &CaseFields, _: &()) -> garde::Result {
    if case.want.is_some() || !case.contains.values().is_empty() || !case.absent.values().is_empty()
    {
        Ok(())
    } else {
        Err(garde::Error::new(format!(
            "the case `{}` needs one of `want`, `contains` or `absent`, or it passes whatever the rule does",
            case.name.value()
        )))
    }
}

/// The names of a list of cases are each used once.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "garde calls a custom rule with a reference to its context."
)]
pub(super) fn unique(cases: &[CaseFields], _: &()) -> garde::Result {
    let mut seen = HashSet::new();
    match cases
        .iter()
        .map(|case| case.name.value())
        .find(|name| !seen.insert(*name))
    {
        Some(name) => Err(garde::Error::new(format!(
            "the name `{name}` is used by two cases: give each case its own name"
        ))),
        None => Ok(()),
    }
}

/// The cases of a `*.test.yml` file: a list at the top.
#[derive(Debug, Deserialize, Validate)]
#[serde(transparent)]
struct CaseList(#[garde(dive, custom(unique))] Vec<CaseFields>);

/// One test case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestCase {
    /// What the report calls the case. It is unique in its file.
    pub name: String,
    /// A note for the reader, never asserted on.
    pub about: String,
    /// The document to lint.
    pub input: String,
    /// The extension `input` is read as, without the dot.
    pub format: String,
    /// The rule file this case isolates itself to, written relative to its
    /// test file, or `None` for a case that runs through the project config.
    pub rule: Option<String>,
    /// The exact output expected, where an empty text means no alerts.
    pub want: Option<String>,
    /// Text the output must hold.
    pub contains: Vec<String>,
    /// Text the output must not hold.
    pub absent: Vec<String>,
}

impl From<&CaseFields> for TestCase {
    fn from(fields: &CaseFields) -> Self {
        let format = fields.format.value().trim_start_matches('.');
        Self {
            name: fields.name.value().to_owned(),
            about: fields.about.value().to_owned(),
            input: fields.input.value().to_owned(),
            format: if format.is_empty() { "md" } else { format }.to_owned(),
            rule: Some(fields.rule.value())
                .filter(|rule| !rule.is_empty())
                .map(str::to_owned),
            want: fields.want.as_ref().map(|want| want.value().to_owned()),
            contains: fields.contains.values().to_vec(),
            absent: fields.absent.values().to_vec(),
        }
    }
}

impl TestCase {
    /// Why `output` fails this case, or `None` when it passes.
    ///
    /// `output` is one line per alert, as [`alert_lines`] writes it.
    /// `want` equals it once both are trimmed, each `contains` text appears
    /// in it, and no `absent` text does.
    #[must_use]
    pub fn verdict(&self, output: &str) -> Option<String> {
        if let Some(want) = &self.want
            && output.trim() != want.trim()
        {
            return Some("output does not match".to_owned());
        }
        if let Some(missing) = self.contains.iter().find(|text| !output.contains(*text)) {
            return Some(format!("output does not contain {missing:?}"));
        }
        self.absent
            .iter()
            .find(|text| output.contains(*text))
            .map(|found| format!("output contains {found:?}"))
    }
}

/// The alerts of a case as it sees them: one line each,
/// `line:column:Rule:message`, in order of line and column.
#[must_use]
pub fn alert_lines(alerts: &[Alert]) -> String {
    let mut rows: Vec<&Alert> = alerts.iter().collect();
    rows.sort_by_key(|alert| (alert.line, alert.first_column));
    let lines: Vec<String> = rows
        .iter()
        .map(|alert| {
            format!(
                "{}:{}:{}:{}",
                alert.line, alert.first_column, alert.rule, alert.message
            )
        })
        .collect();
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

/// Reads the cases of the `*.test.yml` file `file`, whose text is `text`.
///
/// # Errors
///
/// Returns a [`LoadError`] when the text is not a list of cases.
/// It also refuses a case that holds a field it does not define or one of the
/// wrong kind, a case with no name, a case that asserts nothing, and two cases
/// that share a name. Each problem names its line.
pub fn cases_from_text(file: &str, text: &str) -> Result<Vec<TestCase>, LoadError> {
    let input = Input::new(file, text);
    let list: CaseList = input.load().map_err(|error| at_tab(error, text))?;
    Ok(list.0.iter().map(TestCase::from).collect())
}
