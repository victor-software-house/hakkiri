//! Tests for the tagger: the scenarios of the sequence and capitalization
//! specs that read a tag, the corrections made after the model, and a
//! comparison of the model with prose.
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

use super::tokens::{CONTRACTIONS, Contractions, is_emoticon, is_special};
use super::{FormatError, Tagger, treebank};

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
fn the_model_tags_equal_prose_on_its_sentences() -> TestResult {
    let tagger = english()?;
    let fixture = include_str!("../../tests/fixtures/tagger/prose.tsv");
    let mut checked = 0;
    for line in fixture.lines() {
        let (sentence, expected) = line.split_once('\t').ok_or("a line has no tab")?;
        let ranges = super::tokens::tokens(sentence);
        let words: Vec<&str> = ranges
            .iter()
            .map(|range| sentence.get(range.clone()).unwrap_or_default())
            .collect();
        let found: Vec<String> = ranges
            .iter()
            .zip(tagger.model_tags(&words))
            .map(|(range, tag)| format!("{}:{}:{}", range.start, range.end, tag))
            .collect();
        assert_eq!(found.join(" "), expected.trim(), "{sentence}");
        checked += 1;
    }
    assert_eq!(checked, 188);
    Ok(())
}

/// The tag `tag()` gives the token `word` of `sentence`.
fn tag_of(sentence: &str, word: &str) -> Result<String, Box<dyn Error>> {
    let wanted = format!("{word}/");
    pairs(sentence)?
        .into_iter()
        .find_map(|pair| pair.strip_prefix(&wanted).map(str::to_owned))
        .ok_or_else(|| format!("{word} is no token of {sentence}").into())
}

#[test]
fn a_modal_in_capitals_is_a_modal() -> TestResult {
    for modal in ["SHALL", "MUST", "MAY", "SHOULD", "WILL", "CAN"] {
        let sentence = format!("The loop {modal} keep its context.");
        assert_eq!(tag_of(&sentence, modal)?, "MD", "{sentence}");
    }
    Ok(())
}

#[test]
fn a_word_in_capitals_is_a_proper_noun() -> TestResult {
    let sentence = "The CSS file reads the SSH keys at UTC.";
    for word in ["CSS", "SSH", "UTC"] {
        assert_eq!(tag_of(sentence, word)?, "NNP", "{word}");
    }
    Ok(())
}

#[test]
fn a_function_word_or_a_unit_in_capitals_keeps_its_tag() -> TestResult {
    let tagger = english()?;
    for (sentence, word) in [
        ("The file MUST NOT be read.", "NOT"),
        ("THE ANSWER AND THE QUESTION.", "AND"),
        ("Read the first 10 MB of the file.", "MB"),
    ] {
        let ranges = super::tokens::tokens(sentence);
        let words: Vec<&str> = ranges
            .iter()
            .map(|range| sentence.get(range.clone()).unwrap_or_default())
            .collect();
        let model = tagger.model_tags(&words);
        let index = words.iter().position(|each| *each == word).ok_or(word)?;
        assert_eq!(
            tag_of(sentence, word)?,
            model.get(index).copied().unwrap_or_default()
        );
        assert_ne!(tag_of(sentence, word)?, "NNP", "{word}");
    }
    Ok(())
}

#[test]
fn a_letter_in_capitals_and_a_word_in_mixed_case_keep_the_model_tag() -> TestResult {
    let tagger = english()?;
    for sentence in ["Open the Css file as A.", "Read the file in the repo."] {
        let ranges = super::tokens::tokens(sentence);
        let words: Vec<&str> = ranges
            .iter()
            .map(|range| sentence.get(range.clone()).unwrap_or_default())
            .collect();
        assert_eq!(
            tagger.tag_words(&words),
            tagger.model_tags(&words),
            "{sentence}"
        );
    }
    Ok(())
}

