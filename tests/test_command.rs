//! `hakkiri test`: the scenarios of the test-command change.
//!
//! Each test writes a style folder and its cases in a temporary directory, runs
//! the binary there, and reads the exit status and the two streams.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

mod common;

use std::error::Error;

use common::{binary, run};
use ctl_core::fixture::{Root, Suite};
use serde_json::Value;

type TestResult = Result<(), Box<dyn Error>>;

/// A rule that warns on `very`, with two cases.
const FILLER: &str = "\
extends: existence
message: \"Remove '%s'.\"
level: warning
tokens: [very]
tests:
  - name: flags very
    input: This is very good.
    want: \"1:9:Team.Filler:Remove 'very'.\"
  - name: a clean line
    input: It works.
    want: ''
";

/// A rule that warns on `perhaps`, with no case.
const HEDGE: &str =
    "extends: existence\nmessage: \"Be direct: '%s'.\"\nlevel: warning\ntokens: [perhaps]\n";

/// A project config with `*.md` under the style `Team`.
const CONFIG: &str = "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n";

/// A project config that turns `Team.Filler` off for `*.md`.
const CONFIG_OFF: &str =
    "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team], Team.Filler: off}\n";

/// A project whose style `Team` holds a rule with two cases.
fn styled<'a>(suite: &'a Suite, name: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("styles/Team/Filler.yml", FILLER);
    root
}

#[test]
fn a_rule_and_a_test_file_run_five_cases() -> TestResult {
    let suite = Suite::new("test-five");
    let root = styled(&suite, "five");
    root.file(
        "test/package.test.yml",
        "\
- name: one
  rule: ../styles/Team/Filler.yml
  input: A very good day.
  contains: Team.Filler
- name: two
  rule: ../styles/Team/Filler.yml
  input: A good day.
  absent: Team.Filler
- name: three
  rule: ../styles/Team/Filler.yml
  input: A good day.
  want: ''
",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["2 files, 5 passed, 0 failed"]);
    Ok(())
}

#[test]
fn yaml_that_is_not_a_rule_is_not_read() -> TestResult {
    let suite = Suite::new("test-foreign");
    let root = styled(&suite, "foreign");
    root.file(".github/ci.yml", "name: ci\non: push\n");
    root.file("ci.yml", "name: ci\non: push\n");
    root.file("eslint.yml", "extends: standard\nrules: {}\n");
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 2 passed, 0 failed"]);
    Ok(())
}

#[test]
fn no_case_is_an_error() -> TestResult {
    let suite = Suite::new("test-none");
    let root = suite.scenario("none");
    root.file("styles/Team/Hedge.yml", HEDGE);
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 2);
    assert!(result.out.contains("no case found"), "{}", result.out);
    Ok(())
}

#[test]
fn a_case_that_asserts_nothing_does_not_run() -> TestResult {
    let suite = Suite::new("test-nothing");
    let root = styled(&suite, "nothing");
    root.file(
        "bad.test.yml",
        "- name: nothing\n  rule: styles/Team/Filler.yml\n  input: A very day.\n",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 2);
    assert!(
        result
            .out
            .contains("needs one of `want`, `contains` or `absent`"),
        "{}",
        result.out
    );
    assert_eq!(result.lines().last(), Some(&"1 file, 2 passed, 0 failed"));
    Ok(())
}

#[test]
fn a_duplicate_name_and_an_unknown_field_are_refused() -> TestResult {
    let suite = Suite::new("test-refused");
    let root = styled(&suite, "refused");
    root.file(
        "twice.test.yml",
        "- {name: same, rule: styles/Team/Filler.yml, want: ''}\n- {name: same, rule: styles/Team/Filler.yml, want: ''}\n",
    );
    root.file(
        "view.test.yml",
        "- name: view\n  view: Titles\n  rule: styles/Team/Filler.yml\n  want: ''\n",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 2);
    assert!(
        result.out.contains("twice.test.yml: ")
            && result.out.contains("`same` is used by two cases"),
        "{}",
        result.out
    );
    assert!(
        result.out.contains("view.test.yml: ") && result.out.contains("unknown field `view`"),
        "{}",
        result.out
    );
    Ok(())
}

#[test]
fn a_failed_case_prints_its_file_name_and_reason() -> TestResult {
    let suite = Suite::new("test-failed");
    let root = suite.scenario("failed");
    root.file("styles/Team/Filler.yml", FILLER.replace("tests:", "tests:\n  - name: flags filler\n    about: Issue 7.\n    input: It works.\n    contains: Team.Filler"));
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "failed styles/Team/Filler.yml: flags filler",
            "  output does not contain \"Team.Filler\"",
            "  about: Issue 7.",
            "  got: no alerts",
            "1 file, 2 passed, 1 failed",
        ]
    );
    Ok(())
}

#[test]
fn a_want_that_differs_prints_a_diff() -> TestResult {
    let suite = Suite::new("test-diff");
    let root = suite.scenario("diff");
    root.file("styles/Team/Filler.yml", FILLER.replace("1:9:", "1:8:"));
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 1);
    assert_eq!(
        result.lines(),
        [
            "failed styles/Team/Filler.yml: flags very",
            "  output does not match",
            "  - 1:8:Team.Filler:Remove 'very'.",
            "  + 1:9:Team.Filler:Remove 'very'.",
            "1 file, 1 passed, 1 failed",
        ]
    );
    Ok(())
}

