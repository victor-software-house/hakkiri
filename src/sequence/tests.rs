//! Tests for the sequence walk: a comparison with Vale.
//!
//! `tests/fixtures/sequence/rules` holds one rule file for each shape of
//! token the check reads, and `inputs.txt` the texts to read: hand-written
//! headings and sentences, and every third line of "The Adventures of Sherlock
//! Holmes", a public domain text. `vale.tsv` holds a rule name, a text, the
//! text of one alert and its message, for each alert Vale gave, in order. The
//! rows come from Vale at `66bbb85` run in Go on each text as one sentence,
//! with prose `b33e60d` tagging it.
//! The walk reads the tags of the model alone here, since the corrections that
//! hakkiri makes after the model would change the tags that Vale walked.
//!
//! Vale cuts each text again with prose's segmenter and tags each part apart,
//! where hakkiri tags the sentence its own splitter gave whole. `inputs.txt`
//! holds the texts that the segmenter reads as one sentence, so both tag the
//! same words.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test reads its rules with `?` and fails through an assertion."
)]

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use super::find_tagged;
use crate::alert::fill;
use crate::rule::{Check, Rule};
use crate::tagger::Tagger;

type TestResult = Result<(), Box<dyn Error>>;

/// The alerts of the rule file `text` on `sentence`, as text and message.
fn alerts(name: &str, text: &str, sentence: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let rule = Rule::from_text(name, &format!("{name}.yml"), text)?;
    let Check::Sequence(check) = &rule.check else {
        return Err(format!("{name} is not a sequence rule").into());
    };
    let tagged = Tagger::english()
        .map_err(ToString::to_string)?
        .tag_model(sentence);
    Ok(find_tagged(check, sentence, tagged, |_| false)?
        .into_iter()
        .map(|matched| {
            let values: Vec<&str> = matched.words.iter().map(String::as_str).collect();
            (
                sentence.get(matched.range).unwrap_or_default().to_owned(),
                fill(&rule.message, &values),
            )
        })
        .collect())
}

/// The texts of the fixture `directory` that have an alert, and what differs
/// from Vale's rows in it.
fn compare(directory: &str) -> Result<(usize, Vec<String>), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(directory);
    let inputs = fs::read_to_string(root.join("inputs.txt"))?;
    let fixture = fs::read_to_string(root.join("vale.tsv"))?;
    let mut expected: HashMap<(&str, &str), Vec<(String, String)>> = HashMap::new();
    for line in fixture.lines() {
        let columns: Vec<&str> = line.split('\t').collect();
        let [name, text, matched, message] = columns.as_slice() else {
            return Err(format!("a line has not four columns: {line}").into());
        };
        expected
            .entry((name, text))
            .or_default()
            .push(((*matched).to_owned(), (*message).to_owned()));
    }
    let mut files: Vec<_> = fs::read_dir(root.join("rules"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    files.sort();
    let mut wrong = Vec::new();
    let mut with_alerts = 0;
    for file in &files {
        let stem = file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or("a rule file has no name")?;
        let name = format!("Seq.{stem}");
        let text = fs::read_to_string(file)?;
        for sentence in inputs.lines() {
            let want = expected
                .get(&(name.as_str(), sentence))
                .cloned()
                .unwrap_or_default();
            let got = alerts(&name, &text, sentence)?;
            with_alerts += usize::from(!want.is_empty());
            if got != want {
                wrong.push(format!("{name}: {sentence:?} gave {got:?}, Vale {want:?}"));
            }
        }
    }
    Ok((with_alerts, wrong))
}

#[test]
fn walks_the_tokens_as_vale_does() -> TestResult {
    let (with_alerts, wrong) = compare("sequence")?;
    assert!(with_alerts > 150, "{with_alerts} texts with an alert");
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// The rules of `tests/fixtures/sequence-verbs` fill indexed verbs in the
/// message: two `ProseStyle` rules from v0.0.14, and three that take the values
/// out of order, after an index and mixed with plain `%s`. Vale gives each
/// message through Go's `Sprintf`, so the rows are the messages it prints.
#[test]
fn fills_indexed_verbs_as_vale_does() -> TestResult {
    let (with_alerts, wrong) = compare("sequence-verbs")?;
    assert_eq!(with_alerts, 9, "{with_alerts} texts with an alert");
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// A rule that reads four nouns in a row, as `NounStack` does.
const FOUR_NOUNS: &str = "extends: sequence\nmessage: \"Noun stack '%s'.\"\ntokens:\n  - tag: NN|NNS\n    pattern: '[a-z][a-z-]*'\n    min: 4\n";

#[test]
fn a_verb_after_a_noun_phrase_ends_a_noun_stack() -> TestResult {
    let rule = Rule::from_text("Seq.FourNouns", "FourNouns.yml", FOUR_NOUNS)?;
    let Check::Sequence(check) = &rule.check else {
        return Err("FourNouns is not a sequence rule".into());
    };
    let sentence = "The freshness harness reads stdout.";
    let tagger = Tagger::english().map_err(ToString::to_string)?;
    let model = tagger.tag_model(sentence);
    assert_eq!(find_tagged(check, sentence, model, |_| false)?.len(), 1);
    let corrected = tagger.tag(sentence);
    assert_eq!(find_tagged(check, sentence, corrected, |_| false)?, []);
    Ok(())
}

/// A rule that gives a noun the verb `says`, as `ThingsSpeakNoun` does.
const NOUN_SAYS: &str = "extends: sequence\nmessage: \"'%s %s' speaks.\"\ntokens:\n  - tag: NN|NNS\n  - pattern: says\n";

/// A rule with a negated step: `files` that no noun follows.
const FILES_NO_NOUN: &str = "extends: sequence\nmessage: \"'%s' ends.\"\ntokens:\n  - pattern: files\n  - tag: NN\n    negate: true\n";

/// The matches of the rule `text` in `sentence`, where the bytes `code` read a
/// code span.
fn matches_with_code(
    text: &str,
    sentence: &str,
    code: std::ops::Range<usize>,
) -> Result<Vec<String>, Box<dyn Error>> {
    let rule = Rule::from_text("Seq.Rule", "Rule.yml", text)?;
    let Check::Sequence(check) = &rule.check else {
        return Err("Rule is not a sequence rule".into());
    };
    let tagged = Tagger::english()
        .map_err(ToString::to_string)?
        .tag(sentence);
    Ok(find_tagged(check, sentence, tagged, |range| {
        range.start < code.end && code.start < range.end
    })?
    .into_iter()
    .map(|matched| sentence.get(matched.range).unwrap_or_default().to_owned())
    .collect())
}

#[test]
fn a_code_span_is_no_noun_of_a_verb() -> TestResult {
    let sentence = "The call fails as the flag says.";
    let at = sentence.find("flag says").ok_or("no code span")?;
    assert_eq!(
        matches_with_code(NOUN_SAYS, sentence, at..at + "flag".len())?,
        Vec::<String>::new()
    );
    assert_eq!(matches_with_code(NOUN_SAYS, sentence, 0..0)?, ["flag says"]);
    Ok(())
}

#[test]
fn a_negated_token_holds_at_a_code_span() -> TestResult {
    let sentence = "Copy the files data here.";
    let at = sentence.find("data here").ok_or("no code span")?;
    assert_eq!(
        matches_with_code(FILES_NO_NOUN, sentence, 0..0)?,
        Vec::<String>::new()
    );
    assert_eq!(
        matches_with_code(FILES_NO_NOUN, sentence, at..at + "data".len())?,
        ["files data"]
    );
    Ok(())
}
