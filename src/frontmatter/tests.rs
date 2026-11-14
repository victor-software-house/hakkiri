//! Tests for the string values of front matter: which entries are text, and
//! where the text lies in the file.
//!
//! `tests/fixtures/front-matter/cases` holds one file for each case, and
//! `cases_report_their_block_values_fault_and_body` snapshots what each gives.

use std::fmt::Write as _;
use std::fs;

use super::{blank, fault, leading, span};
use crate::markdown::{Flavor, build};
use crate::tree::{Node, Scope};

fn read<'a>(found: impl IntoIterator<Item = &'a Node>, source: &str) -> Vec<(String, String)> {
    found
        .into_iter()
        .map(|node| {
            (
                node.label.clone().unwrap_or_default(),
                node.text(source).unwrap_or_default(),
            )
        })
        .collect()
}

fn pair(key: &str, text: &str) -> (String, String) {
    (key.to_owned(), text.to_owned())
}

/// The line of the file that holds byte `at`, counted from 1.
fn line_of(source: &str, at: usize) -> usize {
    source
        .get(..at)
        .map_or(0, |before| before.matches('\n').count())
        + 1
}

/// What the front matter of `source` gives: the lines of its block, the label
/// and text of each value, the fault of the block, and the line where the body
/// starts.
fn report(source: &str) -> String {
    let mut report = String::new();
    match span(source) {
        Some(block) => {
            let first = line_of(source, block.start);
            let last = line_of(source, block.end.saturating_sub(1));
            writeln!(report, "block: lines {first} to {last}").unwrap();
        }
        None => writeln!(report, "block: none").unwrap(),
    }
    let values = read(&leading(source), source);
    if values.is_empty() {
        writeln!(report, "values: none").unwrap();
    } else {
        writeln!(report, "values:").unwrap();
        for (label, text) in values {
            writeln!(report, "  {label}: {text:?}").unwrap();
        }
    }
    writeln!(
        report,
        "fault: {}",
        fault(source).as_deref().unwrap_or("none")
    )
    .unwrap();
    let built = build(source, Flavor::Markdown).unwrap();
    match built
        .nodes
        .iter()
        .find(|node| node.scope != Scope::FrontMatter)
    {
        Some(node) => writeln!(report, "body: line {}", line_of(source, node.span.start)).unwrap(),
        None => writeln!(report, "body: none").unwrap(),
    }
    report
}

#[test]
fn cases_report_their_block_values_fault_and_body() {
    insta::glob!(
        "../../tests/fixtures/front-matter/cases",
        "*.md.fixture",
        |path| {
            let source = fs::read_to_string(path).unwrap();
            insta::assert_snapshot!(report(&source));
        }
    );
}

#[test]
fn an_opening_fence_may_carry_blanks_after_it() {
    let source = "--- \t\ntitle: A very title\n---\n\nBody.\n";
    assert_eq!(
        read(&leading(source), source),
        [pair("title", "A very title")]
    );
    let toml = "+++  \ntitle = \"A very title\"\n+++\n";
    assert_eq!(read(&leading(toml), toml), [pair("title", "A very title")]);
}

#[test]
fn a_file_whose_fence_carries_blanks_has_its_values_in_the_tree() {
    for source in [
        "--- \t\ntitle: A very title\n---\n\nBody.\n",
        "+++ \ntitle = \"A very title\"\n+++\n\nBody.\n",
    ] {
        let built = build(source, Flavor::Markdown).unwrap();
        let values = built.nodes.iter().filter(|node| node.label.is_some());
        assert_eq!(read(values, source), [pair("title", "A very title")]);
    }
}

#[test]
fn blanking_keeps_each_line_and_each_byte_offset() {
    let source = "---\r\ntitle: x\u{e9}y\r\n---\r\nBody \u{e9}.\n";
    let Some(block) = span(source) else {
        panic!("{source:?} has no block");
    };
    let expected = format!(
        "{}\n{}\n{}\nBody \u{e9}.\n",
        " ".repeat(4),
        " ".repeat(12),
        " ".repeat(4)
    );
    assert_eq!(blank(source, &block), expected);
}
