//! The Markdown reader against Vale's converter: the alerts of one style of
//! rules over a corpus of inputs, each holding one construct.
//!
//! `tests/fixtures/markdown-corpus` holds the package `styles` and the inputs
//! under `text`, with `.fixture` after `.md`.
//! The style `Probe` has one rule for each scope that a construct reaches
//! (`text`, `paragraph`, `heading`, `list`, `blockquote`, `table.cell`,
//! `link`, `strong`, `emphasis` and `code`) and `Probe.Word`, which matches
//! each run of non-space characters that holds a letter or a digit, so that
//! where a construct joins or splits words shows.
//! Each input uses words no other line of its file uses, since Vale places a
//! repeated word at another copy of it.
//! `vale.tsv` holds one row for each alert Vale gave: the file, the line, the
//! first column, the rule and the matched text, with a line feed written `\n`,
//! a tab `\t` and a backslash `\\`.
//! The rows come from Vale at `66bbb85` run from the fixture folder as `vale
//! --config=vale/probe.ini --output=JSON <file>`, one run for each input.
//!
//! `departures.tsv` holds one row for each alert that only one side gave, led
//! by `vale` or `hakkiri`.
//! The test fails on any difference that file does not list, and on any row
//! of it that no longer differs.
//! `panics/vale-panic.md.fixture` is an input that makes Vale 66bbb85 panic,
//! so it has no row.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

mod common;

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use common::run;
use ctl_core::fixture::{Root, Suite};
use serde_json::Value;

type TestResult = Result<(), Box<dyn Error>>;

/// A file, a line, the first column, the rule and the matched text.
type Row = (String, usize, usize, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/markdown-corpus")
}

/// Copies `from` into `root` at `to`, dropping `.fixture` from each name.
fn copy_into(root: &Root<'_>, from: &Path, to: &Path) -> TestResult {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let target = to.join(name.strip_suffix(".fixture").unwrap_or(&name));
        if entry.file_type()?.is_dir() {
            copy_into(root, &entry.path(), &target)?;
        } else {
            root.file(target, fs::read(entry.path())?);
        }
    }
    Ok(())
}

/// A scenario folder that holds the fixture's package, inputs and config.
fn project<'a>(suite: &'a Suite, name: &str) -> Result<Root<'a>, Box<dyn Error>> {
    let root = suite.scenario(name);
    for folder in ["styles", "text", "panics", ".ctl"] {
        copy_into(&root, &fixture().join(folder), Path::new(folder))?;
    }
    Ok(root)
}

/// The text of a cell, with `\n`, `\t` and `\\` read as the characters they
/// stand for.
fn unescape(cell: &str) -> String {
    let mut text = String::with_capacity(cell.len());
    let mut chars = cell.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            text.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => text.push('\n'),
            Some('t') => text.push('\t'),
            Some(other) => text.push(other),
            None => text.push('\\'),
        }
    }
    text
}

fn row_of(cells: &[&str]) -> Result<Row, Box<dyn Error>> {
    let [file, line, first, rule, matched] = cells else {
        return Err(format!("a row has {} cells: {cells:?}", cells.len()).into());
    };
    Ok((
        (*file).to_owned(),
        line.parse()?,
        first.parse()?,
        (*rule).to_owned(),
        unescape(matched),
    ))
}

/// The rows of one file of the fixture, one for each line.
fn table(file: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    Ok(fs::read_to_string(fixture().join(file))?
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect())
}

fn rows(lines: &[Vec<String>]) -> Result<BTreeSet<Row>, Box<dyn Error>> {
    lines
        .iter()
        .map(|cells| row_of(&cells.iter().map(String::as_str).collect::<Vec<_>>()))
        .collect()
}

/// The rows of a `check --format json` run.
fn rows_of(out: &str) -> Result<BTreeSet<Row>, Box<dyn Error>> {
    let json: Value = serde_json::from_str(out)?;
    let mut rows = BTreeSet::new();
    for alert in json["alerts"].as_array().ok_or("no alerts array")? {
        let text = |key: &str| alert[key].as_str().unwrap_or_default().to_owned();
        let number = |key: &str| {
            alert[key]
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
                .unwrap_or_default()
        };
        rows.insert((
            text("path"),
            number("line"),
            number("column"),
            text("rule"),
            text("match"),
        ));
    }
    Ok(rows)
}

#[test]
fn every_input_gives_the_alerts_vale_gives() -> TestResult {
    let suite = Suite::new("markdown-corpus");
    let root = project(&suite, "parity")?;
    let result = run(
        root.path(),
        &[
            "check",
            "--config",
            ".ctl/probe.yaml",
            "--format",
            "json",
            "text",
        ],
        &[],
    )?;
    assert!(
        result.code == 0 || result.code == 1,
        "{}{}",
        result.out,
        result.err
    );
    let ours = rows_of(&result.out)?;
    let theirs = rows(&table("vale.tsv")?)?;
    let listed = |side: &str| -> Result<BTreeSet<Row>, Box<dyn Error>> {
        table("departures.tsv")?
            .iter()
            .filter(|cells| cells.first().is_some_and(|lead| lead == side))
            .map(|cells| row_of(&cells[1..].iter().map(String::as_str).collect::<Vec<_>>()))
            .collect()
    };
    let (only_vale, only_ours) = (listed("vale")?, listed("hakkiri")?);
    let got_vale: BTreeSet<Row> = theirs.difference(&ours).cloned().collect();
    let got_ours: BTreeSet<Row> = ours.difference(&theirs).cloned().collect();
    assert!(
        got_vale == only_vale && got_ours == only_ours,
        "the differences from Vale changed.\nonly Vale gave, not listed: {:#?}\nlisted, but Vale gave it with hakkiri: {:#?}\nonly hakkiri gave, not listed: {:#?}\nlisted, but hakkiri gave it with Vale: {:#?}",
        got_vale.difference(&only_vale).collect::<Vec<_>>(),
        only_vale.difference(&got_vale).collect::<Vec<_>>(),
        got_ours.difference(&only_ours).collect::<Vec<_>>(),
        only_ours.difference(&got_ours).collect::<Vec<_>>(),
    );
    Ok(())
}

#[test]
fn a_run_finishes_where_vale_panics() -> TestResult {
    let suite = Suite::new("markdown-corpus-panic");
    let root = project(&suite, "panic")?;
    let result = run(
        root.path(),
        &[
            "check",
            "--config",
            ".ctl/probe.yaml",
            "--format",
            "json",
            "panics",
        ],
        &[],
    )?;
    assert!(
        result.code == 0 || result.code == 1,
        "{}{}",
        result.out,
        result.err
    );
    assert!(!result.lines().is_empty(), "the run read no alert");
    Ok(())
}