#[test]
fn the_possessive_after_a_noun_is_a_possessive() -> TestResult {
    for sentence in [
        "The checkpoint's pages per window show that cost.",
        "Copy the session's subagent and advisor.",
        "Read the operator\u{2019}s own plan.",
        "Each flow's pid is in the log.",
    ] {
        let ending = if sentence.contains('\u{2019}') {
            "\u{2019}s"
        } else {
            "'s"
        };
        assert_eq!(tag_of(sentence, ending)?, "POS", "{sentence}");
    }
    Ok(())
}

#[test]
fn the_possessive_after_a_proper_noun_in_capitals_is_a_possessive() -> TestResult {
    assert_eq!(tag_of("The SSH's keys are old.", "'s")?, "POS");
    Ok(())
}

#[test]
fn the_ending_of_a_pronoun_is_not_a_possessive() -> TestResult {
    assert_ne!(tag_of("It's fine.", "'s")?, "POS");
    Ok(())
}

#[test]
fn a_semicolon_is_a_colon_tag() -> TestResult {
    assert_eq!(
        tag_of("The file holds rows; each row has a key.", ";")?,
        ":"
    );
    Ok(())
}

#[test]
fn a_plural_noun_after_a_noun_phrase_is_the_verb() -> TestResult {
    for (sentence, verb) in [
        ("The freshness harness reads stdout.", "reads"),
        ("Each entry names its line.", "names"),
        ("A role owns one coherent host.", "owns"),
        ("A request line names its repository first.", "names"),
        ("The client draws the count of new rows.", "draws"),
        ("The log holds no such line.", "holds"),
    ] {
        assert_eq!(tag_of(sentence, verb)?, "VBZ", "{sentence}");
    }
    Ok(())
}

#[test]
fn a_preposition_after_a_plural_noun_makes_it_a_verb_only_in_a_clause_with_no_other_verb()
-> TestResult {
    assert_eq!(tag_of("The store parses from the disk.", "parses")?, "VBZ");
    assert_ne!(
        tag_of("The layout rules of the widgets are short.", "rules")?,
        "VBZ"
    );
    assert_ne!(tag_of("The layout rules of the widgets.", "rules")?, "VBZ");
    Ok(())
}

#[test]
fn only_the_first_plural_noun_of_a_clause_with_no_verb_becomes_one() -> TestResult {
    let sentence = "The store parses user files from the disk.";
    assert_eq!(tag_of(sentence, "parses")?, "VBZ");
    assert_eq!(tag_of(sentence, "files")?, "NNS");
    Ok(())
}

#[test]
fn a_plural_noun_in_a_compound_or_after_a_possessive_stays_a_noun() -> TestResult {
    assert_eq!(
        tag_of("It reads the session notes and the user files.", "files")?,
        "NNS"
    );
    assert_eq!(
        tag_of("A member's features unify across a workspace.", "features")?,
        "NNS"
    );
    Ok(())
}

#[test]
fn a_verb_of_saying_after_a_noun_is_a_verb() -> TestResult {
    assert_eq!(tag_of("The docs say so.", "say")?, "VBP");
    assert_eq!(
        tag_of("The CI builds agree with each other.", "agree")?,
        "VB"
    );
    assert_eq!(
        tag_of("The CI builds agree with each other.", "builds")?,
        "NNS"
    );
    assert_eq!(tag_of("The report agree with the log.", "agree")?, "VBP");
    assert_eq!(tag_of("The report agree with the log.", "report")?, "NN");
    assert_eq!(tag_of("The state says it is not done.", "says")?, "VBZ");
    Ok(())
}

#[test]
fn a_noun_after_a_determiner_is_no_verb_of_saying() -> TestResult {
    assert_eq!(tag_of("They have a say in it.", "say")?, "NN");
    Ok(())
}

