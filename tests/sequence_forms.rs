//! `sequence` rules with `forms`: the scenarios of the sequence-forms change.
//!
//! `tests/fixtures/sequence-forms` holds a rule with three forms,
//! `ThingsSpeak.yml`, the same three lists as the one-list rules of `single`,
//! and `inputs.txt`, one sentence a line.
//! A one-list rule reads `tokens` as Vale does, which the `sequence` tests
//! compare with Vale's alerts, so the three rules together give the alerts the
//! rule with three forms must give.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use hakkiri::alert::Alert;
use hakkiri::engine::lint;
use hakkiri::rule::Rule;
use hakkiri::{Language, parse};

type TestResult = Result<(), Box<dyn Error>>;

/// A place and what was matched there: the line, the first column, the
/// matched text and the message.
type Spot = (usize, usize, String, String);

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sequence-forms")
}

/// The alerts of the rule file `text` in the Markdown `source`.
fn alerts(text: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Words", "Team/Words.yml", text)?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint(&[rule], &tree, source, "md")?)
}

/// The spots of the alerts of `text` in `source`, in order of place.
fn spots(text: &str, source: &str) -> Result<Vec<Spot>, Box<dyn Error>> {
    let mut spots: Vec<Spot> = alerts(text, source)?
        .into_iter()
        .map(|alert| (alert.line, alert.first_column, alert.matched, alert.message))
        .collect();
    spots.sort();
    Ok(spots)
}

/// The matched text of each alert of `text` in `source`.
fn matched(text: &str, source: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(alerts(text, source)?
        .into_iter()
        .map(|alert| alert.matched)
        .collect())
}

/// The message of the error that loading `text` gives.
fn refusal(text: &str) -> Result<String, Box<dyn Error>> {
    match Rule::from_text("Team.Words", "Team/Words.yml", text) {
        Ok(_) => Err("the rule loaded".into()),
        Err(error) => Ok(error.to_string()),
    }
}

/// A rule of two forms of plain words, with `body` as `forms`.
fn forms(body: &str) -> String {
    format!("extends: sequence\nmessage: \"'%s'.\"\nforms:\n{body}")
}

const RED_GREEN_OR_GREEN_BLUE: &str = "  - tokens: [{pattern: red}, {pattern: green}]\n  - tokens: [{pattern: green}, {pattern: blue}]\n";

#[test]
fn the_forms_give_the_alerts_of_their_lists_apart() -> TestResult {
    let merged = fs::read_to_string(fixture().join("ThingsSpeak.yml"))?;
    let apart: Vec<String> = ["Present", "Past", "Pronoun"]
        .iter()
        .map(|name| fs::read_to_string(fixture().join(format!("single/{name}.yml"))))
        .collect::<Result<_, _>>()?;
    let inputs = fs::read_to_string(fixture().join("inputs.txt"))?;
    let mut with_alerts = 0;
    for sentence in inputs.lines() {
        let source = format!("{sentence}\n");
        let mut want = Vec::new();
        for text in &apart {
            want.extend(spots(text, &source)?);
        }
        want.sort();
        let got = spots(&merged, &source)?;
        with_alerts += usize::from(!want.is_empty());
        assert_eq!(got, want, "{sentence}");
    }
    assert_eq!(with_alerts, 7);
    Ok(())
}

#[test]
fn the_merged_rule_reads_a_noun_a_pronoun_and_a_guard() -> TestResult {
    let merged = fs::read_to_string(fixture().join("ThingsSpeak.yml"))?;
    let read = |sentence: &str| matched(&merged, &format!("{sentence}\n"));
    assert_eq!(read("The doc says to run the tool.")?, ["doc says"]);
    assert_eq!(read("The doc asked why the build failed.")?, ["doc asked"]);
    assert_eq!(read("The jobs asked to end early.")?, Vec::<String>::new());
    assert_eq!(
        read("The state is open and it knows its names.")?,
        ["it knows"]
    );
    assert_eq!(read("Both say record type.")?, ["Both say"]);
    assert_eq!(
        read("The request is large, and which asks for the rows is unclear.")?,
        ["which asks"]
    );
    assert_eq!(read("The operator said to keep it.")?, Vec::<String>::new());
    Ok(())
}

#[test]
fn an_alert_carries_the_one_rule_name() -> TestResult {
    let merged = fs::read_to_string(fixture().join("ThingsSpeak.yml"))?;
    let found = alerts(&merged, "The doc says so, and it knows why.\n")?;
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|alert| alert.rule == "Team.Words"));
    Ok(())
}

#[test]
fn a_match_of_any_form_is_reported() -> TestResult {
    let text = forms(RED_GREEN_OR_GREEN_BLUE);
    assert_eq!(
        matched(&text, "Paint red green or green blue.\n")?,
        ["red green", "green blue"]
    );
    assert_eq!(matched(&text, "Paint blue red.\n")?, Vec::<String>::new());
    Ok(())
}

#[test]
fn the_earlier_of_two_overlapping_matches_stays() -> TestResult {
    let text = forms(RED_GREEN_OR_GREEN_BLUE);
    assert_eq!(matched(&text, "Paint red green blue.\n")?, ["red green"]);
    let swapped = forms(
        "  - tokens: [{pattern: green}, {pattern: blue}]\n  - tokens: [{pattern: red}, {pattern: green}]\n",
    );
    assert_eq!(matched(&swapped, "Paint red green blue.\n")?, ["red green"]);
    Ok(())
}

#[test]
fn matches_of_one_list_overlap_as_they_do_without_forms() -> TestResult {
    let tokens =
        "extends: sequence\nmessage: \"'%s'.\"\ntokens: [{pattern: red}, {pattern: red}]\n";
    let one_form = forms("  - tokens: [{pattern: red}, {pattern: red}]\n");
    let source = "Paint red red red red.\n";
    assert_eq!(matched(tokens, source)?.len(), 3);
    assert_eq!(spots(&one_form, source)?, spots(tokens, source)?);
    Ok(())
}

#[test]
fn a_rule_holds_tokens_or_forms_not_both() -> TestResult {
    let both = "extends: sequence\nmessage: m\ntokens: [{pattern: red}]\nforms:\n  - tokens: [{pattern: green}]\n";
    let message = refusal(both)?;
    assert!(
        message.contains("`tokens` and `forms` are both set"),
        "{message}"
    );
    let neither = refusal("extends: sequence\nmessage: m\n")?;
    assert!(neither.contains("no `tokens` or `forms`"), "{neither}");
    Ok(())
}

#[test]
fn a_fault_in_a_form_names_the_form() -> TestResult {
    let empty = refusal(&forms(
        "  - tokens: [{pattern: red}]\n  - tokens: [{negate: true, pattern: green}]\n",
    ))?;
    assert!(empty.contains("forms[1].tokens"), "{empty}");
    let bad = refusal(&forms(
        "  - tokens: [{pattern: red}]\n  - tokens: [{pattern: green}, {pattern: '('}]\n",
    ))?;
    assert!(bad.contains("forms[1].tokens[1].pattern"), "{bad}");
    let stray = refusal(&forms("  - pattern: red\n"))?;
    assert!(stray.contains("pattern"), "{stray}");
    Ok(())
}
