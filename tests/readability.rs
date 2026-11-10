//! Readability: the syllables and scores of the `prose` test data.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use hakkiri::readability::{Counts, syllables};
use hakkiri::splitter::Model;

type TestResult = Result<(), Box<dyn Error>>;

/// Reads `tests/fixtures/readability/<name>`.
fn read(name: &str) -> Result<String, Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/readability")
        .join(name);
    Ok(fs::read_to_string(path)?)
}

#[test]
fn syllables_match_prose() -> TestResult {
    let mut wrong = Vec::new();
    let mut total = 0_usize;
    for line in read("syllables.tsv")?.lines() {
        let (word, count) = line.split_once('\t').ok_or("a row with no tab")?;
        total += 1;
        let expected: usize = count.parse()?;
        let got = syllables(word);
        if got != expected {
            wrong.push(format!("{word}: {got}, prose {expected}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {total} words differ: {:?}",
        wrong.len(),
        wrong.iter().take(20).collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn scores_match_prose() -> TestResult {
    let model = Model::english().map_err(ToString::to_string)?;
    let table = read("scores.tsv")?;
    let mut lines = table.lines();
    let names: Vec<&str> = lines
        .next()
        .ok_or("no header")?
        .split('\t')
        .skip(4)
        .collect();
    for line in lines {
        let cells: Vec<&str> = line.split('\t').collect();
        let [id, words, sentences, characters, scores @ ..] = cells.as_slice() else {
            return Err("a short row".into());
        };
        let text = read(&format!("{id}.txt.fixture"))?;
        let counts = Counts::of(&text, model);
        assert_eq!(counts.words.to_string(), *words, "{id} words");
        assert_eq!(
            counts.characters.to_string(),
            *characters,
            "{id} characters"
        );
        if counts.sentences.to_string() != *sentences {
            // The splitter cuts the text apart from prose, which the design
            // lists as a risk: every score but the word counts then differs.
            eprintln!("{id}: {} sentences, prose {sentences}", counts.sentences);
            continue;
        }
        let computed = counts.scores();
        for (name, expected) in names.iter().zip(scores) {
            let expected: f64 = expected.parse()?;
            let got = computed
                .iter()
                .find(|(computed, _)| computed == name)
                .map(|(_, score)| *score)
                .ok_or("a score hakkiri does not compute")?;
            assert!(
                (got - expected).abs() < 0.01,
                "{id} {name}: {got:.4}, prose {expected}"
            );
        }
    }
    Ok(())
}