#[test]
fn a_base_form_after_a_possessive_is_a_noun() -> TestResult {
    assert_eq!(
        tag_of("The client's resolver asks the server's cache.", "resolver")?,
        "NN"
    );
    assert_eq!(
        tag_of("The client's resolver asks the server's cache.", "asks")?,
        "VBZ"
    );
    assert_eq!(tag_of("The tool's own timing shows a size.", "own")?, "JJ");
    assert_ne!(tag_of("No, let's stay longer.", "stay")?, "NN");
    Ok(())
}

#[test]
fn a_lowercase_name_that_starts_a_sentence_before_a_verb_of_asking_is_a_noun() -> TestResult {
    assert_eq!(tag_of("cargo asks again with a header.", "cargo")?, "NN");
    assert_eq!(tag_of("git asks for a name.", "asks")?, "VBZ");
    assert_eq!(tag_of("Run tests for the name.", "Run")?, "VB");
    Ok(())
}

#[test]
fn a_plural_noun_at_the_start_before_a_determiner_is_the_verb() -> TestResult {
    assert_eq!(tag_of("Takes every waiting item.", "Takes")?, "VBZ");
    assert_eq!(tag_of("Cats that spend time sleep.", "Cats")?, "NNS");
    Ok(())
}

#[test]
fn asks_is_a_noun_after_a_preposition_a_gerund_or_an_imperative() -> TestResult {
    for sentence in [
        "Compact the idle session for the name asks that wait.",
        "Answering the name asks that wait.",
        "Answers the name asks it leaves, which no run will answer.",
        "The person's text from the name asks it answers, joined.",
        "Each ask waits.",
    ] {
        let tag = tag_of(
            sentence,
            if sentence.contains("Each") {
                "ask"
            } else {
                "asks"
            },
        )?;
        assert!(tag.starts_with("NN"), "{sentence}: {tag}");
    }
    Ok(())
}

#[test]
fn asks_is_a_verb_in_the_clause_of_its_subject() -> TestResult {
    for sentence in [
        "A request asks for the rows.",
        "As the name asks, the file is read.",
        "The name asks for.",
        "Starts the compaction name asks for, unless the connection holds one.",
        "The test asks whether the file exists.",
    ] {
        assert_eq!(tag_of(sentence, "asks")?, "VBZ", "{sentence}");
    }
    Ok(())
}

#[test]
fn asked_after_a_noun_phrase_is_a_participle_or_a_past_tense_by_its_place() -> TestResult {
    assert_eq!(
        tag_of(
            "It is stated as the spec says, with the cache hook told of the new file.",
            "told"
        )?,
        "VBN"
    );
    assert_eq!(
        tag_of("How long a batch a request asked for waits.", "asked")?,
        "VBD"
    );
    assert_eq!(
        tag_of(
            "The offset of the one asked for, or the first byte.",
            "asked"
        )?,
        "VBN"
    );
    Ok(())
}

#[test]
fn pull_before_request_is_a_noun() -> TestResult {
    assert_eq!(
        tag_of("The queue load split pull request ended it.", "pull")?,
        "NN"
    );
    Ok(())
}

#[test]
fn a_technical_noun_in_a_pull_request_title_is_a_noun() -> TestResult {
    for (sentence, word) in [
        ("The queue load split pull request ended it.", "split"),
        ("The final mask round pull request fixed it.", "round"),
        ("Before the fade removal pull request removed it.", "fade"),
        ("It is merged (the prompt template pull request).", "prompt"),
    ] {
        assert_eq!(tag_of(sentence, word)?, "NN", "{sentence}");
    }
    Ok(())
}

#[test]
fn a_technical_noun_after_a_determiner_or_a_compound_is_a_noun() -> TestResult {
    for (sentence, word) in [
        (
            "The patch and its notes stay as the thinking mode fields decision set them.",
            "thinking",
        ),
        ("Review the peer user id check against real files.", "check"),
    ] {
        assert_eq!(tag_of(sentence, word)?, "NN", "{sentence}");
    }
    Ok(())
}

