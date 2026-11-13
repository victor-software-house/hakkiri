//! Tests for the sentence splitter: one per scenario of the
//! sentence-splitting spec, run on the embedded English model, and tests of the
//! trainer and the model file on small inputs.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test reads its model with `?` and fails through an assertion."
)]

use std::error::Error;
use std::fmt::Write as _;
use std::time::Instant;

use super::{Model, train};

type TestResult = Result<(), Box<dyn Error>>;

/// The sentences of `text` under the embedded English model.
fn sentences(text: &str) -> Result<Vec<&str>, Box<dyn Error>> {
    let model = Model::english().map_err(ToString::to_string)?;
    Ok(model
        .split(text)
        .into_iter()
        .filter_map(|range| text.get(range))
        .collect())
}

#[test]
fn two_plain_sentences() -> TestResult {
    assert_eq!(
        sentences("The build passed. Ship it now!")?,
        ["The build passed.", "Ship it now!"]
    );
    Ok(())
}

#[test]
fn a_question_inside_a_paragraph() -> TestResult {
    assert_eq!(
        sentences("Does it work? Run the test.")?,
        ["Does it work?", "Run the test."]
    );
    Ok(())
}

#[test]
fn a_title_before_a_name() -> TestResult {
    assert_eq!(
        sentences("Ask Dr. Smith first. She knows.")?,
        ["Ask Dr. Smith first.", "She knows."]
    );
    Ok(())
}

#[test]
fn a_latin_abbreviation_in_the_middle() -> TestResult {
    assert_eq!(
        sentences("Use a list, e.g. a table of flags. Then stop.")?,
        ["Use a list, e.g. a table of flags.", "Then stop."]
    );
    Ok(())
}

#[test]
fn a_version_a_file_and_a_price() -> TestResult {
    assert_eq!(
        sentences("Install v1.2.3 from README.md for 3.50 dollars. Done.")?,
        ["Install v1.2.3 from README.md for 3.50 dollars.", "Done."]
    );
    Ok(())
}

