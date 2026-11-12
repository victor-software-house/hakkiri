//! Tests for the rewrite of a file under block and token patterns: the cases
//! of `Test_applyPatterns` in Vale, and the places the rewritten text came
//! from.

use super::{Changed, rewrite};
use crate::pattern::Pattern;

/// The places in the file of the bytes `range` of the rewritten text, as pairs
/// of a start and an end.
fn places(out: &Changed, range: std::ops::Range<usize>) -> Vec<(usize, usize)> {
    out.origin(range)
        .into_iter()
        .map(|place| (place.start, place.end))
        .collect()
}

/// `source` rewritten under the block patterns `blocks` and the token patterns
/// `tokens`.
fn changed(source: &str, blocks: &[&str], tokens: &[&str]) -> Changed {
    let compile = |sources: &[&str]| -> Vec<Pattern> {
        sources
            .iter()
            .map(|source| Pattern::new(source).expect("the pattern compiles"))
            .collect()
    };
    let (blocks, tokens) = (compile(blocks), compile(tokens));
    let blocks: Vec<&Pattern> = blocks.iter().collect();
    let tokens: Vec<&Pattern> = tokens.iter().collect();
    rewrite(source, &blocks, &tokens).expect("the patterns run")
}

#[test]
fn a_block_becomes_a_fenced_block_that_holds_its_group() {
    let out = changed(
        "Intro.\n\nBEGIN\nskipped\nEND\n",
        &["(?s)(BEGIN.*?END)"],
        &[],
    );
    assert_eq!(out.text(), "Intro.\n\n\n```\nBEGIN\nskipped\nEND\n```\n\n");
}

#[test]
fn a_token_becomes_a_code_span_that_holds_its_group() {
    let out = changed("A test $g_i = p_i$ here.", &[], &[r"(\$+[^\n$]+\$+)"]);
    assert_eq!(out.text(), "A test `$g_i = p_i$` here.");
}

#[test]
fn a_pattern_with_no_group_leaves_dollar_one() {
    let out = changed(
        "a\n\n## Why\n\nb\n",
        &["(?m)^## (?:Why|What Changes)$"],
        &[],
    );
    assert_eq!(out.text(), "a\n\n\n```\n$1\n```\n\n\nb\n");
    let out = changed("a {{ x }} b\n", &[], &["{{.*?}}"]);
    assert_eq!(out.text(), "a `$1` b\n");
}

#[test]
fn a_group_that_took_no_part_leaves_nothing() {
    let out = changed("a x b\n", &[], &["(x)|(b)"]);
    assert_eq!(out.text(), "a `x` ``\n");
}

#[test]
fn a_match_of_no_bytes_changes_nothing() {
    let out = changed("a b\n", &["(z*)"], &["(z*)"]);
    assert_eq!(out.text(), "a b\n");
    assert_eq!(places(&out, 0..4), [(0, 4)]);
}

#[test]
fn a_pattern_reads_the_text_the_one_before_it_left() {
    let out = changed("a\nBEGIN x END\nb\n", &["(?s)(BEGIN.*?END)", "(x)"], &[]);
    assert_eq!(
        out.text(),
        "a\n\n```\nBEGIN \n```\nx\n```\n END\n```\n\nb\n"
    );
}

#[test]
fn a_token_pattern_runs_after_the_block_patterns() {
    let out = changed(
        "a\nBEGIN {{ x }} END\nb {{ y }}\n",
        &["(?s)(BEGIN.*?END)"],
        &["({{.*?}})"],
    );
    assert_eq!(
        out.text(),
        "a\n\n```\nBEGIN `{{ x }}` END\n```\n\nb `{{ y }}`\n"
    );
}

#[test]
fn text_the_patterns_leave_is_found_in_the_file() {
    let source = "a {{ x }} b\n";
    let out = changed(source, &[], &["({{.*?}})"]);
    assert_eq!(out.text(), "a `{{ x }}` b\n");
    // "a " and " b\n" are the file's own bytes.
    assert_eq!(places(&out, 0..2), [(0, 2)]);
    assert_eq!(
        places(&out, out.text().len() - 3..out.text().len()),
        [(9, 12)]
    );
    // The group inside the code span is the file's `{{ x }}`.
    assert_eq!(places(&out, 3..10), [(2, 9)]);
    // The backticks are not in the file.
    assert_eq!(places(&out, 2..3), Vec::<(usize, usize)>::new());
    // A range over a backtick and the text after it gives the text only.
    assert_eq!(places(&out, 10..13), [(9, 11)]);
}

#[test]
fn bytes_the_file_never_held_have_no_place() {
    let out = changed("a\n\nb\n", &["(b)"], &[]);
    assert_eq!(out.text(), "a\n\n\n```\nb\n```\n\n");
    assert_eq!(places(&out, 3..8), Vec::<(usize, usize)>::new());
    assert_eq!(places(&out, 8..9), [(3, 4)]);
}

#[test]
fn a_place_stays_exact_through_two_rewrites() {
    let source = "x BEGIN {{ y }} END z\n";
    let out = changed(source, &["(?s)(BEGIN.*?END)"], &["({{.*?}})"]);
    let at = out
        .text()
        .find("{{ y }}")
        .expect("the token is in the text");
    let found = places(&out, at..at + 7);
    assert_eq!(found.len(), 1);
    let (start, end) = found.first().copied().expect("one place");
    assert_eq!(source.get(start..end), Some("{{ y }}"));
}

#[test]
fn an_empty_range_has_its_place() {
    let out = changed("ab\n", &[], &["(b)"]);
    assert_eq!(out.text(), "a`b`\n");
    assert_eq!(places(&out, 1..1), [(1, 1)]);
}

#[test]
fn a_token_that_holds_a_block_is_split_before_the_tokens_run() {
    let out = changed(
        "{{ BEGIN x END }}\n",
        &["(?s)(BEGIN.*?END)"],
        &["({{.*?}})"],
    );
    assert_eq!(out.text(), "{{ \n```\nBEGIN x END\n```\n }}\n");
}
