//! The `readability` check: the scenarios of the readability-check spec of the
//! pattern-and-readability-checks change.
//!
//! Each test loads one `readability` rule, lints one Markdown input, and
//! checks the alerts.

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

/// A `readability` rule file for `metrics` and `grade`, with `scope` when it
/// is not empty.
fn rule(metrics: &str, grade: &str, scope: &str) -> String {
    let scope = if scope.is_empty() {
        String::new()
    } else {
        format!("scope: {scope}\n")
    };
    format!(
        "extends: readability\nmessage: \"Grade %s.\"\n{scope}metrics: [{metrics}]\ngrade: {grade}\n"
    )
}

/// The alerts of the rule file `text` in the Markdown `source`.
fn alerts(text: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Grade", "Team/Grade.yml", text)?;
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

/// What an alert names about its place: line and first column.
fn place(alert: &Alert) -> (usize, usize) {
    (alert.line, alert.first_column)
}

#[test]
fn one_score() -> TestResult {
    assert_eq!(
        messages(
            &rule("Flesch-Kincaid", "-2", ""),
            "The cat sat on the mat.\n"
        )?,
        ["Grade -1.45."]
    );
    Ok(())
}

#[test]
fn two_scores_averaged() -> TestResult {
    assert_eq!(
        messages(
            &rule("Flesch-Kincaid, SMOG", "0.5", ""),
            "The cat sat on the mat.\n"
        )?,
        ["Grade 0.84."]
    );
    Ok(())
}

#[test]
fn a_grade_under_the_limit() -> TestResult {
    assert_eq!(
        messages(
            &rule("Flesch-Kincaid", "8", ""),
            "The cat sat on the mat.\n"
        )?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn every_score_is_known() -> TestResult {
    let all = "Automated Readability, Coleman-Liau, Flesch-Kincaid, Gunning Fog, SMOG";
    assert_eq!(
        alerts(&rule(all, "-100", ""), "The cat sat on the mat.\n")?.len(),
        1
    );
    Ok(())
}

#[test]
fn a_score_listed_twice_counts_once() -> TestResult {
    assert_eq!(
        messages(
            &rule("Flesch-Kincaid, Flesch-Kincaid", "-2", ""),
            "The cat sat on the mat.\n"
        )?,
        ["Grade -1.45."]
    );
    Ok(())
}

#[test]
fn a_score_hakkiri_does_not_know() -> TestResult {
    let error = Rule::from_text("Team.Grade", "Team/Grade.yml", &rule("Dale-Chall", "8", ""))
        .err()
        .ok_or("the rule loaded")?;
    let text = error.to_string();
    assert!(text.contains("Team.Grade"), "{text}");
    assert!(text.contains("Dale-Chall"), "{text}");
    assert!(text.contains("metrics"), "{text}");
    Ok(())
}

#[test]
fn an_empty_list_of_scores() -> TestResult {
    let error = Rule::from_text("Team.Grade", "Team/Grade.yml", &rule("", "8", ""))
        .err()
        .ok_or("the rule loaded")?;
    let text = error.to_string();
    assert!(text.contains("Team.Grade"), "{text}");
    assert!(text.contains("no score"), "{text}");
    Ok(())
}

#[test]
fn a_missing_list_of_scores() -> TestResult {
    let error = Rule::from_text(
        "Team.Grade",
        "Team/Grade.yml",
        "extends: readability\nmessage: m\ngrade: 8\n",
    )
    .err()
    .ok_or("the rule loaded")?;
    assert!(error.to_string().contains("no score"), "{error}");
    Ok(())
}

#[test]
fn a_grade_that_is_not_a_number() -> TestResult {
    let error = Rule::from_text(
        "Team.Grade",
        "Team/Grade.yml",
        "extends: readability\nmessage: m\nmetrics: [SMOG]\ngrade: high\n",
    )
    .err()
    .ok_or("the rule loaded")?;
    let text = error.to_string();
    assert!(text.contains("grade"), "{text}");
    assert!(text.contains("must be a number"), "{text}");
    Ok(())
}

#[test]
fn two_paragraphs_measured_as_one() -> TestResult {
    let found = alerts(
        &rule("Flesch-Kincaid", "-5", ""),
        "The cat sat.\n\nThe dog ran.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found.iter().map(place).collect::<Vec<_>>(), [(1, 1)]);
    assert_eq!(
        found
            .iter()
            .map(|alert| alert.message.as_str())
            .collect::<Vec<_>>(),
        ["Grade -2.62."]
    );
    Ok(())
}

#[test]
fn the_text_scope_measures_the_whole_file() -> TestResult {
    let found = alerts(
        &rule("Flesch-Kincaid", "-5", "text"),
        "The cat sat.\n\nThe dog ran.\n",
    )?;
    assert_eq!(found.iter().map(place).collect::<Vec<_>>(), [(1, 1)]);
    Ok(())
}

#[test]
fn only_a_heading() -> TestResult {
    assert_eq!(
        messages(&rule("Flesch-Kincaid", "-5", ""), "# One two three\n")?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn each_paragraph_measured() -> TestResult {
    let found = alerts(
        &rule("Flesch-Kincaid", "-5", "paragraph"),
        "The cat sat.\n\nThe dog ran.\n",
    )?;
    assert_eq!(
        found.iter().map(place).collect::<Vec<_>>(),
        [(1, 1), (3, 1)]
    );
    assert!(found.iter().all(|alert| alert.message == "Grade -2.62."));
    Ok(())
}

#[test]
fn each_list_item_is_reported_at_column_one() -> TestResult {
    let found = alerts(
        &rule("Flesch-Kincaid", "-5", "list"),
        "- The cat sat.\n- The dog ran.\n",
    )?;
    assert_eq!(
        found.iter().map(place).collect::<Vec<_>>(),
        [(1, 1), (2, 1)]
    );
    Ok(())
}
