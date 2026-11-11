//! `hakkiri check` with a rule setting that names one rule: the scenarios of
//! the rule-setting change.
//!
//! Each test writes a project in a temporary directory, runs the binary in it,
//! and reads the exit status and the output.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;

mod common;

use common::run;
use ctl_core::fixture::{Root, Suite};

type TestResult = Result<(), Box<dyn Error>>;

/// A text that each rule below flags once: `very`, `terrible`, `maybe` and
/// `bland` start at columns 9, 14, 23 and 29.
const TEXT: &str = "This is very terrible maybe bland.\n";

/// A text only `very` flags.
const VERY: &str = "This is very good.\n";

/// A rule that warns on `very`.
const FILLER: &str =
    "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\ntokens: [very]\n";

/// A second rule file for `very`, with another message.
const FILLER_CUSTOM: &str =
    "extends: existence\nmessage: \"Custom '%s'.\"\nlevel: warning\ntokens: [very]\n";

/// A rule that fails on `terrible`.
const HARSH: &str =
    "extends: existence\nmessage: \"Never '%s'.\"\nlevel: error\ntokens: [terrible]\n";

/// A rule that only suggests against `maybe`.
const HINT: &str =
    "extends: existence\nmessage: \"Say '%s' less.\"\nlevel: suggestion\ntokens: [maybe]\n";

/// A rule of the style `Other` that warns on `bland`.
const BLAND: &str =
    "extends: existence\nmessage: \"Plain '%s'.\"\nlevel: warning\ntokens: [bland]\n";

/// A project with the styles `Team` and `Other`, the config `files` and the
/// files `a.md` and `b.md`, both holding [`TEXT`].
fn project<'a>(suite: &'a Suite, name: &str, files: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("styles/Team/Filler.yml", FILLER);
    root.file("styles/Team/Harsh.yml", HARSH);
    root.file("styles/Team/Hint.yml", HINT);
    root.file("styles/Other/Bland.yml", BLAND);
    root.file(
        ".ctl/hakkiri.yaml",
        format!("packages:\n  - path: styles\nfiles:\n{files}"),
    );
    root.file("a.md", TEXT);
    root.file("b.md", TEXT);
    root
}

#[test]
fn a_rule_of_a_style_no_glob_lists_runs_at_the_level_the_glob_sets() -> TestResult {
    let suite = Suite::new("rule-setting-unlisted");
    let root = project(
        &suite,
        "unlisted",
        "  'a.md': {styles: [Other], Team.Filler: warning}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "a.md:1:29: warning Other.Bland: Plain 'bland'.",
            "2 alerts in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_rule_setting_with_no_styles_runs_that_rule_alone_at_its_level() -> TestResult {
    let suite = Suite::new("rule-setting-alone");
    let root = project(&suite, "alone", "  'a.md': {Team.Hint: error}\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:23: error Team.Hint: Say 'maybe' less.",
            "1 alert in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn off_on_a_rule_of_a_style_no_glob_lists_changes_nothing() -> TestResult {
    let suite = Suite::new("rule-setting-off-unlisted");
    let root = project(
        &suite,
        "off-unlisted",
        "  'a.md': {styles: [Other], Team.Filler: off}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:29: warning Other.Bland: Plain 'bland'.",
            "1 alert in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_style_one_glob_lists_and_another_glob_enables_by_rule() -> TestResult {
    let suite = Suite::new("rule-setting-two-globs");
    let root = project(
        &suite,
        "two-globs",
        "  '*.md': {styles: [Other]}\n  'a.md': {Team.Harsh: error}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:14: error Team.Harsh: Never 'terrible'.",
            "a.md:1:29: warning Other.Bland: Plain 'bland'.",
            "b.md:1:29: warning Other.Bland: Plain 'bland'.",
            "3 alerts in 2 files"
        ]
    );
    Ok(())
}

#[test]
fn a_later_glob_turns_a_rule_on_after_an_earlier_one_turned_it_off() -> TestResult {
    let suite = Suite::new("rule-setting-on-after-off");
    let root = project(
        &suite,
        "on-after-off",
        "  '*.md': {styles: [Team], Team.Harsh: off}\n  'a.md': {Team.Harsh: error}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "a.md:1:14: error Team.Harsh: Never 'terrible'.",
            "a.md:1:23: suggestion Team.Hint: Say 'maybe' less.",
            "b.md:1:9: warning Team.Filler: Remove 'very'.",
            "b.md:1:23: suggestion Team.Hint: Say 'maybe' less.",
            "5 alerts in 2 files"
        ]
    );
    Ok(())
}

#[test]
fn a_later_glob_turns_a_rule_off_after_an_earlier_one_enabled_it() -> TestResult {
    let suite = Suite::new("rule-setting-off-after-on");
    let root = project(
        &suite,
        "off-after-on",
        "  '*.md': {Team.Filler: warning}\n  'a.md': {Team.Filler: off}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "b.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_later_glob_sets_the_level_again() -> TestResult {
    let suite = Suite::new("rule-setting-level-again");
    let root = project(
        &suite,
        "level-again",
        "  '*.md': {Team.Filler: warning}\n  'a.md': {Team.Filler: error}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: error Team.Filler: Remove 'very'.",
            "b.md:1:9: warning Team.Filler: Remove 'very'.",
            "2 alerts in 2 files"
        ]
    );
    Ok(())
}

#[test]
fn a_later_list_of_styles_keeps_the_rule_an_earlier_glob_enabled() -> TestResult {
    let suite = Suite::new("rule-setting-list-keeps");
    let root = project(
        &suite,
        "list-keeps",
        "  '*.md': {Team.Filler: warning}\n  'a.md': {styles: [Other]}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "a.md:1:29: warning Other.Bland: Plain 'bland'.",
            "b.md:1:9: warning Team.Filler: Remove 'very'.",
            "3 alerts in 2 files"
        ]
    );
    Ok(())
}

#[test]
fn an_empty_list_of_styles_drops_every_rule_an_earlier_glob_enabled() -> TestResult {
    let suite = Suite::new("rule-setting-empty-list");
    let root = project(
        &suite,
        "empty-list",
        "  '*.md': {styles: [Other], Team.Filler: warning}\n  'a.md': {styles: []}\n  'b.md': {styles: []}\n  'b?md': {Team.Hint: warning}\n",
    );
    root.file("c.md", TEXT);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "b.md:1:23: warning Team.Hint: Say 'maybe' less.",
            "c.md:1:9: warning Team.Filler: Remove 'very'.",
            "c.md:1:29: warning Other.Bland: Plain 'bland'.",
            "3 alerts in 2 files"
        ]
    );
    Ok(())
}

#[test]
fn the_order_of_keys_inside_a_glob_does_not_matter() -> TestResult {
    let suite = Suite::new("rule-setting-key-order");
    let root = project(
        &suite,
        "key-order",
        "  'a.md': {Team.Harsh: off, styles: [Team]}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "a.md:1:23: suggestion Team.Hint: Say 'maybe' less.",
            "2 alerts in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_name_clash_in_a_style_enabled_only_by_rule_is_not_refused() -> TestResult {
    let suite = Suite::new("rule-setting-clash");
    let root = suite.scenario("clash");
    root.file("styles/Team/Terms.yml", FILLER);
    root.file("styles/Team/Terms.custom.yml", FILLER_CUSTOM);
    root.file("styles/Other/Bland.yml", BLAND);
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Other], Team.Terms: warning}\n",
    );
    root.file("a.md", VERY);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Terms: Remove 'very'.",
            "1 alert in 1 file"
        ]
    );
    Ok(())
}
