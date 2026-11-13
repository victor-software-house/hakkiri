//! Tests for the specificity of a glob, which ranks the globs of a config.

use std::cmp::Reverse;

use super::specificity;

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
        ("*.{md,txt}", (0, 1, 0, 1)),
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
        ("*.md", "*.{md,txt}"),
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