#[test]
fn a_technical_word_that_is_a_verb_stays_a_verb() -> TestResult {
    for (sentence, word, tag) in [
        ("The tests check the cache against the spec.", "check", "VB"),
        ("The gas stations had repair shops in town.", "had", "VBD"),
        ("The final mask round pull request fixed it.", "final", "JJ"),
    ] {
        assert_eq!(tag_of(sentence, word)?, tag, "{sentence}");
    }
    Ok(())
}

#[test]
fn a_title_is_tagged_as_the_model_tags_it() -> TestResult {
    let tagger = english()?;
    let sentence = "The SSH keys; the user's files MUST stay.";
    let words = treebank::tokenize(sentence);
    let refs: Vec<&str> = words.iter().map(String::as_str).collect();
    assert_eq!(tagger.tag_treebank(sentence), tagger.model_tags(&refs));
    assert_ne!(tagger.tag_words(&refs), tagger.model_tags(&refs));
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
fn a_word_count_beyond_the_file_is_refused() {
    // A model with one class and no features, and then a count of four
    // billion words that the file does not hold.
    let model = [
        &b"PROSETAG"[..],
        &2_u32.to_le_bytes(),
        &1_u32.to_le_bytes(),
        &1_u16.to_le_bytes(),
        b"X",
        &0_u32.to_le_bytes(),
        &0_u32.to_le_bytes(),
        &0_u32.to_le_bytes(),
        &0_u32.to_le_bytes(),
        &0_u32.to_le_bytes(),
        &u32::MAX.to_le_bytes(),
    ]
    .concat();
    assert!(matches!(
        Tagger::from_bytes(&model),
        Err(FormatError::Truncated { .. })
    ));
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

/// Fails when tagging the sentence that `make` builds from 20,000 repeats
/// takes over 30 times as long as from 2,000.
fn assert_linear(what: &str, make: impl Fn(usize) -> String) -> TestResult {
    let tagger = english()?;
    // The best of three runs of each, so a pause on a shared machine does not
    // fail correct code.
    let best = |repeats: usize| {
        let sentence = make(repeats);
        let mut best = Duration::MAX;
        for _ in 0..3 {
            let start = Instant::now();
            let found = tagger.tag(&sentence);
            best = best.min(start.elapsed());
            assert!(found.len() > 1, "{} tokens", found.len());
        }
        best
    };
    let (small, large) = (best(2_000), best(20_000));
    eprintln!("{what}: 2,000 took {small:?}, 20,000 took {large:?}");
    // Ten times the input costs ten times as much; 30 leaves room for noise,
    // and a quadratic loop costs 100.
    assert!(
        large < small.max(Duration::from_micros(500)) * 30,
        "{what}: 20,000 took {large:?}, 2,000 took {small:?}"
    );
    Ok(())
}

#[test]
fn a_long_run_of_one_mark_tags_in_linear_time() -> TestResult {
    for mark in [")", "."] {
        assert_linear(mark, |n| format!("word{}", mark.repeat(n)))?;
    }
    Ok(())
}

#[test]
fn a_long_run_of_apostrophes_tags_in_linear_time() -> TestResult {
    assert_linear("apostrophes", |n| format!("a{}", "'".repeat(n)))
}

#[test]
fn many_contractions_in_one_token_tag_in_linear_time() -> TestResult {
    // Each ending splits off, and then the other one does: after a split the
    // ending it made is the first one at `from`, so the same ending cannot
    // split twice in a row.
    assert_linear("contractions", |n| format!("a{}", "'s'm".repeat(n / 4)))
}

#[test]
fn initials_followed_by_a_long_run_of_marks_tag_in_linear_time() -> TestResult {
    assert_linear("initials and marks", |n| {
        format!("{}{}", "a.".repeat(n / 2), ")".repeat(n / 2))
    })
}

#[test]
fn initials_and_titles_with_marks_in_between_tag_in_linear_time() -> TestResult {
    assert_linear("initials, periods and marks", |n| {
        format!("{}{}", "a.".repeat(n / 2), ".)".repeat(n / 4))
    })
}

#[test]
fn the_special_check_equals_the_pattern_it_replaces() -> TestResult {
    let pattern = regex::Regex::new(r"^(?:[A-Za-z]\.){2,}$|^[A-Z][a-z]{1,2}\.$")?;
    let alphabet = ['a', 'Z', '.', ')', '\u{e9}'];
    let mut texts = vec![String::new()];
    let mut checked = 0;
    for _ in 0..6 {
        let longer: Vec<String> = texts
            .iter()
            .flat_map(|text| alphabet.iter().map(move |c| format!("{text}{c}")))
            .collect();
        for text in &longer {
            for from in (0..=text.len()).filter(|at| text.is_char_boundary(*at)) {
                let mut initials = None;
                for to in (from..=text.len()).filter(|at| text.is_char_boundary(*at)) {
                    let window = text.get(from..to).unwrap_or_default();
                    assert_eq!(
                        is_special(text, from, to, &mut initials),
                        is_emoticon(window) || pattern.is_match(window),
                        "{window:?} at {from}..{to} of {text:?}"
                    );
                    checked += 1;
                }
            }
        }
        texts = longer;
    }
    assert!(checked > 100_000, "{checked} windows");
    Ok(())
}

/// The earliest ending in `text[from..to]` as a search of that window finds
/// it: each ending's first place in the window, when it is not at `from`.
fn earliest_in_window(text: &str, from: usize, to: usize) -> Option<usize> {
    let window = text.get(from..to)?;
    CONTRACTIONS
        .iter()
        .filter_map(|ending| {
            let last = window.len().checked_sub(ending.len())?;
            (0..=last).find(|at| {
                window
                    .as_bytes()
                    .get(*at..*at + ending.len())
                    .is_some_and(|found| found.eq_ignore_ascii_case(ending.as_bytes()))
            })
        })
        .filter(|at| *at > 0)
        .min()
}

#[test]
fn the_contractions_found_once_equal_a_search_of_each_window() {
    let alphabet = ['n', 'N', '\'', '\u{2019}', 't', 's', 'l'];
    let mut texts = vec![String::new()];
    let mut checked = 0;
    for _ in 0..5 {
        let longer: Vec<String> = texts
            .iter()
            .flat_map(|text| alphabet.iter().map(move |c| format!("{text}{c}")))
            .collect();
        for text in &longer {
            let mut contractions = Contractions::new(text, 0..text.len());
            for from in (0..=text.len()).filter(|at| text.is_char_boundary(*at)) {
                for to in (from..=text.len()).filter(|at| text.is_char_boundary(*at)) {
                    assert_eq!(
                        contractions.earliest(from, to),
                        earliest_in_window(text, from, to),
                        "{from}..{to} of {text:?}"
                    );
                    checked += 1;
                }
            }
        }
        texts = longer;
    }
    assert!(checked > 100_000, "{checked} windows");
}

#[test]
fn the_treebank_words_equal_prose() -> TestResult {
    // `tests/fixtures/tagger/treebank.tsv` holds a text and its words, joined
    // by spaces, from `NewTreebankWordTokenizer` of prose `b33e60d` run in Go.
    // A line break in a text is written `\n` or `\r`.
    let fixture = include_str!("../../tests/fixtures/tagger/treebank.tsv");
    let mut wrong = Vec::new();
    let mut checked = 0;
    for line in fixture.lines() {
        let (text, words) = line.split_once('\t').ok_or("a line has no tab")?;
        let text = text.replace("\\n", "\n").replace("\\r", "\r");
        let found = treebank::tokenize(&text).join(" ");
        if found != words {
            wrong.push(format!("{text:?} gave {found:?}, not {words:?}"));
        }
        checked += 1;
    }
    assert!(checked > 600, "{checked} texts");
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
