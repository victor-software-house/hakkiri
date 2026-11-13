//! The `capitalization` check: the scenarios of the capitalization-check spec.
//!
//! Each test loads one `capitalization` rule, lints Markdown input, and checks
//! the alerts.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;

use hakkiri::alert::Alert;
use hakkiri::engine::lint;
use hakkiri::rule::Rule;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// A `capitalization` rule file for `matching`, with `extra` lines after it.
fn rule(matching: &str, extra: &str) -> String {
    format!(
        "extends: capitalization\nmessage: \"'%s' should be '%s'.\"\nscope: heading\nmatch: '{matching}'\n{extra}"
    )
}

/// The alerts of the rule file `text` in the Markdown `source`.
fn alerts(text: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Headings", "Team/Headings.yml", text)?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint(&[rule], &tree, source, "md")?)
}

/// The messages of the alerts.
fn messages(text: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(alerts(text, source)?
        .into_iter()
        .map(|alert| alert.message)
        .collect())
}

/// What a heading in no case the rule names gives: nothing.
const NONE: Vec<String> = Vec::new();

#[test]
fn a_heading_in_title_case() -> TestResult {
    let found = alerts(&rule("$sentence", ""), "# Getting Started With Hakkiri\n")?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 1);
    assert_eq!(found[0].matched, "Getting Started With Hakkiri");
    assert_eq!(
        found[0].message,
        "'Getting Started With Hakkiri' should be 'Getting started with hakkiri'."
    );
    Ok(())
}

#[test]
fn an_acronym_keeps_its_case() -> TestResult {
    assert_eq!(
        messages(&rule("$sentence", ""), "# Install the CLI\n")?,
        NONE
    );
    Ok(())
}

#[test]
fn a_first_word_in_lower_case() -> TestResult {
    assert_eq!(
        messages(&rule("$sentence", ""), "# getting started\n")?.len(),
        1
    );
    Ok(())
}

#[test]
fn three_words_of_four_agree() -> TestResult {
    assert_eq!(
        messages(&rule("$sentence", ""), "# Note: Run the tests\n")?.len(),
        1
    );
    Ok(())
}

#[test]
fn a_lower_threshold() -> TestResult {
    assert_eq!(
        messages(
            &rule("$sentence", "threshold: 0.7\n"),
            "# Note: Run the tests\n"
        )?,
        NONE
    );
    Ok(())
}

#[test]
fn a_colon_as_indicator() -> TestResult {
    assert_eq!(
        messages(
            &rule("$sentence", "indicators: [':']\n"),
            "# Note: Run the tests\n"
        )?,
        NONE
    );
    Ok(())
}

#[test]
fn a_product_name() -> TestResult {
    assert_eq!(
        messages(
            &rule("$sentence", "exceptions: [Hakkiri]\n"),
            "# Getting started with Hakkiri\n"
        )?,
        NONE
    );
    Ok(())
}

#[test]
fn a_preposition_of_four_letters() -> TestResult {
    assert_eq!(
        messages(
            &rule("$title", "style: Chicago\n"),
            "# Notes From the Field\n"
        )?,
        ["'Notes From the Field' should be 'Notes from the Field'."]
    );
    Ok(())
}

#[test]
fn the_same_heading_in_ap_style() -> TestResult {
    assert_eq!(
        messages(&rule("$title", "style: AP\n"), "# Notes From the Field\n")?,
        NONE
    );
    Ok(())
}

#[test]
fn to_before_an_article() -> TestResult {
    assert_eq!(
        messages(&rule("$title", "style: AP\n"), "# Welcome to the Team\n")?,
        ["'Welcome to the Team' should be 'Welcome To the Team'."]
    );
    Ok(())
}

#[test]
fn to_before_a_noun() -> TestResult {
    assert_eq!(
        messages(&rule("$title", "style: AP\n"), "# A Trip to Paris\n")?,
        NONE
    );
    Ok(())
}

#[test]
fn a_heading_that_is_not_lower_case() -> TestResult {
    assert_eq!(
        messages(&rule("$lower", ""), "# Release Notes\n")?,
        ["'Release Notes' should be 'release notes'."]
    );
    Ok(())
}

#[test]
fn a_pattern() -> TestResult {
    let found = alerts(&rule("^[A-Z][a-z]+$", ""), "# Changelog\n\n# ChangeLog\n")?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 3);
    Ok(())
}

#[test]
fn a_misspelt_style() {
    let error = Rule::from_text(
        "Team.Headings",
        "Team/Headings.yml",
        &rule("$title", "style: chicago\n"),
    )
    .err()
    .map_or_default(|error| error.to_string());
    assert!(error.contains("Team.Headings"), "{error}");
    assert!(error.contains("chicago"), "{error}");
}
