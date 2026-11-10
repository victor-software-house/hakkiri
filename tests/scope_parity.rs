//! Scope selection against Vale: the alerts of an `existence` and a
//! `sequence` rule for each scope, over Markdown that nests sentences in list
//! items, quotes, tables and headings.
//!
//! `tests/fixtures/scope/rules` holds the rules, `inputs` the Markdown files
//! with `.fixture` after `.md`, and `vale.tsv` one row for each alert Vale
//! gave: the rule, the file, the line, the first and last column, and the
//! matched text. The rows come from Vale at `66bbb85` run on the inputs with
//! the rules as one style.
//!
//! hakkiri gives Vale's alerts except in the cases of design decision 9 of the
//! `case-sequence-and-spelling-checks` change. `departures.tsv` holds one row
//! for each alert that only one side gave, led by `vale` or `hakkiri`. The
//! test fails on any difference that file does not list, and on any row of it
//! that no longer differs.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use hakkiri::engine::lint;
use hakkiri::rule::Rule;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// A rule, a file, a line, the first and last column, and the matched text.
type Row = (String, String, usize, usize, usize, String);

/// The rules hakkiri refuses: Vale reads `sentence.list` as `sentence & list`.
const REFUSED: [&str; 2] = ["existence-sentence-dot-list", "sequence-sentence-dot-list"];

fn row(cells: &[&str]) -> Result<Row, Box<dyn Error>> {
    let [rule, file, line, first, last, matched] = cells else {
        return Err(format!("a row has not six columns: {cells:?}").into());
    };
    Ok((
        (*rule).to_owned(),
        (*file).to_owned(),
        line.parse()?,
        first.parse()?,
        last.parse()?,
        (*matched).to_owned(),
    ))
}

fn files_in(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files: Vec<_> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    files.sort();
    Ok(files)
}

fn name_of(path: &Path) -> Result<&str, Box<dyn Error>> {
    Ok(path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("a file has no name")?)
}

#[test]
fn scopes_select_as_vale_does_but_where_decided() -> TestResult {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scope");
    let mut vale: BTreeSet<Row> = BTreeSet::new();
    for line in fs::read_to_string(root.join("vale.tsv"))?.lines() {
        vale.insert(row(&line.split('\t').collect::<Vec<&str>>())?);
    }
    let inputs = files_in(&root.join("inputs"))?;
    let mut hakkiri: BTreeSet<Row> = BTreeSet::new();
    let mut refused = Vec::new();
    for file in files_in(&root.join("rules"))? {
        let stem = file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or("a rule file has no name")?;
        let name = format!("Scope.{stem}");
        let text = fs::read_to_string(&file)?;
        let Ok(rule) = Rule::from_text(&name, name_of(&file)?, &text) else {
            refused.push(stem.to_owned());
            continue;
        };
        for input in &inputs {
            let source = fs::read_to_string(input)?;
            let tree = parse(Language::Markdown, &source)?;
            for alert in lint(std::slice::from_ref(&rule), &tree, &source, "md")? {
                hakkiri.insert((
                    name.clone(),
                    name_of(input)?.trim_end_matches(".fixture").to_owned(),
                    alert.line,
                    alert.first_column,
                    alert.last_column,
                    alert.matched,
                ));
            }
        }
    }
    assert_eq!(refused, REFUSED);
    let loaded = |rows: &BTreeSet<Row>| -> BTreeSet<Row> {
        rows.iter()
            .filter(|row| !REFUSED.iter().any(|stem| row.0 == format!("Scope.{stem}")))
            .cloned()
            .collect()
    };
    let mut found = Vec::new();
    for only in loaded(&vale).difference(&hakkiri) {
        found.push(format!("vale\t{}", tsv(only)));
    }
    for only in hakkiri.difference(&loaded(&vale)) {
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

fn tsv(row: &Row) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        row.0, row.1, row.2, row.3, row.4, row.5
    )
}
