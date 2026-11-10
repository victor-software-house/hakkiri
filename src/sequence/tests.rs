//! Tests for the sequence walk: a comparison with Vale.
//!
//! `tests/fixtures/sequence/rules` holds one rule file for each shape of
//! token the check reads, and `inputs.txt` the texts to read: hand-written
//! headings and sentences, and every third line of "The Adventures of Sherlock
//! Holmes", a public domain text. `vale.tsv` holds a rule name, a text, the
//! text of one alert and its message, for each alert Vale gave, in order. The
//! rows come from Vale at `66bbb85` run in Go on each text as one sentence,
//! with prose `b33e60d` tagging it.
//!
//! Vale cuts each text again with prose's segmenter and tags each part apart,
//! where hakkiri tags the sentence its own splitter gave whole. `inputs.txt`
//! holds the texts that the segmenter reads as one sentence, so both tag the
//! same words.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test reads its rules with `?` and fails through an assertion."
)]

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use super::find;
use crate::alert::fill;
use crate::rule::{Check, Rule};

type TestResult = Result<(), Box<dyn Error>>;

/// The alerts of the rule file `text` on `sentence`, as text and message.
fn alerts(name: &str, text: &str, sentence: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let rule = Rule::from_text(name, &format!("{name}.yml"), text)?;
    let Check::Sequence(check) = &rule.check else {
        return Err(format!("{name} is not a sequence rule").into());
    };
    Ok(find(check, sentence)?
        .into_iter()
        .map(|matched| {
            let values: Vec<&str> = matched.words.iter().map(String::as_str).collect();
            (
                sentence.get(matched.range).unwrap_or_default().to_owned(),
                fill(&rule.message, &values),
            )
        })
        .collect())
}

#[test]
fn walks_the_tokens_as_vale_does() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sequence");
    let inputs = fs::read_to_string(root.join("inputs.txt"))?;
    let fixture = fs::read_to_string(root.join("vale.tsv"))?;
    let mut expected: HashMap<(&str, &str), Vec<(String, String)>> = HashMap::new();
    for line in fixture.lines() {
        let columns: Vec<&str> = line.split('\t').collect();
        let [name, text, matched, message] = columns.as_slice() else {
            return Err(format!("a line has not four columns: {line}").into());
        };
        expected
            .entry((name, text))
            .or_default()
            .push(((*matched).to_owned(), (*message).to_owned()));
    }
    let mut files: Vec<_> = fs::read_dir(root.join("rules"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    files.sort();
    let mut wrong = Vec::new();
    let mut with_alerts = 0;
    for file in &files {
        let stem = file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or("a rule file has no name")?;
        let name = format!("Seq.{stem}");
        let text = fs::read_to_string(file)?;
        for sentence in inputs.lines() {
            let want = expected
                .get(&(name.as_str(), sentence))
                .cloned()
                .unwrap_or_default();
            let got = alerts(&name, &text, sentence)?;
            with_alerts += usize::from(!want.is_empty());
            if got != want {
                wrong.push(format!("{name}: {sentence:?} gave {got:?}, Vale {want:?}"));
            }
        }
    }
    assert!(with_alerts > 150, "{with_alerts} texts with an alert");
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