#[test]
fn a_quoted_sentence() -> TestResult {
    assert_eq!(
        sentences(r#"He wrote "Stop." Then he left."#)?,
        [r#"He wrote "Stop.""#, "Then he left."]
    );
    Ok(())
}

#[test]
fn a_closing_bracket_stays_with_its_sentence() -> TestResult {
    assert_eq!(
        sentences("It failed (twice.) Then it passed.")?,
        ["It failed (twice.)", "Then it passed."]
    );
    assert_eq!(sentences("Is it done? ) Yes.")?, ["Is it done? )", "Yes."]);
    Ok(())
}

#[test]
fn a_full_sentence_in_brackets() -> TestResult {
    let text = "Run it twice (the cache needs one pass. The second is fast) and stop.";
    assert_eq!(sentences(text)?, [text]);
    Ok(())
}

#[test]
fn a_pause_inside_a_sentence() -> TestResult {
    let text = "Wait... it works now.";
    assert_eq!(sentences(text)?, [text]);
    Ok(())
}

#[test]
fn an_ellipsis_before_a_capital_may_end_a_sentence() -> TestResult {
    assert_eq!(
        sentences("The build passed... The tests did not.")?,
        ["The build passed...", "The tests did not."]
    );
    Ok(())
}

#[test]
fn text_with_no_word_has_no_sentence() -> TestResult {
    assert_eq!(sentences("")?, Vec::<&str>::new());
    assert_eq!(sentences(" \n\t ")?, Vec::<&str>::new());
    Ok(())
}

#[test]
fn ranges_have_no_outer_whitespace() -> TestResult {
    assert_eq!(
        sentences("  One two.\n\nThree four.  ")?,
        ["One two.", "Three four."]
    );
    Ok(())
}

#[test]
fn the_embedded_model_is_read_and_is_in_file_order() -> TestResult {
    let model = Model::english().map_err(ToString::to_string)?;
    assert!(
        model.abbreviations().contains("dr"),
        "the English model knows `dr`"
    );
    assert_eq!(
        model.to_tsv(),
        include_str!("../../data/splitter/en.tsv"),
        "the embedded file is the sorted text of its own model"
    );
    Ok(())
}

/// A corpus where `mr.` is always an abbreviation and `Smith` follows it.
fn corpus() -> String {
    "Mr. Smith went home. He slept well. Then Mr. Smith woke up.\n\n".repeat(200)
}

#[test]
fn training_finds_an_abbreviation_and_splits_with_it() {
    let text = corpus();
    let trained = train(&[&text]);
    assert!(trained.model.abbreviations().contains("mr"));
    let spoken = "Ask Mr. Smith now. Go.";
    let found: Vec<&str> = trained
        .model
        .split(spoken)
        .into_iter()
        .filter_map(|range| spoken.get(range))
        .collect();
    assert_eq!(found, ["Ask Mr. Smith now.", "Go."]);
}

#[test]
fn two_training_runs_write_the_same_file() {
    let text = corpus();
    let first = train(&[&text, "More text. Some more."]).model.to_tsv();
    let second = train(&[&text, "More text. Some more."]).model.to_tsv();
    assert_eq!(first, second);
}

#[test]
fn a_model_file_reads_back_as_the_model_it_came_from() -> TestResult {
    let text = corpus();
    let model = train(&[&text]).model;
    assert_eq!(Model::parse(&model.to_tsv())?, model);
    Ok(())
}

#[test]
fn a_bad_model_file_is_refused() {
    assert!(Model::parse("").is_err());
    assert!(Model::parse("hakkiri-splitter-model\t2\n").is_err());
    assert!(Model::parse("hakkiri-splitter-model\t1\nstarter\n").is_err());
    assert!(Model::parse("hakkiri-splitter-model\t1\northography\tword\t1\n").is_err());
    assert!(Model::parse("hakkiri-splitter-model\t1\nnothing\tword\n").is_err());
    assert!(Model::parse("hakkiri-splitter-model\t1\n").is_ok());
}

#[test]
fn an_empty_model_still_splits_on_plain_end_marks() {
    let model = Model::default();
    let text = "Aa bb. Cc dd? Ee ff!";
    let found: Vec<&str> = model
        .split(text)
        .into_iter()
        .filter_map(|range| text.get(range))
        .collect();
    assert_eq!(found, ["Aa bb.", "Cc dd?", "Ee ff!"]);
}

/// Sentences of every shape the rules treat: abbreviations, brackets, quotes,
/// ellipses and end marks.
const MIXED: &str =
    "Ask Dr. Smith (e.g. the first one. Or the last) now. Does it work? Wait... Yes! \"Stop.\" ";

#[test]
fn a_50000_character_paragraph_splits() -> TestResult {
    let model = Model::english().map_err(ToString::to_string)?;
    let text = MIXED.repeat(50_000 / MIXED.len() + 1);
    assert!(text.len() >= 50_000);
    let start = Instant::now();
    let found = model.split(&text);
    eprintln!(
        "{} characters, {} sentences, {:?}",
        text.len(),
        found.len(),
        start.elapsed()
    );
    assert!(found.len() > 100);
    Ok(())
}

#[test]
fn a_50000_character_run_with_no_space_is_one_sentence() -> TestResult {
    let model = Model::english().map_err(ToString::to_string)?;
    for text in [
        "a.".repeat(25_000),
        "(".repeat(50_000),
        "((a.)b".repeat(8_000),
        "...".repeat(17_000),
    ] {
        assert_eq!(model.split(&text).len(), 1);
    }
    Ok(())
}

#[test]
fn a_60000_line_input_splits() -> TestResult {
    let model = Model::english().map_err(ToString::to_string)?;
    let mut text = String::new();
    for line in 0..60_000 {
        writeln!(text, "Line {line} ends here. Does the next one?")?;
    }
    let start = Instant::now();
    let found = model.split(&text);
    eprintln!(
        "{} lines, {} sentences, {:?}",
        text.lines().count(),
        found.len(),
        start.elapsed()
    );
    assert!(found.len() >= 60_000);
    Ok(())
}

#[test]
fn a_list_number_ends_nothing() -> TestResult {
    assert_eq!(
        sentences("Steps: 1. Install it. 2. Run it.")?,
        ["Steps: 1. Install it.", "2. Run it."]
    );
    assert_eq!(
        sentences("1. First do this. 2. Then do that.")?,
        ["1. First do this.", "2. Then do that."]
    );
    Ok(())
}

#[test]
fn a_chain_of_initials_goes_on() -> TestResult {
    assert_eq!(
        sentences("J. R. R. Tolkien wrote books. Many people read them.")?,
        ["J. R. R. Tolkien wrote books.", "Many people read them."]
    );
    Ok(())
}

#[test]
fn spaced_periods_are_one_ellipsis() -> TestResult {
    let text = "Wait . . . it works now.";
    assert_eq!(sentences(text)?, [text]);
    Ok(())
}
