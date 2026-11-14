//! The `affix` field of `existence` and `substitution` rules: the scenarios
//! of the spelling-check spec.
//!
//! Each test loads one rule in the package under `tests/fixtures/spelling`,
//! lints Markdown input, and checks the alerts or the error that refuses it.

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

/// The alerts of the rule file `text` in the Markdown `source`.
fn alerts(text: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rule = Rule::from_text_in("Team.Forms", "Team/Forms.yml", text, &package())?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint(
        &[rule],
        &tree,
        source,
        tree.language.langdex().as_slice(),
    )?)
}

/// The matched text of each alert.
fn matched(text: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(alerts(text, source)?
        .into_iter()
        .map(|alert| alert.matched)
        .collect())
}

/// What the error that refuses the rule file `text` says.
fn refusal(text: &str) -> Result<String, Box<dyn Error>> {
    let refused = Rule::from_text_in("Team.Forms", "Team/Forms.yml", text, &package())
        .err()
        .ok_or("the rule loaded")?;
    Ok(refused.to_string())
}

/// An `existence` rule file that reads `tokens` under `affix`.
fn existence(affix: &str, tokens: &str) -> String {
    format!("extends: existence\nmessage: m\naffix: {affix}\ntokens: [{tokens}]\n")
}

#[test]
fn every_form_of_a_verb() -> TestResult {
    let text = "extends: substitution\nmessage: \"Use '%s' instead of '%s'.\"\n\
                affix: en_US-web\nswap: {utilize/DSG: use}\n";
    let found = alerts(text, "We utilized it.\n")?;
    let [alert] = found.as_slice() else {
        return Err(format!("expected one alert: {found:?}").into());
    };
    assert_eq!(alert.message, "Use 'use' instead of 'utilized'.");
    Ok(())
}

#[test]
fn the_forms_of_a_flag_and_no_others() -> TestResult {
    let found = matched(
        &existence("en_US-web", "utilize/DSG"),
        "We utilize it, utilizes it, utilized it, utilizing it and utilization.\n",
    )?;
    assert_eq!(found, ["utilize", "utilizes", "utilized", "utilizing"]);
    Ok(())
}

#[test]
fn a_word_with_no_flags_after_the_slash_is_the_word() -> TestResult {
    let found = matched(
        &existence("en_US-web", "utilize/"),
        "We utilize it and utilizes it.\n",
    )?;
    assert_eq!(found, ["utilize"]);
    Ok(())
}

#[test]
fn a_part_with_flags_among_plain_parts() -> TestResult {
    let found = matched(
        &existence("en_US-web", "'approve/D respect'"),
        "Approve respect, approved respect and approve respect.\n",
    )?;
    assert_eq!(found, ["approved respect", "approve respect"]);
    Ok(())
}

#[test]
fn word_boundaries_stay_outside_the_forms() -> TestResult {
    let found = matched(
        &existence("en_US-web", r"'\bcolor/S\b'"),
        "The color of colors and colored things.\n",
    )?;
    assert_eq!(found, ["color", "colors"]);
    Ok(())
}

#[test]
fn an_escaped_slash_is_text() -> TestResult {
    let found = matched(
        &existence("en_US-web", r"'and\/or'"),
        "Use and/or here, and and or there.\n",
    )?;
    assert_eq!(found, ["and/or"]);
    Ok(())
}

#[test]
fn a_rule_with_no_affix_leaves_a_slash_alone() -> TestResult {
    let text = "extends: existence\nmessage: m\ntokens: ['and/or']\n";
    assert_eq!(matched(text, "Use and/or here.\n")?, ["and/or"]);
    Ok(())
}

#[test]
fn an_affix_file_of_the_package() -> TestResult {
    let found = matched(
        &existence("forms", "walk/US, carry/S"),
        "They walk, walks, unwalk, carry and carries.\n",
    )?;
    assert_eq!(found, ["walk", "walks", "unwalk", "carry", "carries"]);
    Ok(())
}

#[test]
fn only_the_keys_of_a_swap_expand() -> TestResult {
    let text = "extends: substitution\nmessage: \"Use '%s' instead of '%s'.\"\n\
                affix: forms\nswap: {walk/D: go/D}\n";
    let found = alerts(text, "They walked and walk.\n")?;
    let messages: Vec<&str> = found.iter().map(|alert| alert.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "Use 'go/D' instead of 'walked'.",
            "Use 'go/D' instead of 'walk'."
        ]
    );
    Ok(())
}

#[test]
fn an_affix_file_that_is_missing_is_refused() -> TestResult {
    let message = refusal(&existence("absent", "run/G"))?;
    assert!(message.contains("Team.Forms"), "{message}");
    assert!(message.contains("absent.aff"), "{message}");
    Ok(())
}

#[test]
fn a_flag_that_names_no_class_is_refused() -> TestResult {
    for (affix, token) in [("en_US-web", "utilize/Q"), ("forms", "walk/Z")] {
        let message = refusal(&existence(affix, token))?;
        assert!(message.contains(token), "{message}");
        assert!(message.contains("literal slash"), "{message}");
    }
    let swap = "extends: substitution\nmessage: m\naffix: en_US-web\nswap: {utilize/Q: use}\n";
    assert!(refusal(swap)?.contains("utilize/Q"));
    Ok(())
}

#[test]
fn a_word_that_is_not_plain_text_is_refused() -> TestResult {
    let message = refusal(&existence("en_US-web", "'a.b/D'"))?;
    assert!(message.contains("plain text"), "{message}");
    Ok(())
}

#[test]
fn flags_with_no_word_are_refused() -> TestResult {
    let message = refusal(&existence("en_US-web", "'/D'"))?;
    assert!(message.contains("need a word"), "{message}");
    Ok(())
}

#[test]
fn an_affix_file_outside_a_package_is_not_read() -> TestResult {
    let text = existence("forms", "walk/S");
    let refused = Rule::from_text("Team.Forms", "Team/Forms.yml", &text)
        .err()
        .ok_or("the rule loaded")?;
    assert!(refused.to_string().contains("package"), "{refused}");
    Rule::from_text(
        "Team.Forms",
        "Team/Forms.yml",
        &existence("en_US-web", "utilize/D"),
    )?;
    Ok(())
}

#[test]
fn a_path_that_leaves_the_package_is_refused() -> TestResult {
    let message = refusal(&existence("../../x", "run/G"))?;
    assert!(message.contains("leaves the package"), "{message}");
    Ok(())
}
