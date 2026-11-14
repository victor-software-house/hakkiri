//! Tests for the string values of front matter: which entries are text, and
//! where the text lies in the file.

use super::{blank, fault, leading, span};
use crate::markdown::{Flavor, build};
use crate::tree::{Node, Scope};

/// The label and the text of each value that `leading` gives for `source`.
fn values(source: &str) -> Vec<(String, String)> {
    read(&leading(source), source)
}

fn read<'a>(found: impl IntoIterator<Item = &'a Node>, source: &str) -> Vec<(String, String)> {
    found
        .into_iter()
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
fn the_values_of_yaml_are_the_top_level_strings() {
    let source = "---\ntitle: A very title\ncount: 3\ndraft: false\ntags: [a, b]\nauthor:\n  name: very nested\nquoted: \"a very quoted\"\nsingle: 'a very single'\nplain: two # comment\n---\n";
    assert_eq!(
        values(source),
        [
            pair("title", "A very title"),
            pair("quoted", "a very quoted"),
            pair("single", "a very single"),
            pair("plain", "two"),
        ]
    );
}

#[test]
fn a_scalar_that_resolves_to_a_string_is_text() {
    let source =
        "---\na: yes\nb: no\nc: on\nd: off\ne: 12:30\nf: 1,000\ng: v1.0\nh: 1.2.3\ni: a:b\n---\n";
    let keys: Vec<_> = values(source).into_iter().map(|(key, _)| key).collect();
    assert_eq!(keys, ["a", "b", "c", "d", "e", "f", "g", "h", "i"]);
}

#[test]
fn a_block_scalar_reads_each_of_its_lines() {
    let source = "---\nfolded: >-\n  one very\n  two\nliteral: |\n  three\n  four\n---\n";
    assert_eq!(
        values(source),
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
        values(source),
        [pair("plain", "one two"), pair("quoted", "three four")]
    );
}

#[test]
fn a_value_on_the_line_after_its_key_is_read_unless_it_is_a_collection() {
    let source = "---\nplain:\n  text here\nlist:\n  - a\nmap:\n  k: v\nempty:\n---\n";
    assert_eq!(values(source), [pair("plain", "text here")]);
}

#[test]
fn an_explicit_string_tag_is_text_and_another_tag_is_not() {
    assert_eq!(
        values("---\na: !!str tagged\nb: !!int 3\n---\n"),
        [pair("a", "tagged")]
    );
}

#[test]
fn a_date_and_a_merge_key_are_no_text_unless_tagged_or_quoted() {
    let source = "---\na: 2024-01-01\nb: 2024-01-01 10:00:00\nc: <<\nd: !!str 2024-01-01\ne: '2024-01-01'\n---\n";
    assert_eq!(
        values(source),
        [pair("d", "2024-01-01"), pair("e", "2024-01-01")]
    );
}

#[test]
fn a_number_that_yaml_v3_reads_is_no_text_and_a_near_miss_is() {
    let source = "---\na: 1_000\nb: 0b11\nc: -0b11\nd: 1_000.5\ne: 0x_1F\nf: _1000\ng: 1_000a\nh: 0b12\ni: v1_0\n---\n";
    let keys: Vec<_> = values(source).into_iter().map(|(key, _)| key).collect();
    assert_eq!(keys, ["f", "g", "h", "i"]);
}

#[test]
fn an_anchor_before_a_value_or_a_key_leaves_the_text() {
    let source = "---\na: &x anchored\n&k b: keyed\n---\n";
    assert_eq!(values(source), [pair("a", "anchored"), pair("b", "keyed")]);
}

#[test]
fn an_alias_and_a_complex_key_hold_no_text_and_drop_no_entry_after_them() {
    let source = "---\na: &x text\nb: *x\n? [k, l]\n: v\nc: after\n---\n";
    assert_eq!(values(source), [pair("a", "text"), pair("c", "after")]);
}

#[test]
fn a_repeated_key_and_a_merge_key_keep_their_values() {
    assert_eq!(
        values("---\na: x\na: y\nb: z\n---\n"),
        [pair("a", "x"), pair("a", "y"), pair("b", "z")]
    );
    let merge = "---\nbase: &b {x: 1}\n<<: *b\nt: text\n---\n";
    assert_eq!(values(merge), [pair("t", "text")]);
}

#[test]
fn a_number_key_a_null_key_and_a_non_finite_float_keep_their_values() {
    let source = "---\n1: one\nnull: two\nup: .inf\nnone: .NaN\ntitle: three\n---\n";
    assert_eq!(
        values(source),
        [
            pair("1", "one"),
            pair("null", "two"),
            pair("title", "three")
        ]
    );
}

#[test]
fn front_matter_that_no_parser_reads_has_a_fault_with_its_line() {
    for (source, line) in [
        ("---\ntitle: ok\nother: [open\n---\n\nBody.\n", "line 3"),
        ("---\nkey: x\n  bad: indent\n---\n", "line 3"),
        ("+++\ntitle = \"open\n+++\n", "line 2"),
        (";;;\n{\"title\": \"x\", \n;;;\n", "line 3"),
    ] {
        let Some(fault) = fault(source) else {
            panic!("{source:?} has no fault");
        };
        assert!(fault.starts_with(line), "{source:?}: {fault}");
    }
}

#[test]
fn front_matter_that_a_parser_reads_or_that_never_closes_has_no_fault() {
    for source in [
        "---\ntitle: ok\n---\n",
        "---\n? [a, b]\n: x\ntitle: y\n---\n",
        "+++\ntitle = \"ok\"\n+++\n",
        ";;;\n{\"title\": \"ok\"}\n;;;\n",
        "---\ntitle: [open\n",
        "Body.\n",
        "",
    ] {
        assert_eq!(fault(source), None, "{source:?}");
    }
}

#[test]
fn the_strings_before_the_first_table_are_the_values_of_toml() {
    let source = "+++\ntitle = \"A very title\"\ncount = 3\ntags = [\"a\"]\nliteral = 'x y'\n[params]\nsub = \"in a table\"\n+++\n";
    assert_eq!(
        values(source),
        [pair("title", "A very title"), pair("literal", "x y")]
    );
}

#[test]
fn a_toml_string_may_go_over_lines() {
    let source = "+++\nbody = \"\"\"\none\ntwo\"\"\"\n+++\n";
    assert_eq!(values(source), [pair("body", "one\ntwo")]);
}

#[test]
fn an_opening_fence_may_carry_blanks_after_it() {
    let source = "--- \t\ntitle: A very title\n---\n\nBody.\n";
    assert_eq!(
        read(&leading(source), source),
        [pair("title", "A very title")]
    );
    let toml = "+++  \ntitle = \"A very title\"\n+++\n";
    assert_eq!(read(&leading(toml), toml), [pair("title", "A very title")]);
}

#[test]
fn the_strings_of_a_json_object_are_values_and_its_other_members_are_not() {
    let source = ";;;\n{\"title\": \"A very json\", \"n\": 3, \"tags\": [\"x\"], \"o\": {\"k\": \"y\"}, \"s\": \"z\"}\n;;;\n\nBody.\n";
    assert_eq!(
        read(&leading(source), source),
        [pair("title", "A very json"), pair("s", "z")]
    );
    let none = Vec::<(String, String)>::new();
    assert_eq!(read(&leading("Body.\n"), "Body.\n"), none);
    let open = ";;;\n{\"a\": \"b\"}\n";
    assert_eq!(read(&leading(open), open), none);
}

#[test]
fn a_file_whose_fence_carries_blanks_has_its_values_in_the_tree() {
    for source in [
        "--- \t\ntitle: A very title\n---\n\nBody.\n",
        "+++ \ntitle = \"A very title\"\n+++\n\nBody.\n",
    ] {
        let built = build(source, Flavor::Markdown).unwrap();
        let values = built.nodes.iter().filter(|node| node.label.is_some());
        assert_eq!(read(values, source), [pair("title", "A very title")]);
    }
}

/// One source for each form of front matter, each with the value `title`.
const FORMS: [&str; 9] = [
    "---\ntitle: A very title\n---\n\nBody.\n",
    "---yaml\ntitle: A very title\n---\n\nBody.\n",
    "+++\ntitle = \"A very title\"\n+++\n\nBody.\n",
    "---toml\ntitle = \"A very title\"\n---\n\nBody.\n",
    ";;;\n{\"title\": \"A very title\"}\n;;;\n\nBody.\n",
    "---json\n{\"title\": \"A very title\"}\n---\n\nBody.\n",
    "{\"title\": \"A very title\"}\n\nBody.\n",
    "{\n\"title\": \"A very title\"\n}\n\nBody.\n",
    "---\ntitle: A very title\n...\n\nBody.\n",
];

#[test]
fn each_form_of_front_matter_gives_its_values() {
    for source in FORMS {
        assert_eq!(
            values(source),
            [pair("title", "A very title")],
            "{source:?}"
        );
    }
}

/// Whether no node of `nodes`, outside the front matter, starts before `end`.
fn body_starts_at_or_after(nodes: &[Node], end: usize) -> bool {
    nodes.iter().all(|node| {
        node.scope == Scope::FrontMatter
            || (node.span.start >= end && body_starts_at_or_after(&node.children, end))
    })
}

#[test]
fn every_form_of_front_matter_is_cut_from_the_body() {
    for source in FORMS {
        let Some(block) = span(source) else {
            panic!("{source:?} has no block");
        };
        let built = build(source, Flavor::Markdown).unwrap();
        let values = built
            .nodes
            .iter()
            .filter(|node| node.scope == Scope::FrontMatter);
        assert_eq!(
            read(values, source),
            [pair("title", "A very title")],
            "{source:?}"
        );
        assert!(
            body_starts_at_or_after(&built.nodes, block.end),
            "{source:?}: a node starts inside the block"
        );
    }
}

#[test]
fn blanking_keeps_each_line_and_each_byte_offset() {
    let source = "---\r\ntitle: x\u{e9}y\r\n---\r\nBody \u{e9}.\n";
    let Some(block) = span(source) else {
        panic!("{source:?} has no block");
    };
    let expected = format!(
        "{}\n{}\n{}\nBody \u{e9}.\n",
        " ".repeat(4),
        " ".repeat(12),
        " ".repeat(4)
    );
    assert_eq!(blank(source, &block), expected);
}

#[test]
fn a_line_that_opens_with_a_brace_and_is_no_json_object_is_no_block() {
    for source in [
        "{/* note */}\n\nBody.\n",
        "{props.title}\n\nBody.\n",
        "{% raw %}\nBody.\n{% endraw %}\n",
        "{{< shortcode >}}\n\nBody.\n",
        "{\"title\": \"x\"} trailing\n\nBody.\n",
    ] {
        assert_eq!(span(source), None, "{source:?}");
        assert_eq!(fault(source), None, "{source:?}");
    }
}
