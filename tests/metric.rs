//! Metrics: the metric-check scenarios of the rule-engine change, and the
//! `metric` scenarios of its rule-files spec.
//!
//! Each test loads one `metric` rule, lints one Markdown input, and checks the
//! alerts.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;

use ctl_core::indoc;
use hakkiri::engine::lint;
use hakkiri::rule::Rule;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// The messages of the alerts that `rule` gives for the Markdown `source`.
fn messages(rule: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Rule", "Team/Rule.yml", rule)?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint(&[rule], &tree, source, "md")?
        .into_iter()
        .map(|alert| alert.message)
        .collect())
}

/// A rule that prints the formula's result and holds for any text.
fn printing(formula: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    messages(
        &format!("extends: metric\nmessage: \"%s\"\nformula: {formula}\ncondition: '> -1000000'\n"),
        source,
    )
}

#[test]
fn words_per_sentence() -> TestResult {
    let found = messages(
        indoc! {"
            extends: metric
            message: \"%s words per sentence\"
            formula: words / sentences
            condition: '>= 4'
        "},
        "One two three four five six. Seven eight.",
    )?;
    assert_eq!(found, ["4.00 words per sentence"]);
    Ok(())
}

#[test]
fn a_contraction_and_initials() -> TestResult {
    assert_eq!(printing("words", "Don't stop the U.S. team.")?, ["5.00"]);
    Ok(())
}

#[test]
fn characters_are_letters_and_digits() -> TestResult {
    assert_eq!(printing("characters", "Ab, 12 c.")?, ["5.00"]);
    Ok(())
}

#[test]
fn a_short_sentence() -> TestResult {
    let found = messages(
        "extends: metric\nmessage: \"%s\"\nformula: flesch_kincaid\ncondition: '< 0'\n",
        "The cat sat on the mat.",
    )?;
    assert_eq!(found, ["-1.45"]);
    Ok(())
}

#[test]
fn second_level_headings() -> TestResult {
    let source = "## One\n\nA.\n\n## Two\n\nB.\n\n## Three\n\nC.\n";
    let found = messages(
        "extends: metric\nmessage: \"%s\"\nformula: heading.h2\ncondition: '> 1'\n",
        source,
    )?;
    assert_eq!(found, ["3.00"]);
    Ok(())
}

#[test]
fn a_heading_count_in_a_condition() -> TestResult {
    let source = "## One\n\nA.\n\n## Two\n\nB.\n";
    let rule = |condition: &str| {
        format!("extends: metric\nmessage: \"%s\"\nformula: paragraphs\ncondition: '{condition}'\n")
    };
    assert_eq!(messages(&rule(">= heading.h2"), source)?, ["2.00"]);
    assert_eq!(
        messages(&rule("> heading.h2"), source)?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn the_comments_of_a_code_file_are_measured() -> TestResult {
    let rule = Rule::from_text(
        "Team.Rule",
        "Team/Rule.yml",
        "extends: metric\nmessage: \"%s\"\nformula: words\ncondition: '> 1'\n",
    )?;
    let source = "// One two three.\n// Four five.\nfn main() {}\n";
    let tree = parse(Language::Rust, source)?;
    let found = lint(&[rule], &tree, source, "rs")?;
    assert_eq!(
        found
            .iter()
            .map(|a| (a.message.as_str(), a.line))
            .collect::<Vec<_>>(),
        [("5.00", 1)]
    );
    Ok(())
}

#[test]
fn an_element_count_is_zero_with_a_scope() -> TestResult {
    let found = messages(
        "extends: metric\nmessage: \"%s\"\nscope: paragraph\nformula: heading.h2 + paragraphs\ncondition: '>= 0'\n",
        "## One\n\nA b.\n",
    )?;
    assert_eq!(found, ["0.00"]);
    Ok(())
}

#[test]
fn a_heading_is_not_measured() -> TestResult {
    let rule = "extends: metric\nmessage: \"%s\"\nformula: words\ncondition: '>= 0'\n";
    assert_eq!(messages(rule, "# One two three\n\nFour five.\n")?, ["2.00"]);
    assert_eq!(messages(rule, "# One two three\n")?, Vec::<String>::new());
    Ok(())
}

#[test]
fn a_two_sided_condition() -> TestResult {
    let found = messages(
        indoc! {"
            extends: metric
            message: \"%s\"
            formula: round(words / 3, 1)
            condition: '> 0.5 && result < 1'
        "},
        "One two.",
    )?;
    assert_eq!(found, ["0.70"]);
    Ok(())
}

#[test]
fn a_scope_measures_each_block() -> TestResult {
    let found = messages(
        "extends: metric\nmessage: \"%s\"\nscope: paragraph\nformula: words\ncondition: '> 1'\n",
        "One two three.\n\nFour.\n\nFive six.\n",
    )?;
    assert_eq!(found, ["3.00", "2.00"]);
    Ok(())
}

#[test]
fn a_formula_cut_short() {
    let error = Rule::from_text(
        "Team.Rate",
        "Team/Rate.yml",
        "extends: metric\nmessage: m\nformula: words /\ncondition: '> 1'\n",
    )
    .err()
    .map(|error| error.to_string());
    assert!(error.is_some_and(|error| error.contains("Team.Rate") && error.contains("formula")),);
}

#[test]
fn an_unknown_variable_is_refused() {
    let error = Rule::from_text(
        "Team.Rate",
        "Team/Rate.yml",
        "extends: metric\nmessage: m\nformula: wordz\ncondition: '> 1'\n",
    )
    .err()
    .map(|error| error.to_string());
    assert!(error.is_some_and(|error| error.contains("wordz") && error.contains("formula")),);
}

#[test]
fn a_semicolon_rate() -> TestResult {
    let rule = indoc! {"
        extends: metric
        message: \"%s semicolons per 1,000 words, over 3: split the sentences.\"
        nonword: true
        tokens: [';']
        formula: occurrences / words * 1000
        condition: '> 3'
    "};
    let found = messages(rule, "One; two; three four five six seven eight nine ten.")?;
    assert_eq!(
        found,
        ["200.00 semicolons per 1,000 words, over 3: split the sentences."]
    );
    Ok(())
}

#[test]
fn a_semicolon_with_word_boundaries() -> TestResult {
    let rule = indoc! {"
        extends: metric
        message: m
        tokens: [';']
        formula: occurrences / words * 1000
        condition: '> 3'
    "};
    assert_eq!(
        messages(rule, "One; two; three four five six seven eight nine ten.")?,
        Vec::<String>::new()
    );
    Ok(())
}
