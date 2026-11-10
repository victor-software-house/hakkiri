//! Tests for the tagger: the scenarios of the sequence and capitalization
//! specs that read a tag, and a comparison with prose.
//!
//! `tests/fixtures/tagger/prose.tsv` holds one sentence a line, then the tokens
//! and tags that prose `b33e60d` gives it, as `start:end:tag` in bytes.
//! The sentences are the hand-written cases of its tokenizer and tagger tests,
//! and every 20th sentence of "The Adventures of Sherlock Holmes", a public
//! domain text.
//! The tags come from `tag.New()` and `tokenize.New()` of that commit, run in
//! Go.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test reads its tagger with `?` and fails through an assertion."
)]

use std::error::Error;
use std::time::{Duration, Instant};

use super::{FormatError, Tagger};

type TestResult = Result<(), Box<dyn Error>>;

/// The tagger of the embedded model.
fn english() -> Result<&'static Tagger, Box<dyn Error>> {
    Ok(Tagger::english().map_err(ToString::to_string)?)
}

/// The tokens of `sentence` with their tags, as `token/TAG` pairs.
fn pairs(sentence: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(english()?
        .tag(sentence)
        .into_iter()
        .map(|tagged| {
            format!(
                "{}/{}",
                sentence.get(tagged.range).unwrap_or_default(),
                tagged.tag
            )
        })
        .collect())
}

#[test]
fn can_is_a_modal_in_a_passive() -> TestResult {
    assert_eq!(
        pairs("The file can be read.")?,
        ["The/DT", "file/NN", "can/MD", "be/VB", "read/VBN", "./."]
    );
    Ok(())
}

#[test]
fn file_is_a_noun_after_an_imperative() -> TestResult {
    assert_eq!(
        pairs("Open the file.")?,
        ["Open/VB", "the/DT", "file/NN", "./."]
    );
    Ok(())
}

#[test]
fn contractions_curly_quotes_and_marks_split_off() -> TestResult {
    let cases: [(&str, &[&str]); 6] = [
        (
            "They'll save and invest more.",
            &["They", "'ll", "save", "and", "invest", "more", "."],
        ),
        ("Don't stop.", &["Do", "n't", "stop", "."]),
        ("$100 well spent.", &["$", "100", "well", "spent", "."]),
        ("Well) done", &["Well", ")", "done"]),
        (
            "He said \u{201c}hi\u{201d} now.",
            &["He", "said", "\u{201c}", "hi", "\u{201d}", "now", "."],
        ),
        ("It\u{2019}s fine.", &["It", "\u{2019}s", "fine", "."]),
    ];
    for (sentence, expected) in cases {
        let tagger = english()?;
        let found: Vec<&str> = tagger
            .tag(sentence)
            .into_iter()
            .filter_map(|tagged| sentence.get(tagged.range))
            .collect();
        assert_eq!(found, expected, "{sentence}");
    }
    Ok(())
}

#[test]
fn a_sentence_with_no_word_has_no_token() -> TestResult {
    let tagger = english()?;
    assert_eq!(tagger.tag(""), Vec::new());
    assert_eq!(tagger.tag("   "), Vec::new());
    Ok(())
}

#[test]
fn tags_equal_prose_on_its_sentences() -> TestResult {
    let tagger = english()?;
    let fixture = include_str!("../../tests/fixtures/tagger/prose.tsv");
    let mut checked = 0;
    for line in fixture.lines() {
        let (sentence, expected) = line.split_once('\t').ok_or("a line has no tab")?;
        let found: Vec<String> = tagger
            .tag(sentence)
            .into_iter()
            .map(|tagged| format!("{}:{}:{}", tagged.range.start, tagged.range.end, tagged.tag))
            .collect();
        assert_eq!(found.join(" "), expected.trim(), "{sentence}");
        checked += 1;
    }
    assert_eq!(checked, 188);
    Ok(())
}

#[test]
fn a_file_that_is_not_a_model_is_refused() {
    assert_eq!(
        Tagger::from_bytes(b"NOTAMODEL").err(),
        Some(FormatError::Magic)
    );
}

#[test]
fn a_file_cut_short_is_refused() -> TestResult {
    let embedded = include_bytes!("../../data/tagger/en.bin");
    let cut = embedded.get(..100).ok_or("the model is under 100 bytes")?;
    assert!(matches!(
        Tagger::from_bytes(cut),
        Err(FormatError::Truncated { .. })
    ));
    Ok(())
}

#[test]
fn a_long_run_of_marks_splits_one_token_each() -> TestResult {
    let sentence = format!("word{}", ")".repeat(20_000));
    assert_eq!(english()?.tag(&sentence).len(), 20_001);
    Ok(())
}

#[test]
fn a_contraction_after_a_long_run_still_splits() -> TestResult {
    let sentence = format!("{}don't", "(".repeat(5_000));
    let found = pairs(&sentence)?;
    let tail = found
        .get(found.len().saturating_sub(2)..)
        .unwrap_or_default();
    assert_eq!(tail.len(), 2);
    assert!(
        tail.iter()
            .zip(["do/", "n't/"])
            .all(|(token, start)| token.starts_with(start)),
        "{tail:?}"
    );
    Ok(())
}

#[test]
fn a_long_run_of_one_mark_tags_in_linear_time() -> TestResult {
    let tagger = english()?;
    // The best of three runs of each, so a pause on a shared machine does not
    // fail correct code.
    let best = |sentence: &str| {
        let mut best = Duration::MAX;
        for _ in 0..3 {
            let start = Instant::now();
            let found = tagger.tag(sentence);
            best = best.min(start.elapsed());
            assert!(found.len() > 1, "{} tokens", found.len());
        }
        best
    };
    for mark in [")", "."] {
        let small = best(&format!("word{}", mark.repeat(2_000)));
        let large = best(&format!("word{}", mark.repeat(20_000)));
        eprintln!("{mark} run: 2,000 took {small:?}, 20,000 took {large:?}");
        // Ten times the marks cost ten times as much; 30 leaves room for
        // noise, and a quadratic loop costs 100.
        assert!(
            large < small.max(Duration::from_micros(500)) * 30,
            "20,000 marks took {large:?}, 2,000 took {small:?}"
        );
    }
    Ok(())
}
