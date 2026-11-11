//! Tests for the string values of front matter: which scalars are text, which
//! keys have an entry, and where the text lies in the file.

use super::{json_nodes, key_of, nodes, resolves_to_text};
use crate::tree::Node;

/// The label and the text of each value that `nodes` gives for `source`, whose
/// front matter fills it.
fn values(source: &str, toml: bool) -> Vec<(String, String)> {
    let span = 0..source.trim_end().len();
    read(&nodes(source, &span, toml), source)
}

fn read(found: &[Node], source: &str) -> Vec<(String, String)> {
    found
        .iter()
        .map(|node| {
            (
                node.label.clone().unwrap_or_default(),
                node.text(source).unwrap_or_default(),
            )
        })
        .collect()
}

fn pair(key: &str, text: &str) -> (String, String) {
    (key.to_owned(), text.to_owned())
}

#[test]
fn a_scalar_that_resolves_to_a_string_is_text() {
    for text in [
        "yes", "no", "on", "off", "y", "n", "Yes", "NO", "12:30", "1,000", "=", "1.2.3", "v1.0",
        "1e", "0xZZ", "a:b", "a#b", "very", "07a",
    ] {
        assert!(resolves_to_text(text), "{text} is text");
    }
}

#[test]
fn a_scalar_that_resolves_to_something_else_is_not() {
    for text in [
        "",
        "~",
        "null",
        "Null",
        "NULL",
        "true",
        "True",
        "FALSE",
        "12",
        "-3",
        "+1",
        "07",
        "08",
        "0x1f",
        "0o17",
        "0b11",
        "1_000",
        "1.5",
        "1e3",
        "1E3",
        "0.5e-2",
        ".5",
        "5.",
        "3.",
        "+.5",
        ".inf",
        "-.inf",
        ".NaN",
        "<<",
        "2024-01-01",
        "2024-01-01T10:00:00Z",
        "2024-01-01 10:00:00",
    ] {
        assert!(!resolves_to_text(text), "{text} is not text");
    }
}

#[test]
fn an_entry_has_a_key_and_a_colon() {
    assert_eq!(key_of("title: x"), Some(("title".to_owned(), 6)));
    assert_eq!(key_of("\"a key\": x"), Some(("a key".to_owned(), 8)));
    assert_eq!(key_of("'a key': x"), Some(("a key".to_owned(), 8)));
    assert_eq!(key_of("title:"), Some(("title".to_owned(), 6)));
    assert_eq!(key_of("- item"), None);
    assert_eq!(key_of("a:b"), None);
    assert_eq!(key_of("[x]"), None);
}

#[test]
fn the_values_of_yaml_are_the_top_level_strings() {
    let source = "---\ntitle: A very title\ncount: 3\ndraft: false\ntags: [a, b]\nauthor:\n  name: very nested\nquoted: \"a very quoted\"\nsingle: 'a very single'\nplain: two # comment\n---\n";
    assert_eq!(
        values(source, false),
        [
            pair("title", "A very title"),
            pair("quoted", "a very quoted"),
            pair("single", "a very single"),
            pair("plain", "two"),
        ]
    );
}

#[test]
fn a_block_scalar_reads_each_of_its_lines() {
    let source = "---\nfolded: >-\n  one very\n  two\nliteral: |\n  three\n  four\n---\n";
    assert_eq!(
        values(source, false),
        [
            pair("folded", "one very two"),
            pair("literal", "three\nfour")
        ]
    );
}

#[test]
fn a_scalar_goes_on_over_the_lines_under_it() {
    let source = "---\nplain: one\n  two\nquoted: \"three\n  four\"\n---\n";
    assert_eq!(
        values(source, false),
        [pair("plain", "one two"), pair("quoted", "three four")]
    );
}

#[test]
fn a_value_on_the_line_after_its_key_is_read_unless_it_is_a_collection() {
    let source = "---\nplain:\n  text here\nlist:\n  - a\nmap:\n  k: v\nempty:\n---\n";
    assert_eq!(values(source, false), [pair("plain", "text here")]);
}

#[test]
fn an_explicit_string_tag_is_text_and_another_tag_is_not() {
    assert_eq!(
        values("---\na: !!str tagged\nb: !!int 3\n---\n", false),
        [pair("a", "tagged")]
    );
}

#[test]
fn the_strings_before_the_first_table_are_the_values_of_toml() {
    let source = "+++\ntitle = \"A very title\"\ncount = 3\ntags = [\"a\"]\nliteral = 'x y'\n[params]\nsub = \"in a table\"\n+++\n";
    assert_eq!(
        values(source, true),
        [pair("title", "A very title"), pair("literal", "x y")]
    );
}

#[test]
fn a_toml_string_may_go_over_lines() {
    let source = "+++\nbody = \"\"\"\none\ntwo\"\"\"\n+++\n";
    assert_eq!(values(source, true), [pair("body", "one\ntwo")]);
}

#[test]
fn the_strings_of_a_json_object_are_values_and_its_other_members_are_not() {
    let source = ";;;\n{\"title\": \"A very json\", \"n\": 3, \"tags\": [\"x\"], \"o\": {\"k\": \"y\"}, \"s\": \"z\"}\n;;;\n\nBody.\n";
    assert_eq!(
        read(&json_nodes(source), source),
        [pair("title", "A very json"), pair("s", "z")]
    );
    let none = Vec::<(String, String)>::new();
    assert_eq!(read(&json_nodes("Body.\n"), "Body.\n"), none);
    let open = ";;;\n{\"a\": \"b\"}\n";
    assert_eq!(read(&json_nodes(open), open), none);
}
