//! The `conditional` check: the scenarios of the pattern-checks spec that
//! concern it.
//!
//! Each test loads one `conditional` rule, lints Markdown input, and checks the
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

/// A `conditional` rule file for the patterns `first` and `second`, with
/// `extra` lines after them.
fn rule(first: &str, second: &str, extra: &str) -> String {
    format!(
        "extends: conditional\nmessage: \"Define '%s'.\"\nfirst: '{first}'\nsecond: '{second}'\n{extra}"
    )
}

/// The acronym rule of the spec.
fn acronyms(extra: &str) -> String {
    rule(r"\b[A-Z]{3,5}\b", r"\(([A-Z]{3,5})\)", extra)
}

/// The alerts of the rule file `text` in the Markdown `source`.
fn alerts(text: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Defined", "Team/Defined.yml", text)?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint(&[rule], &tree, source, "md")?)
}

/// The matched text of the alerts.
fn matched(text: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(alerts(text, source)?
        .into_iter()
        .map(|alert| alert.matched)
        .collect())
}

#[test]
fn a_term_with_no_definition_nearby() -> TestResult {
    let found = alerts(
        &rule(r"\bAPI\b", "application programming interface", ""),
        "Call the API.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].message, "Define 'API'.");
    Ok(())
}

#[test]
fn a_definition_in_the_same_block_is_enough() -> TestResult {
    assert_eq!(
        matched(
            &rule(r"\bAPI\b", "application programming interface", ""),
            "The application programming interface, the API, is stable.\n"
        )?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn a_definition_in_another_block_is_not() -> TestResult {
    assert_eq!(
        matched(
            &rule(r"\bAPI\b", "application programming interface", ""),
            "The application programming interface is stable.\n\nCall the API.\n"
        )?,
        ["API"]
    );
    Ok(())
}

#[test]
fn an_acronym_defined_once() -> TestResult {
    assert_eq!(
        matched(
            &acronyms(""),
            "The World Health Organization (WHO) met. WHO agreed.\n\nThe CDC said no.\n"
        )?,
        ["CDC"]
    );
    Ok(())
}

#[test]
fn a_definition_after_the_use() -> TestResult {
    let found = alerts(
        &acronyms(""),
        "The CDC said no.\n\nThe Centers for Disease Control (CDC) spoke.\n",
    )?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, 1);
    Ok(())
}

#[test]
fn the_definition_in_the_same_block_counts() -> TestResult {
    assert_eq!(
        matched(
            &acronyms(""),
            "The Centers for Disease Control (CDC) said that CDC agreed.\n"
        )?,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn ignorecase_reaches_both_patterns() -> TestResult {
    let text = rule(
        r"\bAPI\b",
        "application programming interface",
        "ignorecase: true\n",
    );
    assert_eq!(
        matched(
            &text,
            "The Application Programming Interface is stable. Call the API.\n"
        )?,
        Vec::<String>::new()
    );
    assert_eq!(matched(&text, "Call the api.\n")?, ["api"]);
    Ok(())
}

#[test]
fn without_ignorecase_case_matters() -> TestResult {
    assert_eq!(
        matched(
            &rule(r"\bAPI\b", "application programming interface", ""),
            "The Application Programming Interface is stable. Call the API.\n"
        )?,
        ["API"]
    );
    Ok(())
}

#[test]
fn ignorecase_reaches_a_defined_term() -> TestResult {
    let source = "Centers (CDC) met. The cdc agreed.\n";
    let with = rule(r"\b[A-Za-z]{3}\b", r"\(([A-Z]{3})\)", "ignorecase: true\n");
    let without = rule(r"\b[A-Za-z]{3}\b", r"\(([A-Z]{3})\)", "");
    assert_eq!(matched(&with, source)?, ["met", "The"]);
    assert_eq!(matched(&without, source)?, ["met", "The", "cdc"]);
    Ok(())
}

#[test]
fn an_exception_holds_a_match_back() -> TestResult {
    assert_eq!(
        matched(&acronyms("exceptions: [NASA]\n"), "NASA and ESA met.\n")?,
        ["ESA"]
    );
    Ok(())
}

#[test]
fn a_match_in_code_is_no_match() -> TestResult {
    let text = acronyms("");
    assert_eq!(matched(&text, "Run `CDC` now.\n")?, Vec::<String>::new());
    assert_eq!(
        matched(&text, "Run `(CDC)` first.\n\nThe CDC said no.\n")?,
        ["CDC"]
    );
    Ok(())
}

#[test]
fn a_parenthesis_that_is_not_a_group() -> TestResult {
    // `\(CDC\)` and `[(]` hold no capture group, and `(?:...)` neither: the
    // rule needs `second` in the same block, and defines no terms.
    for second in [r"\(CDC\)", r"[(]CDC", r"(?:Centers)"] {
        let text = rule(r"\bCDC\b", second, "");
        assert_eq!(
            matched(&text, "The Centers (CDC) spoke.\n\nThe CDC said no.\n")?,
            ["CDC"],
            "{second}"
        );
    }
    Ok(())
}

#[test]
fn a_named_group_is_a_group() -> TestResult {
    for second in [r"\((?P<term>[A-Z]{3,5})\)", r"\((?<term>[A-Z]{3,5})\)"] {
        let text = rule(r"\b[A-Z]{3,5}\b", second, "");
        assert_eq!(
            matched(&text, "The Centers (CDC) spoke.\n\nThe CDC said no.\n")?,
            Vec::<String>::new(),
            "{second}"
        );
    }
    Ok(())
}

#[test]
fn a_rule_that_looks_in_a_view() {
    let error = Rule::from_text(
        "Team.Defined",
        "Team/Defined.yml",
        &rule(r"\bAPI\b", "interface", "in: glossary\n"),
    )
    .err();
    let text = error.map(|error| error.to_string()).unwrap_or_default();
    assert!(text.contains("Team.Defined"), "{text}");
    assert!(text.contains("`in`"), "{text}");
}

#[test]
fn a_pattern_with_no_text_is_refused() {
    for (first, second, field) in [("", "interface", "first"), (r"\bAPI\b", "", "second")] {
        let error =
            Rule::from_text("Team.Defined", "Team/Defined.yml", &rule(first, second, "")).err();
        let text = error.map(|error| error.to_string()).unwrap_or_default();
        assert!(text.contains("Team.Defined"), "{text}");
        assert!(text.contains(&format!("`{field}` holds nothing")), "{text}");
    }
}

#[test]
fn a_space_is_a_pattern() -> TestResult {
    let found = matched(&rule(" ", "zzz", ""), "a b\n")?;
    assert_eq!(found, [" "]);
    Ok(())
}

#[test]
fn a_pattern_that_matches_the_empty_string_is_refused() {
    for (first, second, field) in [("x*", "interface", "first"), (r"\bAPI\b", "()", "second")] {
        let error =
            Rule::from_text("Team.Defined", "Team/Defined.yml", &rule(first, second, "")).err();
        let text = error.map(|error| error.to_string()).unwrap_or_default();
        assert!(text.contains("Team.Defined"), "{text}");
        assert!(
            text.contains(&format!("`{field}` matches the empty string")),
            "{text}"
        );
    }
}

#[test]
fn a_pattern_that_does_not_compile() {
    let error = Rule::from_text(
        "Team.Defined",
        "Team/Defined.yml",
        &rule(r"\bAPI(", "interface", ""),
    )
    .err();
    let text = error.map(|error| error.to_string()).unwrap_or_default();
    assert!(text.contains("first"), "{text}");
}
