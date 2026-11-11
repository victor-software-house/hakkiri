//! A Vale style loads as it is: a rule may extend another rule, and a rule
//! hakkiri cannot run is reported while the rest of its style runs.
//!
//! Each test writes a project in a temporary directory, runs the binary in it,
//! and reads the exit status and the streams. The last tests run the three
//! upstream styles committed under `tests/fixtures/vale-styles`.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;
use std::path::PathBuf;

mod common;

use common::{Run, run};
use ctl_core::fixture::{Root, Suite};

type TestResult = Result<(), Box<dyn Error>>;

/// A parent that flags three words, as a warning.
const PARENT: &str = "extends: existence\nmessage: \"Parent: found %s\"\nlevel: warning\nscope: text\nignorecase: true\ntokens:\n  - alpha\n  - beta\n  - gamma\ntests:\n  - name: parent case\n    input: alpha\n    contains: Parent\n";

/// A parent that swaps two words, as a suggestion.
const SWAPS: &str = "extends: substitution\nmessage: \"Sub: use %s instead of %s\"\nlevel: suggestion\nswap:\n  swapme: swapped\n  delta: delta2\n";

/// A text every rule here flags: the five words start at columns 1, 7, 12, 18
/// and 24.
const TEXT: &str = "alpha beta gamma delta swapme\n";

/// The config that runs the style `Kid`, from the package `styles`.
const KID: &str = "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Kid]}\n";

/// A project with `Base/Parent.yml`, `Base/Sub.yml` and the config `config`,
/// and `child` as `Kid/Child.yml`.
fn project<'a>(suite: &'a Suite, name: &str, child: &str, config: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("styles/Base/Parent.yml", PARENT);
    root.file("styles/Base/Sub.yml", SWAPS);
    root.file("styles/Kid/Child.yml", child);
    root.file(".ctl/hakkiri.yaml", config);
    root.file("a.md", TEXT);
    root
}

/// Everything a run printed.
fn all(result: &Run) -> String {
    format!("{}{}", result.out, result.err)
}

