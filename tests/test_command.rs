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

/// A project config that hides the text between `BEGIN` and `END` from `*.md`.
const CONFIG_IGNORING: &str = "\
packages:
  - path: styles
files:
  '*.md':
    styles: [Team]
    block_ignores:
      - \"(?s)BEGIN(.*?)END\"
";

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
fn a_case_through_the_config_reads_the_text_the_config_ignores() -> TestResult {
    let suite = Suite::new("test-ignoring");
    let root = styled(&suite, "ignoring");
    root.file(".ctl/hakkiri.yaml", CONFIG_IGNORING);
    root.file(
        "through.test.yml",
        "- name: an ignored block\n  input: \"BEGIN\\nA very day.\\nEND\\n\"\n  want: ''\n",
    );
    root.file("doc.md", "BEGIN\nA very day.\nEND\n");
    let checked = run(root.path(), &["check", "doc.md"], &[])?;
    assert_eq!(checked.code, 0, "{}{}", checked.out, checked.err);
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
        "- name: through the config\n  input: Say perhaps later.\n  contains: Team.Foo\n  absent: Team.Foo.v2\n",
    );
    let result = run(root.path(), &["test", "--coverage"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 1 passed, 0 failed, 0 uncovered"]);
    Ok(())
}

#[test]
fn a_nested_rule_file_is_named_by_every_folder_under_its_style() -> TestResult {
    let suite = Suite::new("test-nested");
    let root = suite.scenario("nested");
    root.file("styles/Team/Sub/Deep/Bar.yml", HEDGE);
    root.file(".ctl/hakkiri.yaml", CONFIG);
    root.file(
        "through.test.yml",
        "- name: through the config\n  input: Say perhaps later.\n  contains: Team.Sub.Deep.Bar\n\
         - name: alone\n  rule: styles/Team/Sub/Deep/Bar.yml\n  input: Say perhaps later.\n  contains: Team.Sub.Deep.Bar\n",
    );
    let result = run(root.path(), &["test", "--coverage"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 2 passed, 0 failed, 0 uncovered"]);
    Ok(())
}

/// A rule on `very` that holds one case, so it is named by its own file.
const FOO: &str = "\
extends: existence
message: \"Remove '%s'.\"
level: warning
tokens: [very]
tests:
  - name: flags very
    input: This is very good.
    want: \"1:9:Team.Foo:Remove 'very'.\"
";

#[test]
fn a_rule_file_is_named_from_inside_its_style_folder() -> TestResult {
    let suite = Suite::new("test-inside");
    let root = suite.scenario("inside");
    root.file("styles/Team/Foo.yml", FOO);
    let folder = root.path().join("styles/Team");
    for (args, count) in [
        (&["test", "Foo.yml"][..], "1 file, 1 passed, 0 failed"),
        (&["test", "./Foo.yml"][..], "1 file, 1 passed, 0 failed"),
        (
            &["test", "--coverage", "."][..],
            "1 file, 1 passed, 0 failed, 0 uncovered",
        ),
        (
            &["test", "--coverage", "../Team/Foo.yml"][..],
            "1 file, 1 passed, 0 failed, 0 uncovered",
        ),
    ] {
        let result = run(&folder, args, &[])?;
        assert_eq!(result.code, 0, "{args:?}: {}{}", result.out, result.err);
        assert_eq!(result.lines(), [count], "{args:?}");
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_symlinked_style_folder_is_named_by_its_link() -> TestResult {
    let suite = Suite::new("test-symlink");
    let root = suite.scenario("symlink");
    root.file("shared/TeamStyle/Foo.yml", HEDGE);
    root.file(".ctl/hakkiri.yaml", CONFIG);
    std::fs::create_dir_all(root.path().join("styles"))?;
    std::os::unix::fs::symlink("../shared/TeamStyle", root.path().join("styles/Team"))?;
    root.file(
        "through.test.yml",
        "- name: through the config\n  input: Say perhaps later.\n  contains: Team.Foo\n  absent: Team.Foo.v2\n",
    );
    let result = run(
        root.path(),
        &[
            "test",
            "--coverage",
            "through.test.yml",
            "styles/Team/Foo.yml",
        ],
        &[],
    )?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 1 passed, 0 failed, 0 uncovered"]);
    Ok(())
}

#[test]
fn a_yaml_rule_file_is_neither_read_nor_covered() -> TestResult {
    let suite = Suite::new("test-yaml-rule");
    let root = styled(&suite, "yaml-rule");
    root.file("styles/Team/Other.yaml", FOO);
    let result = run(root.path(), &["test", "--coverage"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 2 passed, 0 failed, 0 uncovered"]);
    Ok(())
}

#[test]
fn an_explicit_node_modules_folder_is_walked() -> TestResult {
    let suite = Suite::new("test-explicit-modules");
    let root = styled(&suite, "explicit-modules");
    root.file(
        "node_modules/pkg/vendored.test.yml",
        "- name: vendored\n  rule: ../../styles/Team/Filler.yml\n  input: A very day.\n  contains: Team.Filler\n",
    );
    root.file(
        "node_modules/pkg/node_modules/deep/inner.test.yml",
        "- name: inner\n  rule: ../../../../styles/Team/Filler.yml\n  input: A very day.\n  absent: Team.Filler\n",
    );
    let result = run(root.path(), &["test", "node_modules/pkg"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 1 passed, 0 failed"]);
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_rule_path_through_a_symlink_and_back_out_is_read_from_the_folded_path() -> TestResult {
    let suite = Suite::new("test-fold");
    let root = suite.scenario("fold");
    root.file(
        "Team/Foo.yml",
        "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\ntokens: [very]\n",
    );
    root.file(
        "shared/elsewhere/Team/Foo.yml",
        "extends: existence\nmessage: \"Be plain: '%s'.\"\nlevel: warning\ntokens: [truly]\n",
    );
    root.file("shared/elsewhere/deeper/.keep", "");
    std::os::unix::fs::symlink("shared/elsewhere/deeper", root.path().join("via"))?;
    root.file(
        "fold.test.yml",
        "- name: through the link and back\n  rule: via/../Team/Foo.yml\n  input: This is very good.\n  want: \"1:9:Team.Foo:Remove 'very'.\"\n",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 1 passed, 0 failed"]);
    Ok(())
}

#[test]
fn an_explicit_node_modules_root_is_walked() -> TestResult {
    let suite = Suite::new("test-modules-root");
    let root = styled(&suite, "modules-root");
    root.file(
        "node_modules/vendored.test.yml",
        "- name: vendored\n  rule: ../styles/Team/Filler.yml\n  input: A very day.\n  contains: Team.Filler\n",
    );
    let result = run(root.path(), &["test", "node_modules"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 1 passed, 0 failed"]);
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

/// A rule on `token` whose one case expects no `Team.Terms` alert, so it
/// passes and never fires.
fn quiet(token: &str) -> String {
    format!(
        "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\ntokens: [{token}]\n\
         tests:\n  - name: quiet\n    input: This is good.\n    absent: Team.Terms\n"
    )
}

#[test]
fn a_name_two_rule_files_share_is_covered_when_either_fires() -> TestResult {
    let suite = Suite::new("test-shared-fires");
    let root = suite.scenario("shared-fires");
    root.file(
        "styles/Team/Terms.yml",
        FOO.replace("Team.Foo", "Team.Terms"),
    );
    root.file("styles/Team/Terms.custom.yml", quiet("simple"));
    let result = run(root.path(), &["test", "--coverage", "styles"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["2 files, 2 passed, 0 failed, 0 uncovered"]);
    Ok(())
}

#[test]
fn a_name_two_rule_files_share_is_reported_once_with_both_files() -> TestResult {
    let suite = Suite::new("test-shared-quiet");
    let root = suite.scenario("shared-quiet");
    root.file("styles/Team/Terms.yml", quiet("zzz"));
    root.file("styles/Team/Terms.custom.yml", quiet("yyy"));
    root.file(
        "styles/Team/Other.yml",
        FOO.replace("Team.Foo", "Team.Other"),
    );
    let result = run(root.path(), &["test", "--coverage", "styles"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "uncovered Team.Terms (named by styles/Team/Terms.custom.yml, styles/Team/Terms.yml): no case makes it flag anything",
            "3 files, 3 passed, 0 failed, 1 uncovered"
        ]
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_project_reached_through_a_link_names_a_nested_rule_as_check_does() -> TestResult {
    let suite = Suite::new("test-link-root");
    let base = suite.scenario("link-root");
    base.file("real/shared/Team/Sub/Bar.yml", HEDGE);
    base.file("real/.ctl/hakkiri.yaml", CONFIG);
    base.file(
        "real/through.test.yml",
        "- name: through the config\n  input: Say perhaps later.\n  contains: Team.Sub.Bar\n",
    );
    std::fs::create_dir_all(base.path().join("real/styles"))?;
    std::os::unix::fs::symlink("../shared/Team", base.path().join("real/styles/Team"))?;
    std::os::unix::fs::symlink("real", base.path().join("link"))?;
    let rule = base.path().join("link/styles/Team/Sub/Bar.yml");
    let result = run(
        &base.path().join("real"),
        &[
            "test",
            "--coverage",
            "through.test.yml",
            &rule.to_string_lossy(),
        ],
        &[],
    )?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 1 passed, 0 failed, 0 uncovered"]);
    Ok(())
}

#[test]
fn json_lists_an_uncovered_rule_as_an_object_with_files_only_for_a_shared_name() -> TestResult {
    let suite = Suite::new("test-uncovered-json");
    let shared = suite.scenario("shared");
    shared.file("styles/Team/Terms.yml", quiet("zzz"));
    shared.file("styles/Team/Terms.custom.yml", quiet("yyy"));
    shared.file(
        "styles/Team/Other.yml",
        FOO.replace("Team.Foo", "Team.Other"),
    );
    let result = run(
        shared.path(),
        &["test", "--coverage", "--format", "json", "styles"],
        &[],
    )?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    let json: Value = serde_json::from_str(&result.out)?;
    assert_eq!(
        json["uncovered"],
        serde_json::json!([{
            "rule": "Team.Terms",
            "files": ["styles/Team/Terms.custom.yml", "styles/Team/Terms.yml"],
        }])
    );
    let alone = suite.scenario("alone");
    alone.file("styles/Team/Terms.yml", quiet("zzz"));
    alone.file(
        "styles/Team/Other.yml",
        FOO.replace("Team.Foo", "Team.Other"),
    );
    let result = run(
        alone.path(),
        &["test", "--coverage", "--format", "json", "styles"],
        &[],
    )?;
    let json: Value = serde_json::from_str(&result.out)?;
    assert_eq!(
        json["uncovered"],
        serde_json::json!([{"rule": "Team.Terms"}])
    );
    Ok(())
}

/// `ProseStyle` v0.0.14 `NounStack` and `ThingsSpeakNoun` write their messages
/// with `%[1]s` and `%[2]s`, and each rule's own cases look for the filled
/// text.
#[test]
fn the_cases_of_two_rules_with_indexed_verbs_pass() -> TestResult {
    let suite = Suite::new("test-indexed");
    let root = suite.scenario("indexed");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sequence-verbs/rules");
    for name in ["NounStack", "ThingsSpeakNoun"] {
        let rule = std::fs::read_to_string(fixture.join(format!("{name}.yml")))?;
        root.file(format!("styles/ProseStyle/{name}.yml"), &rule);
    }
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["2 files, 6 passed, 0 failed"]);
    Ok(())
}

/// `ProseStyle` v0.0.14 `BareURLInComment` reads the whole Rust file as `raw`,
/// and its own case expects a fence in a doc comment to stay open across a
/// blank line.
#[test]
fn the_cases_of_a_rule_that_reads_a_fence_across_a_blank_line_pass() -> TestResult {
    let suite = Suite::new("test-bare-url-comment");
    let root = suite.scenario("bare-url-comment");
    let rule = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/bare-url-comment/BareURLInComment.yml"),
    )?;
    root.file("styles/ProseStyle/BareURLInComment.yml", &rule);
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 9 passed, 0 failed"]);
    Ok(())
}

/// A rule that warns on `very`, with a case in plain text.
const FILLER_TEXT: &str = "\
extends: existence
message: \"Remove '%s'.\"
level: warning
tokens: [very]
tests:
  - name: flags very in plain text
    format: txt
    input: This is very good.
    want: \"1:9:Team.Filler:Remove 'very'.\"
";

#[test]
fn a_case_in_a_format_no_extension_names_reads_as_plain_text() -> TestResult {
    let suite = Suite::new("test-plain");
    let root = suite.scenario("plain");
    root.file("styles/Team/Filler.yml", FILLER_TEXT);
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  '*.txt': {styles: [Team]}\n",
    );
    root.file(
        "through.test.yml",
        "- name: through the config\n  format: txt\n  input: A very day.\n  want: \"1:3:Team.Filler:Remove 'very'.\"\n",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["2 files, 2 passed, 0 failed"]);
    Ok(())
}

#[test]
fn a_formats_entry_picks_the_reader_of_a_case() -> TestResult {
    let suite = Suite::new("test-formats");
    let root = suite.scenario("formats");
    root.file("styles/Team/Filler.yml", FILLER);
    root.file(
        ".ctl/hakkiri.yaml",
        "formats:\n  cfg: md\npackages:\n  - path: styles\nfiles:\n  '*.cfg': {styles: [Team]}\n",
    );
    root.file(
        "through.test.yml",
        "- name: a fence is code\n  format: cfg\n  input: |\n    ```\n    very\n    ```\n  want: ''\n",
    );
    let result = run(root.path(), &["test", "through.test.yml"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines(), ["1 file, 1 passed, 0 failed"]);
    Ok(())
}

/// A project that runs `Team` on `*.txt` and sets `on_unknown`.
fn unknown_project<'a>(suite: &'a Suite, name: &str, word: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("styles/Team/Filler.yml", FILLER);
    root.file(
        ".ctl/hakkiri.yaml",
        format!(
            "on_unknown: {word}\npackages:\n  - path: styles\nfiles:\n  '*.txt': {{styles: [Team]}}\n"
        ),
    );
    root.file(
        "through.test.yml",
        "- name: a very day\n  format: txt\n  input: A very day.\n  want: ''\n",
    );
    root
}

#[test]
fn on_unknown_error_stops_a_case_in_a_format_no_reader_reads() -> TestResult {
    let suite = Suite::new("test-on-unknown-error");
    let root = unknown_project(&suite, "error", "error");
    let result = run(root.path(), &["test", "through.test.yml"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result.out.contains("no reader reads the format `txt`"),
        "{}",
        result.out
    );
    Ok(())
}

#[test]
fn on_unknown_warn_and_ignore_give_a_case_no_alerts() -> TestResult {
    let suite = Suite::new("test-on-unknown-skip");
    for word in ["warn", "ignore"] {
        let root = unknown_project(&suite, word, word);
        let result = run(root.path(), &["test", "through.test.yml"], &[])?;
        assert_eq!(result.code, 0, "{word}: {}{}", result.out, result.err);
        assert_eq!(result.lines(), ["1 file, 1 passed, 0 failed"], "{word}");
    }
    Ok(())
}

#[test]
fn on_unknown_text_reads_the_case_as_plain_text() -> TestResult {
    let suite = Suite::new("test-on-unknown-text");
    let root = unknown_project(&suite, "text", "text");
    let result = run(root.path(), &["test", "through.test.yml"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert!(
        result.out.contains("failed through.test.yml"),
        "{}",
        result.out
    );
    Ok(())
}
