//! Places where Vale splits a sentence that hakkiri keeps whole, or keeps whole
//! what hakkiri splits: the scenarios of the `splitter-code-spans` change.
//!
//! `tests/fixtures/sentence-differences` holds one rule, one file for each
//! case (`text`, with `.fixture` after the extension), and a config for each
//! tool. The rule marks the first character of each sentence.
//! `vale.tsv` holds the alerts Vale gave at `66bbb85`, run as
//! `vale --config=vale/all.ini --output=JSON text`.
//! A row holds the config, the file, the line, the first and last column, the
//! level, the rule, the message and the matched text.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

#[expect(
    dead_code,
    reason = "This test reads the JSON report whole and never its lines."
)]
mod common;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use common::run;
use ctl_core::fixture::Suite;
use serde_json::Value;

type TestResult = Result<(), Box<dyn Error>>;

/// The line and the first column of each sentence start, by file.
type Starts = BTreeMap<String, Vec<(usize, usize)>>;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sentence-differences")
}

fn vale() -> Result<Starts, Box<dyn Error>> {
    let mut found = Starts::new();
    for line in fs::read_to_string(fixture().join("vale.tsv"))?.lines() {
        let cells: Vec<&str> = line.split('\t').collect();
        let [_, file, line, column, ..] = cells[..] else {
            return Err(format!("a row of vale.tsv has {} cells", cells.len()).into());
        };
        found
            .entry(file.to_owned())
            .or_default()
            .push((line.parse()?, column.parse()?));
    }
    Ok(found)
}

fn ours(out: &str) -> Result<Starts, Box<dyn Error>> {
    let json: Value = serde_json::from_str(out)?;
    let mut found = Starts::new();
    for alert in json["alerts"].as_array().ok_or("no alerts array")? {
        let at = |key: &str| usize::try_from(alert[key].as_u64().unwrap_or_default());
        found
            .entry(alert["path"].as_str().unwrap_or_default().to_owned())
            .or_default()
            .push((at("line")?, at("column")?));
    }
    for starts in found.values_mut() {
        starts.sort_unstable();
    }
    Ok(found)
}

fn starts(rows: &[(&str, &[(usize, usize)])]) -> Starts {
    rows.iter()
        .map(|(file, spots)| ((*file).to_owned(), spots.to_vec()))
        .collect()
}

#[test]
fn vale_and_hakkiri_split_these_sentences_otherwise() -> TestResult {
    let suite = Suite::new("sentence-differences");
    let root = suite.scenario("all");
    for dir in ["styles/Team", "text", ".ctl"] {
        for entry in fs::read_dir(fixture().join(dir))? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            root.file(
                Path::new(dir).join(name.strip_suffix(".fixture").unwrap_or(&name)),
                fs::read(entry.path())?,
            );
        }
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
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    // Vale reads `1.` of a numbered heading or item as a sentence, keeps a
    // version, a decimal and a task number whole, and ends a sentence at `vs.`.
    assert_eq!(
        vale()?,
        starts(&[
            ("text/decimal.md", &[(1, 1)]),
            ("text/heading.md", &[(1, 4), (1, 7), (3, 1)]),
            ("text/numbered-item.md", &[(1, 7), (1, 10)]),
            ("text/task.md", &[(1, 1)]),
            ("text/version.md", &[(1, 1)]),
            ("text/versus-number.md", &[(1, 1), (1, 34)]),
            ("text/versus.md", &[(1, 1), (1, 15)]),
        ])
    );
    // hakkiri keeps a numbered heading and item whole, ends a sentence after a
    // version, a decimal and a task number, and goes on after `vs.`.
    assert_eq!(
        ours(&result.out)?,
        starts(&[
            ("text/decimal.md", &[(1, 1), (1, 26)]),
            ("text/heading.md", &[(1, 4), (3, 1)]),
            ("text/numbered-item.md", &[(1, 7)]),
            ("text/task.md", &[(1, 1), (1, 41)]),
            ("text/version.md", &[(1, 1), (1, 31)]),
            ("text/versus-number.md", &[(1, 1), (1, 34)]),
            ("text/versus.md", &[(1, 1)]),
        ])
    );
    Ok(())
}
