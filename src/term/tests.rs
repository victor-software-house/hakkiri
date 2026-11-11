//! Tests for the term shown in the case of the text it replaced: the cases of
//! `recaseToTerm` in Vale.

use super::recase_to_term;

#[test]
fn a_plain_term_is_shown_as_it_is() {
    assert_eq!(recase_to_term("GitHub", "github"), "GitHub");
    assert_eq!(recase_to_term("Node.js", "NODE.JS"), "Node.js");
}

#[test]
fn a_pattern_is_shown_as_the_spelling_the_text_matches() {
    assert_eq!(recase_to_term("harness(?:es)?", "Harnesses"), "harnesses");
    assert_eq!(recase_to_term("harness(?:es)?", "HARNESS"), "harness");
    assert_eq!(recase_to_term("colou?r", "COLOR"), "color");
    assert_eq!(recase_to_term("colou?r", "Colour"), "colour");
    assert_eq!(recase_to_term("OAuth2?", "Oauth"), "OAuth");
}

#[test]
fn a_phrase_that_wraps_counts_as_one_space() {
    assert_eq!(recase_to_term("mea culpa", "mea\nCulpa"), "mea culpa");
}

#[test]
fn a_pattern_no_finite_set_stands_for_is_shown_as_written() {
    assert_eq!(recase_to_term("foo.*bar", "FOO1BAR"), "foo.*bar");
    assert_eq!(recase_to_term("data\\Wset", "dataxset"), "data\\Wset");
    assert_eq!(recase_to_term("[pP]y.*\\b", "PYTHON"), "[pP]y.*\\b");
}

#[test]
fn more_than_sixty_four_spellings_fall_back_to_the_literal_characters() {
    assert_eq!(recase_to_term("a?b?c?d?e?f?g?", "ABCDEFG"), "abcdefg");
}
