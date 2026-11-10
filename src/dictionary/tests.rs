//! Tests for the dictionary reader: the scenarios of the spelling spec that
//! need no rule, and a comparison with Vale.
//!
//! `tests/fixtures/dictionary/vale.tsv` holds a word and `1` or `0` a line:
//! whether Vale's `en_US-web` accepts it, from Vale at `66bbb85` run in Go.
//! The words are every 50th entry of the dictionary and variants of it, in
//! other cases, with an ending, with a period, hyphenated, and misspelt, and
//! hand-written numbers, units and ordinals.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test reads its dictionary with `?` and fails through an assertion."
)]

use std::collections::HashSet;
use std::error::Error;

use super::{Dictionary, DictionaryError};

type TestResult = Result<(), Box<dyn Error>>;

/// The English dictionary.
fn english() -> Result<&'static Dictionary, Box<dyn Error>> {
    Ok(Dictionary::english().map_err(ToString::to_string)?)
}

/// Whether the English dictionary accepts `word`, with no word list.
fn accepts(word: &str) -> Result<bool, Box<dyn Error>> {
    Ok(english()?.spell(word, &HashSet::new()))
}

#[test]
fn words_of_the_dictionary_and_names() -> TestResult {
    for word in ["receive", "GitHub", "mail", "color"] {
        assert!(accepts(word)?, "{word}");
    }
    Ok(())
}

#[test]
fn a_word_it_lacks_is_refused() -> TestResult {
    assert!(!accepts("blorft")?);
    Ok(())
}

#[test]
fn an_affix_flag_makes_four_forms() -> TestResult {
    let mut forms = english()?.expand("utilize", "DSG")?;
    forms.sort();
    assert_eq!(forms, ["utilize", "utilized", "utilizes", "utilizing"]);
    Ok(())
}

#[test]
fn a_flag_that_names_no_class_is_refused() -> TestResult {
    assert_eq!(
        english()?.expand("run", "\u{1}").err(),
        Some(DictionaryError::UnknownFlag {
            flag: "\u{1}".to_owned()
        })
    );
    Ok(())
}

#[test]
fn a_word_list_adds_words() -> TestResult {
    let listed = HashSet::from(["hakkiri".to_owned()]);
    assert!(english()?.spell("hakkiri", &listed));
    assert!(!english()?.spell("hakkiri", &HashSet::new()));
    Ok(())
}

#[test]
fn a_curly_apostrophe_reads_as_a_straight_one() -> TestResult {
    assert_eq!(english()?.convert("don\u{2019}t"), "don't");
    assert!(accepts("don\u{2019}t")?);
    Ok(())
}

#[test]
fn the_one_spelling_in_another_case() -> TestResult {
    assert_eq!(
        english()?.recase("github", &HashSet::new()),
        Some("GitHub".to_owned())
    );
    Ok(())
}

#[test]
fn accepts_what_vale_accepts() -> TestResult {
    let fixture = include_str!("../../tests/fixtures/dictionary/vale.tsv");
    let mut checked = 0;
    let mut wrong = Vec::new();
    for line in fixture.lines() {
        let (word, vale) = line.split_once('\t').ok_or("a line has no tab")?;
        if accepts(word)? != (vale == "1") {
            wrong.push(word);
        }
        checked += 1;
    }
    assert!(checked > 5000, "{checked} words");
    assert_eq!(wrong, Vec::<&str>::new());
    Ok(())
}

#[test]
fn a_character_set_other_than_utf8_or_latin1_is_refused() {
    let aff = b"SET KOI8-R\n";
    assert_eq!(
        Dictionary::parse(aff, b"0\n").err(),
        Some(DictionaryError::Charset {
            set: "KOI8-R".to_owned()
        })
    );
}

#[test]
fn a_latin1_dictionary_reads() -> TestResult {
    let aff = b"SET ISO8859-1\n";
    let dic = b"1\nr\xe9sum\xe9\n";
    let dictionary = Dictionary::parse(aff, dic)?;
    assert!(dictionary.spell("r\u{e9}sum\u{e9}", &HashSet::new()));
    Ok(())
}
