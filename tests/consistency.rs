//! The `consistency` check: the scenarios of the pattern-checks spec that
//! concern it.
//!
//! Each test loads one `consistency` rule, lints Markdown input, and checks the
//! alerts.

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

/// The rule of the spec for the `either` mapping `pairs`, with `extra` lines
/// after it.
fn rule(pairs: &str, extra: &str) -> String {
    format!("extends: consistency\nmessage: \"Inconsistent: '%s'\"\neither: {{{pairs}}}\n{extra}")
}

/// The alerts of the rule file `text` in each of the Markdown `files`, one
/// lint run for each.
fn alerts(text: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Spelling", "Team/Spelling.yml", text)?;
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

#[test]
fn two_spellings_in_two_paragraphs() -> TestResult {
    let found = alerts(
        &rule("advisor: adviser", ""),
        "An advisor came.\n\nThe adviser left.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].message, "Inconsistent: 'adviser'");
    assert_eq!(
        (found[0].line, found[0].first_column, found[0].last_column),
        (3, 5, 11)
    );
    Ok(())
}

#[test]
fn one_spelling_throughout() -> TestResult {
    assert_eq!(
        messages(
            &rule("advisor: adviser", ""),
            "An advisor came.\n\nThe advisor left.\n"
        )?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn two_pairs_one_mixed() -> TestResult {
    let found = alerts(
        &rule("advisor: adviser, color: colour", ""),
        "The advisor likes color. The adviser likes color.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].rule, "Team.Spelling");
    assert_eq!(found[0].matched, "adviser");
    Ok(())
}

#[test]
fn a_paragraph_that_holds_both_forms() -> TestResult {
    let found = alerts(
        &rule("advisor: adviser", ""),
        "The advisor and the adviser met.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].matched, "adviser");
    Ok(())
}

#[test]
fn the_alert_is_at_the_last_match_of_the_pair() -> TestResult {
    let found = alerts(
        &rule("advisor: adviser", ""),
        "An advisor came.\n\nThe adviser met an advisor and an adviser.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].matched, "adviser");
    assert_eq!((found[0].line, found[0].first_column), (3, 35));
    Ok(())
}

#[test]
fn the_other_form_first() -> TestResult {
    let found = alerts(
        &rule("advisor: adviser", ""),
        "An adviser came.\n\nThe advisor left.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].matched, "advisor");
    Ok(())
}

#[test]
fn a_pair_of_other_words_does_not_count() -> TestResult {
    assert_eq!(
        messages(
            &rule("advisor: adviser, color: colour", ""),
            "An advisor came.\n\nThe colour left.\n"
        )?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn case_matters_without_ignorecase() -> TestResult {
    let pairs = "advisor: adviser";
    let source = "An advisor came.\n\nThe Adviser left.\n";
    assert_eq!(messages(&rule(pairs, ""), source)?, Vec::<String>::new());
    assert_eq!(
        messages(&rule(pairs, "ignorecase: true\n"), source)?,
        ["Inconsistent: 'Adviser'"]
    );
    Ok(())
}

#[test]
fn word_boundaries_unless_nonword() -> TestResult {
    let pairs = "advis: advi";
    let source = "An advisor came.\n\nThe advice left.\n";
    assert_eq!(messages(&rule(pairs, ""), source)?, Vec::<String>::new());
    assert_eq!(
        messages(&rule(pairs, "nonword: true\n"), source)?,
        ["Inconsistent: 'advi'"]
    );
    Ok(())
}

#[test]
fn a_form_in_code_is_not_a_match() -> TestResult {
    assert_eq!(
        messages(
            &rule("advisor: adviser", ""),
            "An advisor came.\n\nThe `adviser` left.\n"
        )?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn a_form_with_a_space_matches_across_a_line_wrap() -> TestResult {
    let found = alerts(
        &rule("e-mail address: email address", ""),
        "An e-mail\naddress came.\n\nThe email address left.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].matched, "email address");
    let found = alerts(
        &rule("e-mail address: email address", ""),
        "An e-mail address came.\n\nThe email\naddress left.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].matched, "email\naddress");
    Ok(())
}

#[test]
fn a_rule_starts_each_file_with_nothing_seen() -> TestResult {
    let text = rule("advisor: adviser", "");
    let first = messages(&text, "An advisor came.\n")?;
    let second = messages(&text, "The adviser left.\n")?;
    assert_eq!((first.len(), second.len()), (0, 0));
    Ok(())
}

#[test]
fn a_rule_with_no_pair_is_refused() {
    let error = Rule::from_text(
        "Team.Spelling",
        "Team/Spelling.yml",
        "extends: consistency\nmessage: m\n",
    )
    .err();
    assert!(
        error.is_some_and(|error| error.to_string().contains("either")),
        "a rule with no pair loaded"
    );
}

#[test]
fn a_form_with_no_text_is_refused() {
    let error = Rule::from_text(
        "Team.Spelling",
        "Team/Spelling.yml",
        &rule("advisor: ''", ""),
    )
    .err();
    assert!(
        error.is_some_and(|error| error.to_string().contains("either")),
        "a form with no text loaded"
    );
}
