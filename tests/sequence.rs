//! The `sequence` check: the scenarios of the sequence-check spec.
//!
//! Each test loads one `sequence` rule, lints Markdown input, and checks the
//! alerts or the refusal.

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

/// A `sequence` rule file with `message`, then `body`.
fn rule(message: &str, body: &str) -> String {
    format!("extends: sequence\nmessage: \"{message}\"\n{body}")
}

/// The alerts of the rule file `text` in the Markdown `source`.
fn alerts(text: &str, source: &str) -> Result<Vec<Alert>, Box<dyn Error>> {
    let rule = Rule::from_text("Team.Words", "Team/Words.yml", text)?;
    let tree = parse(Language::Markdown, source)?;
    Ok(lint(&[rule], &tree, source, "md")?)
}

/// The matched text and the message of each alert.
fn found(text: &str, source: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    Ok(alerts(text, source)?
        .into_iter()
        .map(|alert| (alert.matched, alert.message))
        .collect())
}

/// The message of the error that loading `text` gives.
fn refusal(name: &str, text: &str) -> Result<String, Box<dyn Error>> {
    match Rule::from_text(name, "Team/Rule.yml", text) {
        Ok(_) => Err("the rule loaded".into()),
        Err(error) => Ok(error.to_string()),
    }
}

const YOUR_SELF: &str = "tokens: [{pattern: your}, {pattern: self}]\n";

fn your_self() -> String {
    rule("Write '%s%s' as one word.", YOUR_SELF)
}

/// No alert.
fn none() -> Vec<(String, String)> {
    Vec::new()
}

fn pair(matched: &str, message: &str) -> (String, String) {
    (matched.to_owned(), message.to_owned())
}

#[test]
fn two_words_as_patterns() -> TestResult {
    assert_eq!(
        found(&your_self(), "Do it your self.\n")?,
        [pair("your self", "Write 'yourself' as one word.")]
    );
    Ok(())
}

#[test]
fn a_pattern_is_a_whole_word() -> TestResult {
    assert_eq!(found(&your_self(), "Value your self-worth.\n")?, none());
    Ok(())
}

#[test]
fn a_tag_and_a_pattern() -> TestResult {
    let text = rule("Passive: '%s %s'.", "tokens: [{tag: MD}, {pattern: be}]\n");
    assert_eq!(
        found(&text, "The file can be read.\n")?,
        [pair("can be", "Passive: 'can be'.")]
    );
    Ok(())
}

#[test]
fn sentences_of_list_items() -> TestResult {
    let text = rule(
        "Write '%s%s' as one word.",
        &format!("scope: list\n{YOUR_SELF}"),
    );
    let found = alerts(&text, "Do it your self.\n\n- Do it your self.\n")?;
    assert_eq!(
        found.iter().map(|alert| alert.line).collect::<Vec<_>>(),
        [3]
    );
    Ok(())
}

#[test]
fn a_match_does_not_cross_a_sentence_end() -> TestResult {
    assert_eq!(found(&your_self(), "Do it your. Self first.\n")?, none());
    Ok(())
}

fn if_then() -> String {
    rule(
        "Drop 'then'.",
        "ignorecase: true\ntokens: [{pattern: if}, {pattern: then, skip: 2}]\n",
    )
}

#[test]
fn words_between_two_tokens() -> TestResult {
    assert_eq!(
        found(&if_then(), "If it rains then stay.\n")?,
        [pair("If it rains then", "Drop 'then'.")]
    );
    Ok(())
}

#[test]
fn too_many_words_between() -> TestResult {
    assert_eq!(
        found(&if_then(), "If it rains all day then stay.\n")?,
        none()
    );
    Ok(())
}

#[test]
fn a_word_twice() -> TestResult {
    let text = rule("Repeated.", "tokens: [{pattern: very, min: 2}]\n");
    assert_eq!(
        found(&text, "This is very very good.\n")?,
        [pair("very very", "Repeated.")]
    );
    Ok(())
}