#[test]
fn a_child_takes_the_fields_of_its_parent() -> TestResult {
    let suite = Suite::new("vale-child");
    let root = project(
        &suite,
        "child",
        "extends: Base.Parent\nmessage: \"Child: found %s\"\nlevel: error\n",
        KID,
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}", all(&result));
    assert_eq!(
        result.lines(),
        [
            "a.md:1:1: error Kid.Child: Child: found alpha",
            "a.md:1:7: error Kid.Child: Child: found beta",
            "a.md:1:12: error Kid.Child: Child: found gamma",
            "3 alerts in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_plus_key_adds_to_a_list_of_the_parent() -> TestResult {
    let suite = Suite::new("vale-plus");
    let root = project(
        &suite,
        "plus",
        "extends: Base.Parent\ntokens+:\n  - delta\n",
        KID,
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert_eq!(
        result.lines(),
        [
            "a.md:1:1: warning Kid.Child: Parent: found alpha",
            "a.md:1:7: warning Kid.Child: Parent: found beta",
            "a.md:1:12: warning Kid.Child: Parent: found gamma",
            "a.md:1:18: warning Kid.Child: Parent: found delta",
            "4 alerts in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_minus_key_removes_an_entry_of_a_list_of_the_parent() -> TestResult {
    let suite = Suite::new("vale-minus");
    let root = project(
        &suite,
        "minus",
        "extends: Base.Parent\ntokens-:\n  - beta\n",
        KID,
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert_eq!(
        result.lines(),
        [
            "a.md:1:1: warning Kid.Child: Parent: found alpha",
            "a.md:1:12: warning Kid.Child: Parent: found gamma",
            "2 alerts in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn plus_and_minus_edit_a_mapping_of_the_parent() -> TestResult {
    let suite = Suite::new("vale-mapping");
    let root = project(
        &suite,
        "mapping",
        "extends: Base.Sub\nswap+:\n  alpha: omega\nswap-:\n  - delta\n",
        KID,
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert_eq!(
        result.lines(),
        [
            "a.md:1:1: suggestion Kid.Child: Sub: use omega instead of alpha",
            "a.md:1:24: suggestion Kid.Child: Sub: use swapped instead of swapme",
            "2 alerts in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_chain_of_three_rules_is_resolved() -> TestResult {
    let suite = Suite::new("vale-chain");
    let root = project(
        &suite,
        "chain",
        "extends: Base.Parent\ntokens+:\n  - delta\n",
        "packages:\n  - path: styles\nfiles:\n  '*.md':\n    styles: [Kid]\n    Kid.Child: off\n",
    );
    root.file(
        "styles/Kid/Grand.yml",
        "extends: Kid.Child\ntokens-:\n  - alpha\nlevel: error\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}", all(&result));
    assert_eq!(
        result.lines(),
        [
            "a.md:1:7: error Kid.Grand: Parent: found beta",
            "a.md:1:12: error Kid.Grand: Parent: found gamma",
            "a.md:1:18: error Kid.Grand: Parent: found delta",
            "3 alerts in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_child_reads_the_scope_of_its_parent() -> TestResult {
    let suite = Suite::new("vale-scope");
    let root = project(&suite, "scope", "extends: Base.Scoped\nlevel: error\n", KID);
    root.file(
        "styles/Base/Scoped.yml",
        "extends: existence\nmessage: \"Heading: found %s\"\nscope: heading\ntokens: [alpha]\n",
    );
    root.file("a.md", "# alpha\n\nalpha here.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}", all(&result));
    assert_eq!(
        result.lines(),
        [
            "a.md:1:3: error Kid.Child: Heading: found alpha",
            "1 alert in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_parent_may_sit_in_a_nested_folder() -> TestResult {
    let suite = Suite::new("vale-nested");
    let root = project(&suite, "nested", "extends: Base.Deep.Parent\n", KID);
    root.file("styles/Base/Deep/Parent.yml", PARENT);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert_eq!(result.lines().last(), Some(&"3 alerts in 1 file"));
    Ok(())
}

#[test]
fn a_parent_may_sit_in_an_underscore_folder() -> TestResult {
    let suite = Suite::new("vale-inert");
    let root = project(&suite, "inert", "extends: Base._shared.Parent\n", KID);
    root.file("styles/Base/_shared/Parent.yml", PARENT);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert_eq!(
        result.lines(),
        [
            "a.md:1:1: warning Kid.Child: Parent: found alpha",
            "a.md:1:7: warning Kid.Child: Parent: found beta",
            "a.md:1:12: warning Kid.Child: Parent: found gamma",
            "3 alerts in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_parent_may_sit_in_another_package() -> TestResult {
    let suite = Suite::new("vale-package");
    let root = suite.scenario("package");
    root.file("one/Kid/Child.yml", "extends: Base.Parent\nlevel: error\n");
    root.file("two/Base/Parent.yml", PARENT);
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: one\n  - path: two\nfiles:\n  '*.md': {styles: [Kid]}\n",
    );
    root.file("a.md", TEXT);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}", all(&result));
    assert_eq!(result.lines().last(), Some(&"3 alerts in 1 file"));
    Ok(())
}

#[test]
fn a_child_keeps_its_own_cases_and_drops_those_of_its_parent() -> TestResult {
    let suite = Suite::new("vale-cases");
    let root = project(
        &suite,
        "cases",
        "extends: Base.Parent\ntokens+:\n  - delta\ntests:\n  - name: child case\n    input: delta\n    contains: Kid.Child\n",
        KID,
    );
    root.file("styles/Base/Sub.yml", SWAPS);
    let result = run(root.path(), &["test", "styles"], &[])?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert_eq!(result.lines(), ["2 files, 2 passed, 0 failed"]);
    Ok(())
}

/// The refusals Vale makes of a chain: the child, and the text of the error.
const CHAIN_FAULTS: [(&str, &str, &str); 6] = [
    (
        "both",
        "extends: Base.Parent\ntokens:\n  - gamma\ntokens+:\n  - delta\n",
        "tokens+: both replaces `tokens` and edits it; pick one",
    ),
    (
        "missing-entry",
        "extends: Base.Parent\ntokens-:\n  - missing\n",
        "tokens-: the parent has no entry `missing` to remove",
    ),
    (
        "cycle",
        "extends: Kid.Child\nlevel: error\n",
        "inheritance cycle: `Kid.Child` is already in this chain",
    ),
    (
        "built-in",
        "extends: Vale.Terms\n",
        "`Vale.Terms` is a built-in rule generated at runtime; it has no file to extend",
    ),
    (
        "absent",
        "extends: Base.Absent\n",
        "`Base.Absent` does not exist on the search path (looked for `Base/Absent.yml`)",
    ),
    (
        "no-parent",
        "extends: existence\nmessage: x\ntokens: [a]\ntokens+:\n  - b\n",
        "tokens+",
    ),
];

#[test]
fn a_chain_vale_refuses_is_refused() -> TestResult {
    let suite = Suite::new("vale-faults");
    for (name, child, message) in CHAIN_FAULTS {
        let root = project(&suite, name, child, KID);
        let result = run(root.path(), &["check"], &[])?;
        assert_eq!(result.code, 2, "{name}: {}", all(&result));
        assert!(all(&result).contains(message), "{name}: {}", all(&result));
        assert!(
            all(&result).contains("Kid/Child.yml"),
            "{name}: {}",
            all(&result)
        );
    }
    Ok(())
}

#[test]
fn a_chain_over_ten_rules_is_refused() -> TestResult {
    let suite = Suite::new("vale-deep");
    let root = project(&suite, "deep", "extends: Chain.R0\n", KID);
    for step in 0..11 {
        root.file(
            format!("styles/Chain/R{step}.yml"),
            format!("extends: Chain.R{}\n", step + 1),
        );
    }
    root.file("styles/Chain/R11.yml", PARENT);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}", all(&result));
    assert!(
        all(&result).contains("the inheritance chain is over 10 rules deep"),
        "{}",
        all(&result)
    );
    Ok(())
}

/// A rule hakkiri cannot run: a script.
const SCRIPT: &str =
    "extends: script\nmessage: Script\nscript: |\n  text := import(\"text\")\n  matches := []\n";

/// A rule hakkiri cannot run: a selector over rendered HTML.
const DOC: &str = "extends: existence\nmessage: Doc\nscope: doc(h1)\ntokens: [alpha]\n";

/// A project whose style `Kid` holds the rule `Parent`, as a warning, and the
/// rule `Cannot` from `cannot`, with `config`.
fn cannot<'a>(suite: &'a Suite, name: &str, cannot: &str, config: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("styles/Kid/Parent.yml", PARENT);
    root.file("styles/Kid/Cannot.yml", cannot);
    root.file(".ctl/hakkiri.yaml", config);
    root.file("a.md", TEXT);
    root
}

#[test]
fn a_rule_hakkiri_cannot_run_fails_the_run_while_the_rest_of_its_style_runs() -> TestResult {
    let suite = Suite::new("vale-script");
    for (name, rule) in [("script", SCRIPT), ("doc", DOC)] {
        let root = cannot(&suite, name, rule, KID);
        let result = run(root.path(), &["check"], &[])?;
        assert_eq!(result.code, 2, "{name}: {}", all(&result));
        let lines = result.lines();
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.contains("Kid.Parent"))
                .count(),
            3,
            "{name}: {}",
            all(&result)
        );
        assert!(
            lines.iter().any(|line| line.starts_with("failed ")
                && line.contains("Kid.Cannot")
                && line.contains("Cannot.yml")
                && line.contains("set `Kid.Cannot: off` under `files` to leave it out")),
            "{name}: {}",
            all(&result)
        );
    }
    Ok(())
}

#[test]
fn off_leaves_out_a_rule_hakkiri_cannot_run() -> TestResult {
    let suite = Suite::new("vale-script-off");
    let root = cannot(
        &suite,
        "off",
        SCRIPT,
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Kid], Kid.Cannot: off}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert_eq!(result.lines().last(), Some(&"3 alerts in 1 file"));
    assert!(!all(&result).contains("failed"), "{}", all(&result));
    Ok(())
}

#[test]
fn a_rule_hakkiri_cannot_run_is_ignored_in_a_style_no_glob_lists() -> TestResult {
    let suite = Suite::new("vale-script-unlisted");
    let root = cannot(
        &suite,
        "unlisted",
        SCRIPT,
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Other]}\n",
    );
    root.file(
        "styles/Other/Plain.yml",
        "extends: existence\nmessage: Plain\ntokens: [swapme]\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert_eq!(
        result.lines(),
        ["a.md:1:24: warning Other.Plain: Plain", "1 alert in 1 file"]
    );
    Ok(())
}

#[test]
fn a_level_or_on_does_not_make_a_rule_hakkiri_cannot_run_run() -> TestResult {
    let suite = Suite::new("vale-script-level");
    for (name, setting) in [("level", "warning"), ("on", "on")] {
        let root = cannot(
            &suite,
            name,
            SCRIPT,
            &format!(
                "packages:\n  - path: styles\nfiles:\n  '*.md': {{styles: [Other], Kid.Cannot: {setting}}}\n"
            ),
        );
        root.file(
            "styles/Other/Plain.yml",
            "extends: existence\nmessage: Plain\ntokens: [swapme]\n",
        );
        let result = run(root.path(), &["check"], &[])?;
        assert_eq!(result.code, 2, "{name}: {}", all(&result));
        assert!(
            all(&result).contains("set `Kid.Cannot: off` under `files` to leave it out"),
            "{name}: {}",
            all(&result)
        );
    }
    Ok(())
}

#[test]
fn a_child_of_a_script_rule_is_a_rule_hakkiri_cannot_run() -> TestResult {
    let suite = Suite::new("vale-script-child");
    let root = cannot(&suite, "child", "extends: Kid.Base\nmessage: Child\n", KID);
    root.file("styles/Kid/Base.yml", SCRIPT);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}", all(&result));
    assert!(all(&result).contains("Kid.Cannot"), "{}", all(&result));
    Ok(())
}

#[test]
fn the_cases_of_a_rule_hakkiri_cannot_run_are_reported_as_not_run() -> TestResult {
    let suite = Suite::new("vale-script-cases");
    let root = cannot(
        &suite,
        "cases",
        &format!("{SCRIPT}tests:\n  - name: script case\n    input: alpha\n    contains: Script\n"),
        KID,
    );
    let result = run(root.path(), &["test", "styles"], &[])?;
    assert_eq!(result.code, 2, "{}", all(&result));
    assert!(
        all(&result).contains("its cases did not run"),
        "{}",
        all(&result)
    );
    assert!(
        result
            .lines()
            .iter()
            .any(|line| line.ends_with("1 files, 1 passed, 0 failed")
                || line.ends_with("1 file, 1 passed, 0 failed")),
        "{}",
        all(&result)
    );
    Ok(())
}

#[test]
fn a_case_through_a_config_that_lists_a_rule_hakkiri_cannot_run_is_not_run() -> TestResult {
    let suite = Suite::new("vale-script-project-case");
    let root = cannot(&suite, "project-case", SCRIPT, KID);
    root.file(
        "through.test.yml",
        "- name: through the config\n  input: alpha\n  contains: Kid.Parent\n",
    );
    let result = run(root.path(), &["test"], &[])?;
    assert_eq!(result.code, 2, "{}", all(&result));
    assert!(
        all(&result).contains("did not run: Kid.Cannot"),
        "{}",
        all(&result)
    );
    Ok(())
}

/// The committed upstream styles.
fn upstream() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vale-styles")
}

/// A project whose config loads the packages of the committed styles.
fn upstream_project<'a>(suite: &'a Suite, name: &str, files: &str) -> Root<'a> {
    let root = suite.scenario(name);
    let base = upstream();
    root.file(
        ".ctl/hakkiri.yaml",
        format!(
            "packages:\n  - path: '{}'\n  - path: '{}'\nfiles:\n{files}",
            base.join("voices").display(),
            base.join("upstream").display()
        ),
    );
    root
}

#[test]
fn voices_and_std_pass_their_own_cases() -> TestResult {
    let suite = Suite::new("vale-upstream-cases");
    let root = upstream_project(&suite, "cases", "  '*.md': {styles: []}\n");
    let base = upstream();
    let voices = base.join("voices");
    let std = base.join("upstream/Std");
    let result = run(
        root.path(),
        &[
            "test",
            "--coverage",
            &voices.display().to_string(),
            &std.display().to_string(),
        ],
        &[],
    )?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert_eq!(
        result.lines(),
        ["48 files, 103 passed, 0 failed, 0 uncovered"]
    );
    Ok(())
}

/// A sentence far above grade 8.
const DENSE: &str = "The institutional reorganization necessitated comprehensive interdepartmental communication regarding organizational responsibilities.\n";

#[test]
fn readability_runs_with_its_script_rule_turned_off() -> TestResult {
    let suite = Suite::new("vale-upstream-readability");
    let root = upstream_project(
        &suite,
        "readability",
        "  '*.md': {styles: [Readability], Readability.Polysyllables: off}\n",
    );
    root.file("a.md", DENSE);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", all(&result));
    assert!(
        result
            .lines()
            .iter()
            .any(|line| line.contains("Readability.FleschKincaid")),
        "{}",
        all(&result)
    );
    assert!(!all(&result).contains("failed"), "{}", all(&result));
    Ok(())
}

#[test]
fn readability_fails_for_its_script_rule_until_it_is_turned_off() -> TestResult {
    let suite = Suite::new("vale-upstream-script");
    let root = upstream_project(&suite, "script", "  '*.md': {styles: [Readability]}\n");
    root.file("a.md", DENSE);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}", all(&result));
    let failed: Vec<&str> = result
        .lines()
        .into_iter()
        .filter(|line| line.starts_with("failed "))
        .collect();
    assert_eq!(failed.len(), 1, "{}", all(&result));
    assert!(
        failed[0].contains("Readability.Polysyllables"),
        "{}",
        all(&result)
    );
    assert!(
        result
            .lines()
            .iter()
            .any(|line| line.contains("Readability.FleschKincaid")),
        "{}",
        all(&result)
    );
    Ok(())
}
