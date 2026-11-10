//! Flesch-Kincaid parity: hakkiri's counts and score against Vale 3.26.0's,
//! for the case inputs of `ProseStyle` that hold no code span.
//!
//! The fixture is `tests/fixtures/fk-parity/vale-fk.tsv`. Where hakkiri's
//! counts equal Vale's, the score is within 0.01 of Vale's, and on every input
//! it is within 0.5. The test prints each input whose counts differ.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use hakkiri::engine::lint;
use hakkiri::rule::Rule;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// A rule that prints one value: the formula, with the score in millionths
/// so that two decimals keep six.
fn value_rule(name: &str, formula: &str) -> Result<Rule, Box<dyn Error>> {
    Ok(Rule::from_text(
        name,
        "Metrics/Value.yml",
        &format!(
            "extends: metric\nmessage: \"%s\"\nformula: {formula}\ncondition: '> -1000000000'\n"
        ),
    )?)
}

/// The four values of `source`, read as Markdown: score, words, sentences and
/// syllables.
fn measure(source: &str) -> Result<[f64; 4], Box<dyn Error>> {
    let rules = [
        value_rule("Metrics.Score", "flesch_kincaid * 1000000")?,
        value_rule("Metrics.Words", "words")?,
        value_rule("Metrics.Sentences", "sentences")?,
        value_rule("Metrics.Syllables", "syllables")?,
    ];
    let tree = parse(Language::Markdown, source)?;
    let alerts = lint(&rules, &tree, source, "md")?;
    let mut values = [0.0; 4];
    for (slot, rule) in values.iter_mut().zip(&rules) {
        let alert = alerts
            .iter()
            .find(|alert| alert.rule == rule.name)
            .ok_or_else(|| format!("no value for {}", rule.name))?;
        *slot = alert.message.parse()?;
    }
    values[0] /= 1_000_000.0;
    Ok(values)
}

#[test]
fn flesch_kincaid_matches_vale() -> TestResult {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fk-parity");
    let table = fs::read_to_string(root.join("vale-fk.tsv"))?;
    let mut compared = 0;
    let mut differing = Vec::new();
    let mut failures = Vec::new();
    for row in table.lines().skip(1) {
        let cells: Vec<&str> = row.split('\t').collect();
        let [id, _, _, format, status, score, words, sentences, syllables] = cells.as_slice()
        else {
            return Err(format!("a short row: {row}").into());
        };
        if *status != "measured" {
            continue;
        }
        let source = fs::read_to_string(root.join(format!("inputs/{id}.{format}.fixture")))?;
        let [got_score, got_words, got_sentences, got_syllables] = measure(&source)?;
        let vale = [words, sentences, syllables].map(|cell| cell.parse::<f64>());
        let [Ok(vale_words), Ok(vale_sentences), Ok(vale_syllables)] = vale else {
            return Err(format!("a bad count in {row}").into());
        };
        let vale_score: f64 = score.parse()?;
        let same_counts = (got_words - vale_words).abs() < 0.5
            && (got_sentences - vale_sentences).abs() < 0.5
            && (got_syllables - vale_syllables).abs() < 0.5;
        let tolerance = if same_counts { 0.01 } else { 0.5 };
        if !same_counts {
            differing.push(format!(
                "{id}: words {got_words} (Vale {vale_words}), sentences {got_sentences} \
                 (Vale {vale_sentences}), syllables {got_syllables} (Vale {vale_syllables})"
            ));
        }
        if (got_score - vale_score).abs() > tolerance {
            failures.push(format!(
                "{id}: {got_score:.4}, Vale {vale_score:.4}, tolerance {tolerance}"
            ));
        }
        compared += 1;
    }
    eprintln!(
        "{compared} inputs compared; counts differ on {}",
        differing.len()
    );
    for line in &differing {
        eprintln!("  {line}");
    }
    assert_eq!(compared, 51);
    assert!(failures.is_empty(), "{failures:#?}");
    Ok(())
}
