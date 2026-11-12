//! A place where hakkiri reads a file otherwise than Vale: the scenario of the
//! `block-token-ignores` change.
//!
//! `tests/fixtures/markup-differences` holds two rules, one file (`text`,
//! with `.fixture` after the extension) and one config for each tool.
//! `vale.tsv` holds the alerts Vale gave at `66bbb85`, with the Rust grammar
//! built from the same commit, run as `vale --config=vale/all.ini
//! --output=JSON text`.
//! A row holds the config, the file, the line, the first and last column, the
//! level, the rule, the message and the matched text.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

mod common;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use common::run;
use ctl_core::fixture::Suite;
use serde_json::Value;

type TestResult = Result<(), Box<dyn Error>>;

/// The file, the line, the rule and the matched text of an alert.
type Spot = (String, usize, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/markup-differences")
}

/// The file, the line, the rule and the matched text of each of Vale's alerts.
fn vale() -> Result<Vec<Spot>, Box<dyn Error>> {
    let mut rows = Vec::new();
    for line in fs::read_to_string(fixture().join("vale.tsv"))?.lines() {
        let cells: Vec<&str> = line.split('\t').collect();
        let [_, file, line, _, _, _, rule, _, matched] = cells[..] else {
            return Err(format!("a row of vale.tsv has {} cells", cells.len()).into());
        };
        rows.push((
            file.to_owned(),
            line.parse()?,
            rule.to_owned(),
            matched.to_owned(),
        ));
    }
    Ok(rows)
}

/// The same four cells for each alert of a `check --format json` run.
fn ours(out: &str) -> Result<Vec<Spot>, Box<dyn Error>> {
    let json: Value = serde_json::from_str(out)?;
    let mut rows = Vec::new();
    for alert in json["alerts"].as_array().ok_or("no alerts array")? {
        let text = |key: &str| alert[key].as_str().unwrap_or_default().to_owned();
        let line = usize::try_from(alert["line"].as_u64().unwrap_or_default())?;
        rows.push((text("path"), line, text("rule"), text("match")));
    }
    Ok(rows)
}

fn row(file: &str, line: usize, rule: &str, matched: &str) -> Spot {
    (file.to_owned(), line, rule.to_owned(), matched.to_owned())
}

#[test]
fn vale_and_hakkiri_read_the_sentences_of_a_comment_otherwise() -> TestResult {
    let suite = Suite::new("markup-differences");
    let root = suite.scenario("all");
    for (from, to) in [
        ("styles/Team", "styles/Team"),
        ("text", "text"),
        (".ctl", ".ctl"),
    ] {
        for entry in fs::read_dir(fixture().join(from))? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            root.file(
                Path::new(to).join(name.strip_suffix(".fixture").unwrap_or(&name)),
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
    assert!(!result.lines().is_empty(), "{}", result.out);
    // Vale runs no `sentence` rule on a comment.
    assert_eq!(vale()?, [row("text/m3.rs", 1, "Team.Words", "very"),]);
    // hakkiri runs the rule on the sentence of the comment.
    let mut found = ours(&result.out)?;
    found.sort();
    assert_eq!(
        found,
        [
            row("text/m3.rs", 1, "Team.Really", "really"),
            row("text/m3.rs", 1, "Team.Words", "very"),
        ]
    );
    Ok(())
}
