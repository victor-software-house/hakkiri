//! Tests for the character categories of the `spelling` check.
//!
//! `is` tests an ASCII character by its byte and any other through a regex
//! class, so one test holds the byte tests to the classes for every character.

use super::{Category, LETTER, LOWER, UPPER, is};

#[test]
fn a_character_is_in_a_category_as_its_regex_class_says() {
    let mut differing = Vec::new();
    for c in '\0'..=char::MAX {
        for (category, class) in [
            (Category::Letter, &LETTER),
            (Category::Upper, &UPPER),
            (Category::Lower, &LOWER),
        ] {
            if is(category, c) != class.is_match(c.encode_utf8(&mut [0; 4])) {
                differing.push(c);
            }
        }
    }
    assert!(
        differing.is_empty(),
        "these characters differ from their regex class: {differing:?}"
    );
}
