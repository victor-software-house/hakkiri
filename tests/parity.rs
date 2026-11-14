//! Flesch-Kincaid parity: hakkiri's counts and score against Vale 3.26.0's,
//! for the case inputs of `ProseStyle` that hold no code span.
//!
//! The fixture is `tests/fixtures/fk-parity/vale-fk.tsv`. Where hakkiri's
//! counts equal Vale's, the score is within 0.01 of Vale's, and on every input
//! it is within 0.5. The test prints each input whose counts differ.
//!
//! The 10 Rust inputs are the one place hakkiri departs from Vale, by decision
//! 16 of the rule-engine design: Vale prints no value for them, and hakkiri
//! measures the prose of a code file's comments.

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
    let alerts = lint(&rules, &tree, source, tree.language.langdex().as_slice())?;
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

/// The Rust inputs where Vale prints no value and hakkiri gives one, because
/// each holds a comment whose prose it measures.
const RUST_WITH_COMMENTS: [&str; 6] = [
    "BareURLInComment-01",
    "BareWorkItemInComment-01",
    "package-05",
    "package-06",
    "package-12",
    "package-15",
];

/// The Rust inputs with no comment, only strings and code. Vale prints no
/// value, and hakkiri has no prose to measure either.
const RUST_WITHOUT_COMMENTS: [&str; 4] = [
    "BareURLInComment-07",
    "BareWorkItemInComment-03",
    "package-11",
    "package-14",
];

/// The word count that hakkiri gives for the Rust `source`, or `None` when it
/// gives no value.
fn rust_words(source: &str) -> Result<Option<f64>, Box<dyn Error>> {
    let rule = value_rule("Metrics.Words", "words")?;
    let tree = parse(Language::Rust, source)?;
    let alerts = lint(&[rule], &tree, source, tree.language.langdex().as_slice())?;
    alerts
        .first()
        .map(|alert| alert.message.parse::<f64>())
        .transpose()
        .map_err(Into::into)
}

#[test]
fn a_rust_input_vale_cannot_measure_is_measured_by_its_comments() -> TestResult {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fk-parity");
    let table = fs::read_to_string(root.join("vale-fk.tsv"))?;
    let mut rows: Vec<&str> = table
        .lines()
        .skip(1)
        .filter(|row| {
            let cells: Vec<&str> = row.split('\t').collect();
            cells.get(3) == Some(&"rs") && cells.get(4) == Some(&"no-words")
        })
        .filter_map(|row| row.split('\t').next())
        .collect();
    rows.sort_unstable();
    let mut expected: Vec<&str> = RUST_WITH_COMMENTS
        .iter()
        .chain(&RUST_WITHOUT_COMMENTS)
        .copied()
        .collect();
    expected.sort_unstable();
    assert_eq!(rows, expected, "the fixture's Rust no-words rows changed");
    for id in rows {
        let source = fs::read_to_string(root.join(format!("inputs/{id}.rs.fixture")))?;
        let words = rust_words(&source)?;
        eprintln!("{id}: hakkiri counts {words:?} words, Vale none");
        if RUST_WITH_COMMENTS.contains(&id) {
            assert!(words.is_some_and(|count| count > 0.0), "{id}: {words:?}");
        } else {
            assert_eq!(words, None, "{id}");
        }
    }
    Ok(())
}
