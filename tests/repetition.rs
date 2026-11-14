//! The `repetition` check: the scenarios of the pattern-checks spec that
//! concern it.
//!
//! Each test loads one `repetition` rule, lints one Markdown input, and checks
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

/// The doubled-word rule of the spec, with `extra` lines after its fields.
fn rule(extra: &str) -> String {
    format!(
        "extends: repetition\nmessage: \"'%s' is repeated!\"\nignorecase: true\nalpha: true\ntokens: ['[^\\s.!?]+']\n{extra}"
    )
}

/// The alerts of the rule file `text` in the Markdown `source`.
fn alerts(text: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Repeat", "Team/Repeat.yml", text)?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint(
        &[rule],
        &tree,
        source,
        tree.language.langdex().as_slice(),
    )?)
}

/// The matched text of the alerts.
fn matched(text: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(alerts(text, source)?
        .into_iter()
        .map(|alert| alert.matched)
        .collect())
}

#[test]
fn a_doubled_word() -> TestResult {
    let found = alerts(&rule(""), "This is is a test.\n")?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].message, "'is' is repeated!");
    assert_eq!(found[0].matched, "is is");
    assert_eq!(
        (found[0].line, found[0].first_column, found[0].last_column),
        (1, 6, 10)
    );
    Ok(())
}

#[test]
fn a_repeat_across_two_sentences() -> TestResult {
    assert_eq!(
        matched(
            &rule(""),
            "I almost forgot about that. That is important.\n"
        )?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn two_separate_repeats_with_max_one() -> TestResult {
    assert_eq!(
        matched(&rule("max: 1\n"), "The the cat saw a a dog.\n")?,
        ["a a"]
    );
    Ok(())
}

#[test]
fn the_count_goes_back_to_zero_after_an_alert() -> TestResult {
    assert_eq!(
        matched(&rule(""), "It is is is fine.\n")?,
        ["is is", "is is"]
    );
    Ok(())
}

#[test]
fn a_repeat_over_two_sentences_still_counts() -> TestResult {
    // `Fine. Fine` repeats across a sentence break and gives no alert, but it
    // counts, so the next repeat is the second and is over `max: 1`.
    assert_eq!(
        matched(&rule("max: 1\n"), "Fine. Fine. It is is.\n")?,
        ["is is"]
    );
    assert_eq!(
        matched(&rule("max: 1\n"), "It is is.\n")?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn case_matters_without_ignorecase() -> TestResult {
    let text = "extends: repetition\nmessage: m\nalpha: true\ntokens: ['[^\\s.!?]+']\n";
    assert_eq!(matched(text, "The the cat.\n")?, Vec::<String>::new());
    assert_eq!(matched(text, "The The cat.\n")?, ["The The"]);
    Ok(())
}

#[test]
fn alpha_leaves_out_matches_that_are_not_letters() -> TestResult {
    let with = rule("");
    let without = "extends: repetition\nmessage: m\ntokens: ['[^\\s.!?]+']\n";
    assert_eq!(
        matched(&with, "It costs 5 5 dollars.\n")?,
        Vec::<String>::new()
    );
    assert_eq!(matched(without, "It costs 5 5 dollars.\n")?, ["5 5"]);
    Ok(())
}

#[test]
fn an_exception_holds_a_repeat_back() -> TestResult {
    assert_eq!(
        matched(&rule("exceptions: ['had had']\n"), "She had had enough.\n")?,
        Vec::<String>::new()
    );
    assert_eq!(
        matched(&rule("exceptions: ['had had']\n"), "She was was there.\n")?,
        ["was was"]
    );
    Ok(())
}

#[test]
fn a_match_in_code_breaks_the_chain() -> TestResult {
    assert_eq!(
        matched(&rule(""), "This `is` is a test.\n")?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn a_repeat_across_a_line_wrap() -> TestResult {
    let found = alerts(&rule(""), "This is\nis a test.\n")?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].matched, "is\nis");
    Ok(())
}

#[test]
fn a_blank_token_is_left_out() -> TestResult {
    let text = "extends: repetition\nmessage: m\nignorecase: true\ntokens: ['[^\\s.!?]+', '']\n";
    assert_eq!(matched(text, "This is is a test.\n")?, ["is is"]);
    Ok(())
}

#[test]
fn a_rule_with_no_token_is_refused() {
    let error = Rule::from_text(
        "Team.Repeat",
        "Team/Repeat.yml",
        "extends: repetition\nmessage: m\n",
    )
    .err();
    assert!(
        error.is_some_and(|error| error.to_string().contains("tokens")),
        "a rule with no token loaded"
    );
}
