//! Math: the scenarios of the math-as-code change.
//!
//! `tests/fixtures/math` holds one package of five rules (`styles`), three
//! Markdown files and one MDX file (`text`, with `.fixture` after the
//! extension), and one config for each tool: `vale/all.ini` and
//! `.ctl/all.yaml`.
//! `vale.tsv` holds one row for each alert Vale gave on the files: the config,
//! the file, the line, the first and last column, the level, the rule, the
//! message and the matched text.
//! The rows come from Vale at `66bbb85`, run from the fixture folder as
//! `vale --config=vale/all.ini --output=JSON text`.

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

/// A file, a line, the first and last column, the level, the rule, the
/// message and the matched text.
type Row = (String, usize, usize, usize, String, String, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/math")
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

/// The rows Vale gave.
fn vale_rows() -> Result<BTreeSet<Row>, Box<dyn Error>> {
    let mut rows = BTreeSet::new();
    for line in fs::read_to_string(fixture().join("vale.tsv"))?.lines() {
        let cells: Vec<&str> = line.split('\t').collect();
        let [
            config,
            file,
            line,
            first,
            last,
            level,
            rule,
            message,
            matched,
        ] = cells[..]
        else {
            return Err(format!("a row of vale.tsv has {} cells", cells.len()).into());
        };
        if config == "all" {
            let matched = matched.replace("\\n", "\n");
            let last = if matched.contains('\n') {
                0
            } else {
                last.parse()?
            };
            rows.insert((
                file.to_owned(),
                line.parse()?,
                first.parse()?,
                last,
                level.to_owned(),
                rule.to_owned(),
                message.to_owned(),
                matched,
            ));
        }
    }
    Ok(rows)
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
        let matched = text("match");
        let last = if matched.contains('\n') {
            0
        } else {
            number("end_column")
        };
        rows.insert((
            text("path"),
            number("line"),
            number("column"),
            last,
            text("level"),
            text("rule"),
            text("message"),
            matched,
        ));
    }
    Ok(rows)
}

#[test]
fn every_file_gives_the_alerts_vale_gives() -> TestResult {
    let suite = Suite::new("math-parity");
    let root = suite.scenario("parity");
    for dir in ["styles", "text", ".ctl"] {
        copy_into(&root, &fixture().join(dir), Path::new(dir))?;
    }
    let result = run(
        root.path(),
        &[
            "check",
            "--config",
            ".ctl/all.yaml",
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
    assert!(!result.lines().is_empty(), "{}", result.out);
    let ours = rows_of(&result.out)?;
    let vale = vale_rows()?;
    let only_vale: Vec<_> = vale.difference(&ours).collect();
    let only_ours: Vec<_> = ours.difference(&vale).collect();
    assert!(
        only_vale.is_empty() && only_ours.is_empty(),
        "only Vale gave {only_vale:#?}, only hakkiri gave {only_ours:#?}"
    );
    Ok(())
}
