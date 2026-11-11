//! Tests for the case check: a comparison with Vale.
//!
//! `tests/fixtures/capitalization/vale.tsv` holds a configuration name, a text,
//! the text as the rule's converter writes it, and `1` when Vale passes the
//! text or `0` when not. The rows come from Vale at `66bbb85` run in Go: its
//! `title` and `sentence` functions, with the converters of prose `b33e60d`
//! and the exceptions as Vale's `updateExceptions` leaves them. The texts are
//! hand-written headings and every 40th short sentence of "The Adventures of
//! Sherlock Holmes", a public domain text, as it is, with each word
//! capitalized, and in lower case.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test reads its rules with `?` and fails through an assertion."
)]

use std::collections::HashMap;
use std::error::Error;

use super::evaluate;
use crate::rule::{Capitalization, Check, Rule};

type TestResult = Result<(), Box<dyn Error>>;

/// The extra lines of the rule file of a configuration of the fixture.
fn lines(name: &str) -> Result<&'static str, String> {
    Ok(match name {
        "title-ap" => "match: $title\nstyle: AP\n",
        "title-chicago" => "match: $title\nstyle: Chicago\n",
        "title-ap-exceptions" => "match: $title\nexceptions: [Hakkiri, API, the web, 'Py\\w+']\n",
        "title-ap-prefix" => "match: $title\nprefix: '^\\d+\\. '\n",
        "sentence" => "match: $sentence\n",
        "sentence-colon" => "match: $sentence\nindicators: [':']\n",
        "sentence-exceptions" => {
            "match: $sentence\nindicators: [':']\nexceptions: [Hakkiri, API, GitHub, 'Py\\w+']\n"
        }
        "sentence-threshold" => "match: $sentence\nindicators: [':']\nthreshold: 0.7\n",
        "sentence-prefix" => "match: $sentence\nprefix: '^[a-z]\\. '\n",
        other => return Err(format!("the fixture names a configuration `{other}`")),
    })
}

/// The check of a configuration.
fn check(name: &str) -> Result<Capitalization, Box<dyn Error>> {
    let text = format!(
        "extends: capitalization\nmessage: m\nscope: heading\n{}",
        lines(name)?
    );
    let Check::Capitalization(check) = Rule::from_text("Team.Case", "Team/Case.yml", &text)?.check
    else {
        return Err("the rule is not a capitalization rule".into());
    };
    Ok(check)
}

#[test]
fn converts_and_counts_as_vale_does() -> TestResult {
    let fixture = include_str!("../../tests/fixtures/capitalization/vale.tsv");
    let mut checks: HashMap<&str, Capitalization> = HashMap::new();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for line in fixture.lines() {
        let mut columns = line.split('\t');
        let (Some(name), Some(text), Some(expected), Some(passes), None) = (
            columns.next(),
            columns.next(),
            columns.next(),
            columns.next(),
            columns.next(),
        ) else {
            return Err(format!("a line has not four columns: {line}").into());
        };
        if !checks.contains_key(name) {
            checks.insert(name, check(name)?);
        }
        let Some(rule) = checks.get(name) else {
            continue;
        };
        let (made, matched) = evaluate(rule, text);
        if made != expected || matched != (passes == "1") {
            wrong.push(format!("{name}: {text:?} gave {made:?}, {matched}"));
        }
        checked += 1;
    }
    assert!(checked > 1_500, "{checked} rows");
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
