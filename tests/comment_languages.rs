//! The comments of Go, Swift, YAML, TOML and Shell files, and the directives
//! and doc links of every source language, against Vale.
//! The alerts are those of an `existence` rule that matches every word, over
//! files that hold one kind of comment each.
//!
//! `tests/fixtures/comment-languages/rules` holds the rule, `inputs` the files
//! with `.fixture` after the extension, and `vale.tsv` one row for each alert
//! Vale gave: the rule, the file, the line, the first and last column, and the
//! matched text. The rows come from Vale at `66bbb85` with the grammars of its
//! `grammars/build.sh`.
//! Vale ran on the inputs with the rule as one style, a `View`
//! that reads the comments of YAML and TOML files as Markdown, and `[formats]`
//! that reads the comments of the other files as Markdown.
//!
//! hakkiri gives Vale's alerts except in the cases of the
//! `comment-languages` change. `departures.tsv` holds one row for each alert
//! that only one side gave, led by `vale` or `hakkiri`. The test fails on any
//! difference that file does not list, and on any row of it that no longer
//! differs.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::collections::BTreeSet;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{fs, slice};

use hakkiri::engine::lint;
use hakkiri::rule::Rule;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// A rule, a file, a line, the first and last column, and the matched text.
type Row = (String, String, usize, usize, usize, String);

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

fn tsv(row: &Row) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        row.0, row.1, row.2, row.3, row.4, row.5
    )
}

fn name_of(path: &Path) -> Result<&str, Box<dyn Error>> {
    Ok(path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("a file has no name")?)
}

#[test]
fn comments_read_as_vale_reads_them_but_where_decided() -> TestResult {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/comment-languages");
    let mut vale: BTreeSet<Row> = BTreeSet::new();
    for line in fs::read_to_string(root.join("vale.tsv"))?.lines() {
        vale.insert(row(&line.split('\t').collect::<Vec<&str>>())?);
    }
    let text = fs::read_to_string(root.join("rules/Words.yml"))?;
    let rule = Rule::from_text("Comments.Words", "Words.yml", &text)?;
    let mut inputs: Vec<PathBuf> = fs::read_dir(root.join("inputs"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    inputs.sort();
    let mut hakkiri: BTreeSet<Row> = BTreeSet::new();
    for input in &inputs {
        let file = name_of(input)?.trim_end_matches(".fixture");
        let language = Language::from_path(Path::new(file))
            .ok_or_else(|| format!("no reader reads {file}"))?;
        let source = fs::read_to_string(input)?;
        let tree = parse(language, &source)?;
        for alert in lint(
            slice::from_ref(&rule),
            &tree,
            &source,
            language.langdex().as_slice(),
        )? {
            hakkiri.insert((
                "Comments.Words".to_owned(),
                file.to_owned(),
                alert.line,
                alert.first_column,
                alert.last_column,
                alert.matched,
            ));
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

/// The line and matched text of each alert of the rule `text` over `source`,
/// read as `language`.
fn alerts(
    language: Language,
    text: &str,
    source: &str,
) -> Result<Vec<(usize, String)>, Box<dyn Error>> {
    let rule = Rule::from_text("Comments.Probe", "Probe.yml", text)?;
    let tree = parse(language, source)?;
    Ok(lint(
        slice::from_ref(&rule),
        &tree,
        source,
        language.langdex().as_slice(),
    )?
    .into_iter()
    .map(|alert| (alert.line, alert.matched))
    .collect())
}

/// A rule that flags `marker` in the scope `scope`.
fn marker_rule(scope: &str) -> String {
    format!("extends: existence\nmessage: \"found '%s'\"\nscope: {scope}\ntokens:\n  - marker\n")
}

#[test]
fn a_swift_doc_comment_has_the_doc_scope() -> TestResult {
    let found = alerts(
        Language::Swift,
        &marker_rule("text.comment.doc"),
        "// A marker here.\n/// A marker there.\n",
    )?;
    assert_eq!(found, [(2, "marker".to_owned())]);
    Ok(())
}

#[test]
fn a_hash_comment_is_a_line_comment() -> TestResult {
    let found = alerts(
        Language::Yaml,
        &marker_rule("text.comment.line"),
        "a: marker # A marker here.\n",
    )?;
    assert_eq!(found, [(1, "marker".to_owned())]);
    Ok(())
}

#[test]
fn a_go_link_with_a_target_is_prose() -> TestResult {
    let words = "extends: existence\nmessage: \"found '%s'\"\nnonword: true\nscope: text\ntokens:\n  - '[A-Za-z]+'\n";
    let found = alerts(
        Language::Go,
        words,
        "package x\n\n// Run starts it, see [Stop].\nfunc Run() {}\n\n// See [Stop](https://example.com) now.\nfunc Other() {}\n",
    )?;
    let matched: Vec<&str> = found.iter().map(|(_, word)| word.as_str()).collect();
    assert_eq!(matched, ["starts", "it", "see", "See", "Stop", "now"]);
    Ok(())
}

#[test]
fn a_go_comment_apart_from_its_declaration_keeps_the_name() -> TestResult {
    let words = "extends: existence\nmessage: \"found '%s'\"\nnonword: true\nscope: text\ntokens:\n  - '[A-Za-z]+'\n";
    let found = alerts(
        Language::Go,
        words,
        "package x\n\n// Run starts.\n\nfunc Run() {}\n\nvar a = 1 // b counts.\nvar b = 2\n",
    )?;
    let matched: Vec<&str> = found.iter().map(|(_, word)| word.as_str()).collect();
    assert_eq!(matched, ["Run", "starts", "b", "counts"]);
    Ok(())
}

/// A rule that flags each bracketed word, which Markdown reads as text when no
/// definition makes it a link.
const BRACKETS: &str = "extends: existence\nmessage: \"found '%s'\"\nnonword: true\nscope: text\ntokens:\n  - '\\[[^\\]]*\\]'\n";

#[test]
fn a_go_doc_link_in_a_fence_is_masked_as_in_a_go_file() -> TestResult {
    let source =
        "See [Name] in prose.\n\n```go\n// Run starts it, see [Name].\nfunc Run() {}\n```\n";
    let found = alerts(Language::Markdown, BRACKETS, source)?;
    let lines: Vec<usize> = found.iter().map(|(line, _)| *line).collect();
    assert_eq!(lines, [1], "{found:?}");
    Ok(())
}

#[test]
fn a_rust_doc_link_in_a_fence_is_masked_as_in_a_rust_file() -> TestResult {
    let source =
        "See [`Name`] in prose.\n\n```rust\n/// Runs it, see [`Name`].\nfn run() {}\n```\n";
    let found = alerts(Language::Markdown, BRACKETS, source)?;
    let lines: Vec<usize> = found.iter().map(|(line, _)| *line).collect();
    assert_eq!(lines, [1], "{found:?}");
    Ok(())
}
