//! Tests for the specificity of a glob, which ranks the globs of a config.

use std::cmp::Reverse;

use super::{MAX_ALTERNATIVES, alternatives, specificity};

/// A glob and its specificity: the parts of literal characters only, the parts
/// of a literal character and something else, the parts that are `**`, and the
/// literal characters of the whole glob.
#[test]
fn a_glob_counts_its_parts_and_its_literal_characters() {
    for (pattern, counts) in [
        ("docs/*", (1, 0, 0, 4)),
        ("*.mdx", (0, 1, 0, 4)),
        ("docs/**/*.md", (1, 1, 1, 7)),
        ("*.md", (0, 1, 0, 3)),
        ("CHANGELOG.md", (1, 0, 0, 12)),
        ("openspec/changes/**/proposal.md", (3, 0, 1, 26)),
        ("**/openspec/changes/**/*.md", (2, 1, 2, 18)),
        ("a.md", (1, 0, 0, 4)),
        ("a?md", (0, 1, 0, 3)),
        ("a\\*.md", (1, 0, 0, 5)),
        ("**", (0, 0, 1, 0)),
        ("*", (0, 0, 0, 0)),
        ("[ab].md", (0, 1, 0, 3)),
        ("[]a].md", (0, 1, 0, 3)),
        ("[!a].md", (0, 1, 0, 3)),
    ] {
        let (only, mixed, recursive, literals) = counts;
        assert_eq!(
            specificity(pattern),
            (only, mixed, Reverse(recursive), literals),
            "{pattern}"
        );
    }
}

/// Each row of the table of the design: a file, and two globs that match it,
/// the more specific first.
#[test]
fn the_more_specific_glob_ranks_above_the_other() {
    for (winner, loser) in [
        ("docs/*", "*.mdx"),
        ("docs/**/*.md", "*.md"),
        ("CHANGELOG.md", "*.md"),
        ("a*.md", "*.md"),
        (
            "openspec/changes/**/proposal.md",
            "**/openspec/changes/**/*.md",
        ),
        ("a.md", "a?md"),
        ("a.md", "*.md"),
        ("*", "**"),
    ] {
        assert!(
            specificity(winner) > specificity(loser),
            "{winner} over {loser}"
        );
    }
    assert_eq!(specificity("a?md"), specificity("*.md"));
}

/// A glob with `{…}` groups expands to one glob per choice, and a brace in a
/// class or after a backslash is no group.
#[test]
fn a_glob_expands_to_its_alternatives() {
    for (pattern, expected) in [
        ("*.{md,mdx,rs}", vec!["*.md", "*.mdx", "*.rs"]),
        ("{CHANGELOG.md,docs/**}", vec!["CHANGELOG.md", "docs/**"]),
        (
            "{a,b}/{c,d}.md",
            vec!["a/c.md", "a/d.md", "b/c.md", "b/d.md"],
        ),
        ("{a,{b,c}}.md", vec!["a.md", "b.md", "c.md"]),
        ("{,x}.md", vec![".md", "x.md"]),
        ("\\{a,b\\}.md", vec!["\\{a,b\\}.md"]),
        ("[{]a,b}.md", vec!["[{]a,b}.md"]),
        ("docs/*.md", vec!["docs/*.md"]),
    ] {
        let mut got = alternatives(pattern).unwrap_or_default();
        got.sort_unstable();
        let mut want: Vec<String> = expected.into_iter().map(String::from).collect();
        want.sort_unstable();
        assert_eq!(got, want, "{pattern}");
    }
}

/// A glob that expands past the limit has no alternatives: eight groups of two
/// give 256, the limit, and nine give 512.
#[test]
fn a_glob_over_the_limit_of_alternatives_is_refused() {
    assert_eq!(
        alternatives(&"{a,b}".repeat(8)).map(|all| all.len()),
        Some(MAX_ALTERNATIVES)
    );
    assert_eq!(alternatives(&"{a,b}".repeat(9)), None);
}