fn no_noun(scope: &str) -> String {
    rule(
        "No noun.",
        &format!("{scope}tokens: [{{pattern: the}}, {{tag: NN, negate: true}}]\n"),
    )
}

#[test]
fn not_followed_by_a_noun() -> TestResult {
    assert_eq!(
        found(&no_noun(""), "Pick the red one.\n")?,
        [pair("the red", "No noun.")]
    );
    Ok(())
}

#[test]
fn a_negated_token_at_the_end() -> TestResult {
    assert_eq!(
        found(&no_noun("scope: heading\n"), "# Open the\n")?,
        [pair("the", "No noun.")]
    );
    Ok(())
}

#[test]
fn point_at_one_word() -> TestResult {
    let text = rule(
        "Join it.",
        "tokens: [{pattern: your}, {pattern: self, target: true}]\n",
    );
    assert_eq!(
        found(&text, "Do it your self.\n")?,
        [pair("self", "Join it.")]
    );
    Ok(())
}

#[test]
fn a_universal_noun() -> TestResult {
    let text = rule("Found.", "tokens: [{pattern: the}, {upos: NOUN}]\n");
    assert_eq!(
        found(&text, "Open the file.\n")?,
        [pair("the file", "Found.")]
    );
    Ok(())
}

#[test]
fn both_tag_kinds_are_refused() -> TestResult {
    let message = refusal("Team.Noun", &rule("m", "tokens: [{tag: NN, upos: NOUN}]\n"))?;
    assert!(message.contains("Team.Noun"), "{message}");
    assert!(message.contains("`tag` and `upos`"), "{message}");
    Ok(())
}

#[test]
fn an_unknown_universal_tag_is_refused() -> TestResult {
    let message = refusal("Team.Noun", &rule("m", "tokens: [{upos: NAME}]\n"))?;
    assert!(message.contains("NAME"), "{message}");
    assert!(message.contains("NOUN"), "{message}");
    Ok(())
}

#[test]
fn a_quoted_phrase_is_an_exception() -> TestResult {
    let text = rule(
        "Write '%s%s' as one word.",
        &format!("exceptions: ['\"[^\"]*\"']\n{YOUR_SELF}"),
    );
    assert_eq!(found(&text, "He wrote \"your self\" twice.\n")?, none());
    Ok(())
}

#[test]
fn a_model_is_refused() -> TestResult {
    let message = refusal(
        "Team.Tags",
        &rule("m", "model: other\ntokens: [{pattern: your}]\n"),
    )?;
    assert!(message.contains("Team.Tags"), "{message}");
    assert!(message.contains("model"), "{message}");
    Ok(())
}

#[test]
fn tokens_with_nothing_to_start_from_are_refused() -> TestResult {
    let message = refusal(
        "Team.Words",
        &rule("m", "tokens: [{tag: NN, negate: true}]\n"),
    )?;
    assert!(message.contains("never match"), "{message}");
    Ok(())
}

#[test]
fn a_token_field_it_does_not_define_is_refused() -> TestResult {
    let message = refusal(
        "Team.Words",
        &rule("m", "tokens: [{pattern: a, tags: NN}]\n"),
    )?;
    assert!(message.contains("tags"), "{message}");
    Ok(())
}

#[test]
fn a_huge_skip_and_min_cost_nothing() -> TestResult {
    let started = Instant::now();
    let huge = rule(
        "Found.",
        "tokens: [{pattern: go, skip: 100000000, min: 1000}]\n",
    );
    assert_eq!(found(&huge, "Go go go.\n")?, none());
    let twenty = rule(
        "Found.",
        "tokens: [{pattern: go, skip: 100000000, min: 20}]\n",
    );
    assert_eq!(
        found(&twenty, &format!("{}.\n", ["go"; 20].join(" ")))?,
        [pair(&["go"; 20].join(" "), "Found.")]
    );
    assert_eq!(
        found(&twenty, &format!("{}.\n", ["go"; 19].join(" ")))?,
        none()
    );
    assert!(started.elapsed() < Duration::from_secs(10));
    Ok(())
}
