//! The `spelling` check and the `affix` field against Vale: the alerts of one
//! style of rules over Markdown inputs.
//!
//! `tests/fixtures/spelling/Parity` holds the `spelling` rules and
//! `tests/fixtures/spelling/Affix` the `existence` and `substitution` rules
//! with `affix`, in the package under `tests/fixtures/spelling`.
//! `tests/fixtures/spelling-parity/inputs` and `affix-parity/inputs` hold the
//! Markdown files of each, with `.fixture` after `.md`.
//! `vale.tsv` beside each holds one row for each alert Vale gave: the rule, the
//! file, the line, the first and last column, the matched text and the
//! message.
//! The rows come from Vale at `66bbb85` run on the inputs with the package as
//! its styles path.
//!
//! `departures.tsv` holds one row for each alert that only one side gave,
//! led by `vale` or `hakkiri`.
//! The test fails on any difference that file does not list, and on any row
//! of it that no longer differs.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::collections::BTreeSet;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{fs, slice};

use hakkiri::engine::lint;
use hakkiri::rule::load_style;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// A rule, a file, a line, the first and last column, the matched text and
/// the message.
type Row = (String, String, usize, usize, usize, String, String);

fn row(cells: &[&str]) -> Result<Row, Box<dyn Error>> {
    let [rule, file, line, first, last, matched, message] = cells else {
        return Err(format!("a row has not seven columns: {cells:?}").into());
    };
    Ok((
        (*rule).to_owned(),
        (*file).to_owned(),
        line.parse()?,
        first.parse()?,
        last.parse()?,
        (*matched).to_owned(),
        (*message).to_owned(),
    ))
}

fn tsv(row: &Row) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
        row.0, row.1, row.2, row.3, row.4, row.5, row.6
    )
}

fn files_in(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files: Vec<_> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    files.sort();
    Ok(files)
}

/// Holds the alerts of the style `style` over the inputs of `root` to Vale's.
fn compare(style: &str, root: &str) -> TestResult {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let root = fixtures.join(root);
    let mut vale: BTreeSet<Row> = BTreeSet::new();
    for line in fs::read_to_string(root.join("vale.tsv"))?.lines() {
        vale.insert(row(&line.split('\t').collect::<Vec<&str>>())?);
    }
    let rules = load_style(&fixtures.join("spelling").join(style))?;
    let mut hakkiri: BTreeSet<Row> = BTreeSet::new();
    for input in files_in(&root.join("inputs"))? {
        let source = fs::read_to_string(&input)?;
        let tree = parse(Language::Markdown, &source)?;
        let file = input
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("an input has no name")?
            .trim_end_matches(".fixture");
        for rule in &rules {
            for alert in lint(
                slice::from_ref(rule),
                &tree,
                &source,
                tree.language.langdex().as_slice(),
            )? {
                hakkiri.insert((
                    alert.rule,
                    file.to_owned(),
                    alert.line,
                    alert.first_column,
                    alert.last_column,
                    alert.matched,
                    alert.message,
                ));
            }
        }
    }
    let mut found = Vec::new();
    for only in vale.difference(&hakkiri) {
        found.push(format!("vale\t{}", tsv(only)));
    }
    for only in hakkiri.difference(&vale) {
        found.push(format!("hakkiri\t{}", tsv(only)));
    }
    let listed: Vec<String> = fs::read_to_string(root.join("departures.tsv"))?
        .lines()
        .map(str::to_owned)
        .collect();
    assert!(
        found == listed,
        "the differences from Vale changed; the list now reads:\n{}",
        found.join("\n")
    );
    Ok(())
}

#[test]
fn spelling_reports_as_vale_does_but_where_decided() -> TestResult {
    compare("Parity", "spelling-parity")
}

#[test]
fn affix_expands_as_vale_does_but_where_decided() -> TestResult {
    compare("Affix", "affix-parity")
}
