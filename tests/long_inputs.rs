//! Long inputs: the four check types of the pattern-and-readability-checks
//! change, `capitalization` and `sequence`, and `spelling` with `affix`, over
//! a line of 50,000 characters and a file of 60,000 lines.
//!
//! Each test lints one input with one set of rules:
//! `consistency`, `conditional`, `repetition` and `readability`; or
//! `capitalization` and `sequence`; or `spelling`, and an `existence` and a
//! `substitution` rule that read `affix`.
//! It checks the alerts each rule gives, and prints the time of the run.
//! One more test reads a single sentence of 50,000 characters with a
//! `sequence` rule, where the walk starts once for each word the anchor
//! accepts.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::time::{Duration, Instant};

use hakkiri::alert::Alert;
use hakkiri::engine::lint;
use hakkiri::rule::Rule;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// The four rules, as a name and a rule file.
const RULES: [(&str, &str); 4] = [
    (
        "Team.Consistency",
        "extends: consistency\nmessage: m\neither: {words: terms}\n",
    ),
    (
        "Team.Conditional",
        "extends: conditional\nmessage: m\nfirst: '\\bSome\\b'\nsecond: 'application programming interface'\n",
    ),
    (
        "Team.Repetition",
        "extends: repetition\nmessage: m\nignorecase: true\nalpha: true\ntokens: ['[^\\s.!?]+']\n",
    ),
    (
        "Team.Readability",
        "extends: readability\nmessage: m\nmetrics: [Flesch-Kincaid]\ngrade: -100\n",
    ),
];

/// How many alerts each rule gave, in the order of [`RULES`].
fn per_rule(alerts: &[Alert]) -> Vec<usize> {
    RULES
        .iter()
        .map(|(name, _)| alerts.iter().filter(|alert| alert.rule == *name).count())
        .collect()
}

/// The alerts of the four rules in `source`, and the time the lint took.
fn lint_four(source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rules = RULES
        .iter()
        .map(|(name, text)| Rule::from_text(name, "Team/Rule.yml", text))
        .collect::<Result<Vec<_>, _>>()?;
    let tree = parse(Language::Markdown, source)?;
    let start = Instant::now();
    let alerts = lint(&rules, &tree, source, "md")?;
    eprintln!("{} alerts in {:?}", alerts.len(), start.elapsed());
    Ok(alerts)
}

#[test]
fn a_line_of_fifty_thousand_characters() -> TestResult {
    let unit = "Some words are here. Some terms are the the there. ";
    let repeats = 50_000 / unit.len() + 1;
    let source = format!("{}\n", unit.repeat(repeats));
    assert!(source.len() >= 50_000);
    let alerts = lint_four(&source)?;
    assert_eq!(per_rule(&alerts), [1, 2 * repeats, repeats, 1]);
    Ok(())
}

#[test]
fn a_file_of_sixty_thousand_lines() -> TestResult {
    let source = "Some words are here.\nSome terms are the the there.\n".repeat(30_000);
    let alerts = lint_four(&source)?;
    assert_eq!(per_rule(&alerts), [1, 60_000, 30_000, 1]);
    Ok(())
}

/// The `capitalization` and `sequence` rules, as a name and a rule file.
const CASE_RULES: [(&str, &str); 2] = [
    (
        "Team.Capitalization",
        "extends: capitalization\nmessage: m\nmatch: $upper\n",
    ),
    (
        "Team.Sequence",
        "extends: sequence\nmessage: m\ntokens: [{pattern: the}, {pattern: the}]\n",
    ),
];

/// How many alerts `rule` gave.
fn count(alerts: &[Alert], rule: &str) -> usize {
    alerts.iter().filter(|alert| alert.rule == rule).count()
}

/// The alerts of `rules` in `source`, and the time the lint took.
fn lint_rules(
    rules: &[(&str, &str)],
    source: &str,
) -> Result<(Vec<Alert>, Duration), Box<dyn Error>> {
    let rules = rules
        .iter()
        .map(|(name, text)| Rule::from_text(name, "Team/Rule.yml", text))
        .collect::<Result<Vec<_>, _>>()?;
    let tree = parse(Language::Markdown, source)?;
    let start = Instant::now();
    let alerts = lint(&rules, &tree, source, "md")?;
    let took = start.elapsed();
    eprintln!("{} alerts in {took:?}", alerts.len());
    Ok((alerts, took))
}

#[test]
fn case_and_sequence_over_a_line_of_fifty_thousand_characters() -> TestResult {
    let unit = "Some words are here. Some terms are the the there. ";
    let repeats = 50_000 / unit.len() + 1;
    let source = format!("{}\n", unit.repeat(repeats));
    let (alerts, _) = lint_rules(&CASE_RULES, &source)?;
    assert_eq!(count(&alerts, "Team.Capitalization"), 1);
    assert_eq!(count(&alerts, "Team.Sequence"), repeats);
    Ok(())
}

#[test]
fn case_and_sequence_over_a_file_of_sixty_thousand_lines() -> TestResult {
    let source = "Some words are here.\nSome terms are the the there.\n".repeat(30_000);
    let (alerts, _) = lint_rules(&CASE_RULES, &source)?;
    assert_eq!(count(&alerts, "Team.Capitalization"), 1);
    assert_eq!(count(&alerts, "Team.Sequence"), 30_000);
    Ok(())
}

#[test]
fn a_sentence_of_fifty_thousand_characters() -> TestResult {
    let words = 50_000 / "go ".len() + 1;
    let source = format!("{}\n", "go ".repeat(words));
    let rules = [(
        "Team.Sequence",
        "extends: sequence\nmessage: m\ntokens: [{pattern: go}]\n",
    )];
    let (alerts, took) = lint_rules(&rules, &source)?;
    assert_eq!(count(&alerts, "Team.Sequence"), words);
    assert!(took < Duration::from_secs(20), "took {took:?}");
    Ok(())
}

/// The `spelling` rule, and the `existence` and `substitution` rules that
/// read `affix`, as a name and a rule file.
const SPELLING_RULES: [(&str, &str); 3] = [
    ("Team.Spelling", "extends: spelling\nmessage: m\n"),
    (
        "Team.Existence",
        "extends: existence\nmessage: m\naffix: en_US-web\ntokens: [utilize/DSG]\n",
    ),
    (
        "Team.Substitution",
        "extends: substitution\nmessage: m\naffix: en_US-web\nswap: {approve/DSG: okay}\n",
    ),
];

#[test]
fn spelling_and_affix_over_a_line_of_fifty_thousand_characters() -> TestResult {
    let unit = "We utilize it, then blorft the approved one. ";
    let repeats = 50_000 / unit.len() + 1;
    let source = format!("{}\n", unit.repeat(repeats));
    let (alerts, _) = lint_rules(&SPELLING_RULES, &source)?;
    for (name, _) in SPELLING_RULES {
        assert_eq!(count(&alerts, name), repeats, "{name}");
    }
    Ok(())
}

#[test]
fn spelling_and_affix_over_a_file_of_sixty_thousand_lines() -> TestResult {
    let source = "We utilize it.\nThen blorft the approved one.\n".repeat(30_000);
    let (alerts, _) = lint_rules(&SPELLING_RULES, &source)?;
    for (name, _) in SPELLING_RULES {
        assert_eq!(count(&alerts, name), 30_000, "{name}");
    }
    Ok(())
}