#[test]
fn a_case_that_cannot_run_is_reported_with_the_others() -> TestResult {
    let suite = Suite::new("test-cannot");
    let root = styled(&suite, "cannot");
    root.file(
        "mixed.test.yml",
        "\
- name: gone
  rule: Missing.yml
  input: A very day.
  contains: Team.Filler
- name: fails
  rule: styles/Team/Filler.yml
  input: A day.
  contains: Team.Filler
",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 2);
    assert!(
        result.out.contains("could not run mixed.test.yml: gone: "),
        "{}",
        result.out
    );
    assert!(
        result.out.contains("failed mixed.test.yml: fails"),
        "{}",
        result.out
    );
    assert_eq!(result.lines().last(), Some(&"2 files, 2 passed, 2 failed"));
    Ok(())
}

#[test]
fn an_isolated_case_ignores_the_config_and_another_follows_it() -> TestResult {
    let suite = Suite::new("test-config");
    let root = styled(&suite, "config");
    root.file(".ctl/hakkiri.yaml", CONFIG_OFF);
    root.file(
        "through.test.yml",
        "- name: through the config\n  input: A very day.\n  contains: Team.Filler\n",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert!(
        result
            .out
            .contains("failed through.test.yml: through the config"),
        "{}",
        result.out
    );
    assert_eq!(result.lines().last(), Some(&"2 files, 2 passed, 1 failed"));
    Ok(())
}

#[test]
fn a_case_through_the_config_lints_with_its_styles() -> TestResult {
    let suite = Suite::new("test-through");
    let root = styled(&suite, "through");
    root.file(".ctl/hakkiri.yaml", CONFIG);
    root.file(
        "through.test.yml",
        "- name: through the config\n  input: A very day.\n  contains: Team.Filler\n",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["2 files, 3 passed, 0 failed"]);
    Ok(())
}

#[test]
fn coverage_names_a_rule_no_case_fires() -> TestResult {
    let suite = Suite::new("test-coverage");
    let root = styled(&suite, "coverage");
    root.file(
        "styles/Team/Hedge.yml",
        format!("{HEDGE}tests:\n  - name: nothing flagged\n    input: Perhaps.\n    want: ''\n"),
    );
    let plain = run(root.path(), &["test"], &[])?;
    assert_eq!(plain.code, 0, "{}{}", plain.out, plain.err);
    let covered = run(root.path(), &["test", "--coverage"], &[])?;
    assert_eq!(covered.code, 1, "{}{}", covered.out, covered.err);
    assert_eq!(
        covered.lines(),
        [
            "uncovered Team.Hedge: no case makes it flag anything",
            "2 files, 3 passed, 0 failed, 1 uncovered",
        ]
    );
    Ok(())
}

#[test]
fn a_walk_skips_node_modules() -> TestResult {
    let suite = Suite::new("test-node-modules");
    let root = styled(&suite, "node-modules");
    root.file(
        "node_modules/pkg/vendored.test.yml",
        "- name: vendored\n  rule: ../../styles/Team/Filler.yml\n  input: A very day.\n  absent: Team.Filler\n",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 2 passed, 0 failed"]);
    Ok(())
}

#[test]
fn a_rule_file_with_a_dotted_name_is_named_as_the_style_loader_names_it() -> TestResult {
    let suite = Suite::new("test-dotted");
    let root = suite.scenario("dotted");
    root.file("styles/Team/Foo.v2.yml", HEDGE);
    root.file(".ctl/hakkiri.yaml", CONFIG);
    root.file(
        "through.test.yml",
        "- name: through the config\n  input: Say perhaps later.\n  contains: Team.Foo.v2\n",
    );
    let result = run(root.path(), &["test", "--coverage"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 1 passed, 0 failed, 0 uncovered"]);
    Ok(())
}

#[test]
fn json_holds_no_escape_byte_under_forced_colour() -> TestResult {
    let suite = Suite::new("test-json");
    let root = styled(&suite, "json");
    for args in [
        &["test", "--format", "json"][..],
        &["test", "--bogus", "--format", "json"][..],
    ] {
        let output = binary()
            .current_dir(root.path())
            .args(args)
            .env("CLICOLOR_FORCE", "1")
            .env("FORCE_COLOR", "3")
            .env("COLORTERM", "truecolor")
            .output()?;
        assert!(!output.stdout.contains(&0x1b), "{args:?}");
        assert!(!output.stderr.contains(&0x1b), "{args:?}");
    }
    let output = binary()
        .current_dir(root.path())
        .args(["test", "--format", "json"])
        .env("CLICOLOR_FORCE", "1")
        .output()?;
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["passed"], 2);
    assert_eq!(json["failed"], 0);
    assert_eq!(json["results"][0]["name"], "flags very");
    assert_eq!(json["uncovered"].as_array().map(Vec::len), Some(0));
    Ok(())
}

#[test]
fn check_loads_the_cases_and_runs_none() -> TestResult {
    let suite = Suite::new("test-check");
    let root = suite.scenario("check");
    root.file(
        "styles/Team/Filler.yml",
        FILLER.replace("want: ''", "want: this-would-fail"),
    );
    root.file(".ctl/hakkiri.yaml", CONFIG);
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines().last(), Some(&"1 alert in 1 file"));
    Ok(())
}

#[test]
fn a_malformed_case_in_a_rule_is_refused_at_load() -> TestResult {
    let suite = Suite::new("test-malformed");
    let root = suite.scenario("malformed");
    root.file(
        "styles/Team/Filler.yml",
        FILLER.replace("    input: It works.", "    inputs: It works."),
    );
    root.file(".ctl/hakkiri.yaml", CONFIG);
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2);
    assert!(result.err.contains("line 10"), "{}", result.err);
    assert!(
        result.err.contains("unknown field `inputs`"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_style_folder_of_three_rules_covers_every_rule() -> TestResult {
    let result = run(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
        &["test", "--coverage", "tests/fixtures/cases"],
        &[],
    )?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["3 files, 6 passed, 0 failed, 0 uncovered"]);
    Ok(())
}
