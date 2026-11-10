//! The `spelling` check: the scenarios of the spelling-check spec.
//!
//! Each test loads one `spelling` rule in the package under
//! `tests/fixtures/spelling`, lints Markdown input, and checks the alerts.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::path::PathBuf;

use hakkiri::alert::Alert;
use hakkiri::engine::lint;
use hakkiri::rule::Rule;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// The package the rules are read in.
fn package() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/spelling")
}

/// A `spelling` rule file with `extra` lines after its message.
fn rule(extra: &str) -> String {
    format!("extends: spelling\nmessage: \"Did you really mean '%s'?\"\n{extra}")
}

/// The alerts of the rule file `text` in the Markdown `source`.
fn alerts(text: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rule = Rule::from_text_in("Team.Spelling", "Team/Spelling.yml", text, &package())?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint(&[rule], &tree, source, "md")?)
}

/// The matched text of each alert.
fn matched(text: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(alerts(text, source)?
        .into_iter()
        .map(|alert| alert.matched)
        .collect())
}

/// What a text with no misspelt word gives: nothing.
const NONE: Vec<String> = Vec::new();

#[test]
fn a_misspelt_word() -> TestResult {
    let found = alerts(&rule(""), "We blorft mail.\n")?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].matched, "blorft");
    assert_eq!(found[0].message, "Did you really mean 'blorft'?");
    assert_eq!((found[0].line, found[0].first_column), (1, 4));
    Ok(())
}

#[test]
fn a_word_in_a_code_span() -> TestResult {
    assert_eq!(matched(&rule(""), "Run `blorft` now.\n")?, NONE);
    Ok(())
}

#[test]
fn a_name_an_acronym_and_a_version() -> TestResult {
    assert_eq!(matched(&rule(""), "Ask FooBar and ACME about v2.\n")?, NONE);
    Ok(())
}

#[test]
fn built_in_filters_off() -> TestResult {
    assert_eq!(
        matched(&rule("custom: true\n"), "Ask FooBar now.\n")?,
        ["FooBar"]
    );
    Ok(())
}

#[test]
fn a_filter_of_the_rule() -> TestResult {
    assert_eq!(
        matched(&rule("filters: ['^hak']\n"), "Run hakkiri now.\n")?,
        NONE
    );
    Ok(())
}

#[test]
fn a_project_term_in_an_ignore_list() -> TestResult {
    assert_eq!(
        matched(&rule("ignore: [terms.txt]\n"), "Run hakkiri now.\n")?,
        NONE
    );
    assert_eq!(matched(&rule(""), "Run hakkiri now.\n")?, ["hakkiri"]);
    Ok(())
}

#[test]
fn an_identifier_without_split() -> TestResult {
    assert_eq!(
        matched(&rule(""), "Call blorftMessage now.\n")?,
        ["blorftMessage"]
    );
    Ok(())
}

#[test]
fn an_identifier_with_split() -> TestResult {
    let found = alerts(&rule("split: true\n"), "Call blorftMessage now.\n")?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].matched, "blorft");
    assert_eq!((found[0].line, found[0].first_column), (1, 6));
    Ok(())
}

#[test]
fn a_casing_mistake() -> TestResult {
    let found = alerts(&rule(""), "Push to github.\n")?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].message, "Use 'GitHub' instead of 'github'.");
    Ok(())
}

#[test]
fn an_identifier_in_markdown() -> TestResult {
    let found = alerts(&rule(""), "Set apiVersion first.\n")?;
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].message,
        "'apiVersion' looks like code; format it as code."
    );
    Ok(())
}

#[test]
fn a_word_with_a_curly_quote_is_reported_whole() -> TestResult {
    let found = alerts(&rule(""), "We blorft\u{2019}s mail.\n")?;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].matched, "blorft\u{2019}s");
    assert_eq!(found[0].message, "Did you really mean 'blorft's'?");
    assert_eq!((found[0].line, found[0].first_column), (1, 4));
    Ok(())
}

#[test]
fn a_team_dictionary_beside_the_built_in_one() -> TestResult {
    let text = rule("dictionaries: [team]\nappend: true\n");
    assert_eq!(matched(&text, "Run hakkiri on blorft.\n")?, ["blorft"]);
    Ok(())
}

#[test]
fn a_team_dictionary_alone() -> TestResult {
    let text = rule("dictionaries: [team]\n");
    assert_eq!(
        matched(&text, "Run hakkiri on mail.\n")?,
        ["Run", "on", "mail"]
    );
    Ok(())
}

#[test]
fn a_dictionary_by_its_two_files() -> TestResult {
    let text = rule("aff: team.aff\ndic: team.dic\n");
    assert_eq!(
        matched(&text, "Run hakkiri on mail.\n")?,
        ["Run", "on", "mail"]
    );
    Ok(())
}

#[test]
fn an_aff_without_a_dic_is_refused() -> TestResult {
    let refused = Rule::from_text_in(
        "Team.Spelling",
        "Team/Spelling.yml",
        &rule("aff: team.aff\n"),
        &package(),
    )
    .err()
    .ok_or("the rule loaded")?;
    assert!(refused.to_string().contains("`dic`"), "{refused}");
    Ok(())
}

#[test]
fn a_missing_dictionary_is_refused() -> TestResult {
    let refused = Rule::from_text_in(
        "Team.Spelling",
        "Team/Spelling.yml",
        &rule("dictionaries: [absent]\n"),
        &package(),
    )
    .err()
    .ok_or("the rule loaded")?;
    let message = refused.to_string();
    assert!(message.contains("Team.Spelling"), "{message}");
    assert!(message.contains("absent"), "{message}");
    Ok(())
}

#[test]
fn a_path_that_leaves_the_package_is_refused() -> TestResult {
    for extra in [
        "dictionaries: ['../x']\n",
        "ignore: ['../../Cargo.toml']\n",
        "aff: ../../Cargo.toml\ndic: ../../Cargo.toml\n",
        "dicpath: ..\n",
    ] {
        let refused = Rule::from_text_in(
            "Team.Spelling",
            "Team/Spelling.yml",
            &rule(extra),
            &package(),
        )
        .err()
        .ok_or("the rule loaded")?;
        let message = refused.to_string();
        assert!(message.contains("leaves the package"), "{extra}: {message}");
    }
    Ok(())
}

#[test]
fn a_rule_outside_a_package_reads_no_file() -> TestResult {
    let refused = Rule::from_text("Team.Spelling", "Team/Spelling.yml", &rule("ignore: [x]\n"))
        .err()
        .ok_or("the rule loaded")?;
    assert!(refused.to_string().contains("package"), "{refused}");
    Ok(())
}
