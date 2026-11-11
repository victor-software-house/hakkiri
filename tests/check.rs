//! `hakkiri check`: the scenarios of the check-command change, and the
//! project-config scenarios it makes real.
//!
//! Each test writes a project in a temporary directory, runs the binary in it,
//! and reads the exit status and the two streams.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

use std::error::Error;

mod common;

use common::run;
use ctl_core::fixture::{Root, Suite};
use serde_json::Value;

type TestResult = Result<(), Box<dyn Error>>;

/// A rule that warns on `very`.
const FILLER: &str =
    "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\ntokens: [very]\n";

/// A rule that fails on `terrible`.
const HARSH: &str =
    "extends: existence\nmessage: \"Never '%s'.\"\nlevel: error\ntokens: [terrible]\n";

/// A rule that only suggests against `maybe`.
const HINT: &str =
    "extends: existence\nmessage: \"Say '%s' less.\"\nlevel: suggestion\ntokens: [maybe]\n";

/// A config with one package and `*.md` under the style `Team`.
const CONFIG: &str = "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n";

/// A project with the style `Team` in `styles/` and `config` as its config.
fn project<'a>(suite: &'a Suite, name: &str, config: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("styles/Team/Filler.yml", FILLER);
    root.file("styles/Team/Harsh.yml", HARSH);
    root.file("styles/Team/Hint.yml", HINT);
    root.file(".ctl/hakkiri.yaml", config);
    root
}

#[test]
fn a_directory_and_a_file() -> TestResult {
    let suite = Suite::new("check-paths");
    let root = project(&suite, "paths", CONFIG);
    root.file("docs/a.md", "This is very good.\n");
    root.file("docs/sub/b.md", "This is very good.\n");
    root.file("README.md", "This is very good.\n");
    root.file("other.md", "This is very good.\n");
    let result = run(root.path(), &["check", "docs", "README.md"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "README.md:1:9: warning Team.Filler: Remove 'very'.",
            "docs/a.md:1:9: warning Team.Filler: Remove 'very'.",
            "docs/sub/b.md:1:9: warning Team.Filler: Remove 'very'.",
            "3 alerts in 3 files",
        ]
    );
    Ok(())
}

#[test]
fn no_argument_lints_the_current_directory() -> TestResult {
    let suite = Suite::new("check-cwd");
    let root = project(&suite, "cwd", CONFIG);
    root.file("a.md", "Fine.\n");
    root.file("docs/b.md", "Fine.\n");
    root.file("notes.txt", "very\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines(), ["0 alerts in 2 files"]);
    Ok(())
}

#[test]
fn folders_a_walk_leaves_alone() -> TestResult {
    let suite = Suite::new("check-skipped");
    let root = project(&suite, "skipped", CONFIG);
    root.file("node_modules/x.md", "This is very good.\n");
    root.file(".git/y.md", "This is very good.\n");
    root.file("styles/z.md", "This is very good.\n");
    root.file(".github/w.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            ".github/w.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file",
        ]
    );
    Ok(())
}

#[test]
fn a_file_no_language_reads() -> TestResult {
    let suite = Suite::new("check-language");
    let root = project(
        &suite,
        "language",
        "packages:\n  - path: styles\nfiles:\n  '*': {styles: [Team]}\n",
    );
    root.file("logo.png", "not text");
    root.file("a.md", "Fine.\n");
    let walked = run(root.path(), &["check"], &[])?;
    assert_eq!(walked.code, 0, "{}", walked.err);
    assert_eq!(walked.lines(), ["0 alerts in 1 file"]);
    root.file("notes.txt", "very\n");
    let named = run(root.path(), &["check", "notes.txt"], &[])?;
    assert_eq!(named.code, 2, "{}", named.out);
    assert!(
        named.out.contains("failed notes.txt: no language reads"),
        "{}",
        named.out
    );
    Ok(())
}

#[test]
fn one_broken_mdx_file() -> TestResult {
    let suite = Suite::new("check-broken");
    let root = project(
        &suite,
        "broken",
        "packages:\n  - path: styles\nfiles:\n  '*.{md,mdx}': {styles: [Team]}\n",
    );
    root.file("bad.mdx", "<div>\n");
    root.file("good.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}", result.err);
    let lines = result.lines();
    assert_eq!(
        lines.first(),
        Some(&"good.md:1:9: warning Team.Filler: Remove 'very'.")
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("failed bad.mdx: ")),
        "{lines:?}"
    );
    assert_eq!(lines.last(), Some(&"1 alert in 1 file"));
    Ok(())
}

#[test]
fn a_clean_run() -> TestResult {
    let suite = Suite::new("check-clean");
    let root = project(&suite, "clean", CONFIG);
    for name in ["a.md", "b.md", "c.md"] {
        root.file(name, "All fine here.\n");
    }
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines(), ["0 alerts in 3 files"]);
    Ok(())
}

#[test]
fn json_holds_no_escape_byte_under_forced_colour() -> TestResult {
    let suite = Suite::new("check-json");
    let root = project(&suite, "json", CONFIG);
    root.file("a.md", "This is very good.\n");
    let forced = [
        ("CLICOLOR_FORCE", "1"),
        ("FORCE_COLOR", "3"),
        ("COLORTERM", "truecolor"),
    ];
    let cases: [&[&str]; 2] = [
        &["check", "--format", "json"],
        &["check", "--bogus", "--format", "json"],
    ];
    for args in cases {
        let result = run(root.path(), args, &forced)?;
        assert!(
            !result.out.contains('\u{1b}') && !result.err.contains('\u{1b}'),
            "{args:?}: {}{}",
            result.out,
            result.err
        );
        let json: Value = serde_json::from_str(&result.out)?;
        if args.contains(&"--bogus") {
            assert_eq!(result.code, 2);
            continue;
        }
        assert_eq!(result.code, 0);
        let alerts = json["alerts"].as_array().ok_or("no alerts array")?;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0]["path"], "a.md");
        assert_eq!(alerts[0]["line"], 1);
        assert_eq!(alerts[0]["column"], 9);
        assert_eq!(alerts[0]["end_line"], 1);
        assert_eq!(alerts[0]["end_column"], 12);
        assert_eq!(alerts[0]["level"], "warning");
        assert_eq!(alerts[0]["rule"], "Team.Filler");
        assert_eq!(alerts[0]["match"], "very");
        assert_eq!(json["files"], 1);
        assert_eq!(json["failures"], Value::Array(Vec::new()));
    }
    Ok(())
}

#[test]
fn the_exit_status_follows_the_fail_level() -> TestResult {
    let suite = Suite::new("check-exit");
    let root = project(&suite, "exit", CONFIG);
    root.file("a.md", "This is very good.\n");
    assert_eq!(run(root.path(), &["check"], &[])?.code, 0);
    let lowered = run(root.path(), &["check", "--fail-level", "warning"], &[])?;
    assert_eq!(lowered.code, 1, "{}", lowered.err);
    let suggestion = run(root.path(), &["check", "--fail-level", "suggestion"], &[])?;
    assert_eq!(suggestion.code, 1);
    Ok(())
}

#[test]
fn an_alert_the_config_hides_does_not_fail() -> TestResult {
    let suite = Suite::new("check-hidden");
    let root = project(
        &suite,
        "hidden",
        "level: error\npackages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check", "--fail-level", "warning"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines(), ["0 alerts in 1 file"]);
    Ok(())
}

#[test]
fn a_failure_wins_over_an_alert() -> TestResult {
    let suite = Suite::new("check-failure");
    let root = project(
        &suite,
        "failure",
        "packages:\n  - path: styles\nfiles:\n  '*.{md,mdx}': {styles: [Team]}\n",
    );
    root.file("bad.mdx", "<div>\n");
    root.file("hard.md", "This is terrible.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}", result.err);
    assert!(
        result
            .out
            .contains("hard.md:1:9: error Team.Harsh: Never 'terrible'."),
        "{}",
        result.out
    );
    assert!(result.out.contains("failed bad.mdx: "), "{}", result.out);
    Ok(())
}

#[test]
fn a_broken_config_stops_the_run() -> TestResult {
    let suite = Suite::new("check-config");
    let root = project(
        &suite,
        "config",
        "packages:\n  - path: styles\nMinAlertLevel: warning\n",
    );
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2);
    assert_eq!(result.out, "");
    assert!(
        result
            .err
            .contains("failed .ctl/hakkiri.yaml: line 3: unknown field `MinAlertLevel`"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_missing_config_stops_the_run() -> TestResult {
    let suite = Suite::new("check-missing");
    let root = suite.scenario("missing");
    root.file("a.md", "Fine.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2);
    assert!(result.err.contains(".ctl/hakkiri.yaml"), "{}", result.err);
    Ok(())
}

#[test]
fn a_variable_sets_a_flag_and_the_flag_wins() -> TestResult {
    let suite = Suite::new("check-env");
    let root = project(&suite, "env", CONFIG);
    root.file("a.md", "This is very good.\n");
    let from_variable = run(
        root.path(),
        &["check"],
        &[("HAKKIRI_FAIL_LEVEL", "warning")],
    )?;
    assert_eq!(from_variable.code, 1, "{}", from_variable.err);
    let flag_wins = run(
        root.path(),
        &["check", "--fail-level", "error"],
        &[("HAKKIRI_FAIL_LEVEL", "warning")],
    )?;
    assert_eq!(flag_wins.code, 0, "{}", flag_wins.err);
    let empty = run(root.path(), &["check"], &[("HAKKIRI_FAIL_LEVEL", "")])?;
    assert_eq!(empty.code, 0, "{}", empty.err);
    Ok(())
}

#[test]
fn the_variable_names_the_config() -> TestResult {
    let suite = Suite::new("check-config-env");
    let root = project(&suite, "config-env", "files: {}\n");
    root.file(".ctl/other.yaml", CONFIG);
    root.file("a.md", "This is very good.\n");
    let result = run(
        root.path(),
        &["check"],
        &[("HAKKIRI_CONFIG", ".ctl/other.yaml")],
    )?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines().last(), Some(&"1 alert in 1 file"));
    Ok(())
}

/// A config whose one glob is `docs/**`.
const DOCS_CONFIG: &str = "packages:\n  - path: styles\nfiles:\n  'docs/**': {styles: [Team]}\n";

#[test]
fn a_dot_slash_argument_matches_the_glob() -> TestResult {
    let suite = Suite::new("check-dot-slash");
    let root = project(&suite, "dot-slash", DOCS_CONFIG);
    root.file("docs/a.md", "This is very good.\n");
    let result = run(root.path(), &["check", "./docs"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines().first(),
        Some(&"docs/a.md:1:9: warning Team.Filler: Remove 'very'.")
    );
    Ok(())
}

#[test]
fn an_absolute_argument_matches_the_glob() -> TestResult {
    let suite = Suite::new("check-absolute");
    let root = project(&suite, "absolute", DOCS_CONFIG);
    let file = root.file("docs/a.md", "This is very good.\n");
    let argument = file.to_string_lossy().into_owned();
    let result = run(root.path(), &["check", &argument], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines().last(), Some(&"1 alert in 1 file"));
    Ok(())
}

#[test]
fn a_parent_argument_matches_the_glob() -> TestResult {
    let suite = Suite::new("check-parent");
    let root = project(&suite, "parent", DOCS_CONFIG);
    root.file("docs/a.md", "This is very good.\n");
    root.file("other/b.md", "Fine.\n");
    let result = run(
        &root.path().join("other"),
        &["check", "../docs/a.md"],
        &[("HAKKIRI_CONFIG", "../.ctl/hakkiri.yaml")],
    )?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines().last(), Some(&"1 alert in 1 file"));
    Ok(())
}

/// A project in `project/` whose one glob is `**`, beside `outside.md`.
fn nested<'a>(suite: &'a Suite, name: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("project/styles/Team/Filler.yml", FILLER);
    root.file(
        "project/.ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  '**': {styles: [Team]}\n",
    );
    root.file("project/in.md", "This is very good.\n");
    root.file("outside.md", "This is very good.\n");
    root
}

#[test]
fn a_file_outside_the_root_draws_a_warning() -> TestResult {
    let suite = Suite::new("check-outside");
    let root = nested(&suite, "outside");
    let result = run(
        &root.path().join("project"),
        &["check", "in.md", "../outside.md"],
        &[],
    )?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines().last(), Some(&"1 alert in 1 file"));
    assert_eq!(result.err, OUTSIDE_WARNING);
    Ok(())
}

const OUTSIDE_WARNING: &str =
    "warning ../outside.md: outside the project root, so no glob matches it and it is not linted\n";

#[test]
fn a_mixed_run_exits_by_the_alert_level() -> TestResult {
    let suite = Suite::new("check-mixed");
    let root = nested(&suite, "mixed");
    let result = run(
        &root.path().join("project"),
        &["check", "--fail-level", "warning", "in.md", "../outside.md"],
        &[],
    )?;
    assert_eq!(result.code, 1);
    assert_eq!(result.err, OUTSIDE_WARNING);
    assert_eq!(result.lines().last(), Some(&"1 alert in 1 file"));
    Ok(())
}

#[test]
fn json_keeps_the_warning_on_standard_error() -> TestResult {
    let suite = Suite::new("check-outside-json");
    let root = nested(&suite, "outside-json");
    let project = root.path().join("project");
    let mixed = run(
        &project,
        &["check", "--format", "json", "in.md", "../outside.md"],
        &[],
    )?;
    assert_eq!(mixed.code, 0, "{}", mixed.err);
    assert_eq!(mixed.err, OUTSIDE_WARNING);
    let json: Value = serde_json::from_str(&mixed.out)?;
    assert_eq!(json["files"], 1);
    assert_eq!(json["alerts"].as_array().map(Vec::len), Some(1));
    assert_eq!(json["failures"].as_array().map(Vec::len), Some(0));
    let none = run(
        &project,
        &["check", "--format", "json", "../outside.md"],
        &[],
    )?;
    assert_eq!(none.code, 2);
    assert_eq!(none.err, OUTSIDE_WARNING);
    let json: Value = serde_json::from_str(&none.out)?;
    assert_eq!(json["failures"][0]["config"], ".ctl/hakkiri.yaml");
    assert!(
        json["failures"][0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("project root")),
        "{}",
        none.out
    );
    Ok(())
}

#[test]
fn quiet_keeps_the_warning() -> TestResult {
    let suite = Suite::new("check-outside-quiet");
    let root = nested(&suite, "outside-quiet");
    let result = run(
        &root.path().join("project"),
        &["check", "--quiet", "in.md", "../outside.md"],
        &[],
    )?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.out, "");
    assert_eq!(result.err, OUTSIDE_WARNING);
    Ok(())
}

#[test]
fn a_missing_package_folder_is_a_config_error() -> TestResult {
    let suite = Suite::new("check-no-package");
    let root = suite.scenario("no-package");
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file("a.md", "Fine.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2);
    assert!(
        result.err.contains("line 2: path: `styles`"),
        "{}",
        result.err
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_walk_does_not_enter_a_symlink_to_a_package_folder() -> TestResult {
    let suite = Suite::new("check-link-package");
    let root = project(&suite, "link-package", CONFIG);
    root.file("docs/a.md", "This is very good.\n");
    root.file("styles/Team/notes.md", "This is very good.\n");
    std::os::unix::fs::symlink("../styles", root.path().join("docs/styles"))?;
    let walked = run(root.path(), &["check", "docs"], &[])?;
    assert_eq!(walked.code, 0, "{}", walked.err);
    assert_eq!(walked.lines().last(), Some(&"1 alert in 1 file"));
    let named = run(root.path(), &["check", "styles"], &[])?;
    assert_eq!(named.code, 0, "{}", named.err);
    assert_eq!(named.lines().last(), Some(&"1 alert in 1 file"));
    Ok(())
}

#[test]
fn no_path_under_the_root_exits_2() -> TestResult {
    let suite = Suite::new("check-none-under");
    let root = nested(&suite, "none-under");
    let result = run(
        &root.path().join("project"),
        &["check", "../outside.md"],
        &[],
    )?;
    assert_eq!(result.code, 2);
    assert!(result.err.contains("../outside.md"), "{}", result.err);
    assert!(
        result
            .out
            .contains("failed .ctl/hakkiri.yaml: no path given lies under the project root"),
        "{}",
        result.out
    );
    Ok(())
}

#[test]
fn a_config_elsewhere_puts_the_current_directory_outside_the_root() -> TestResult {
    let suite = Suite::new("check-config-elsewhere");
    let root = nested(&suite, "config-elsewhere");
    root.file("other/hakkiri.yaml", "files: {}\n");
    let result = run(
        &root.path().join("project"),
        &["check"],
        &[("HAKKIRI_CONFIG", "../other/hakkiri.yaml")],
    )?;
    assert_eq!(result.code, 2);
    assert!(
        result
            .err
            .starts_with("warning .: outside the project root"),
        "{}",
        result.err
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_root_reached_through_a_symlink_still_matches() -> TestResult {
    let suite = Suite::new("check-root-link");
    let root = nested(&suite, "root-link");
    std::os::unix::fs::symlink(root.path().join("project"), root.path().join("link"))?;
    let result = run(
        root.path(),
        &["check", "link/in.md"],
        &[("HAKKIRI_CONFIG", "link/.ctl/hakkiri.yaml")],
    )?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines().last(), Some(&"1 alert in 1 file"));
    assert_eq!(result.err, "");
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_file_symlink_matches_by_the_name_it_is_reached_by() -> TestResult {
    let suite = Suite::new("check-file-link");
    let root = suite.scenario("file-link");
    root.file("styles/Team/Filler.yml", FILLER);
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  'docs/**': {styles: [Team]}\n",
    );
    root.file("shared/a.md", "This is very good.\n");
    root.file("outside/b.md", "This is very good.\n");
    std::fs::create_dir_all(root.path().join("docs"))?;
    std::os::unix::fs::symlink("../shared/a.md", root.path().join("docs/a.md"))?;
    std::os::unix::fs::symlink("../outside/b.md", root.path().join("docs/b.md"))?;
    let result = run(root.path(), &["check", "docs", "shared/a.md"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "docs/a.md:1:9: warning Team.Filler: Remove 'very'.",
            "docs/b.md:1:9: warning Team.Filler: Remove 'very'.",
            "2 alerts in 2 files",
        ]
    );
    Ok(())
}

#[test]
fn a_walk_from_another_folder_skips_the_package_folders() -> TestResult {
    let suite = Suite::new("check-walk-skip");
    let root = project(
        &suite,
        "walk-skip",
        "packages:\n  - path: styles\nfiles:\n  '**/*.md': {styles: [Team]}\n",
    );
    root.file("docs/a.md", "This is very good.\n");
    root.file("styles/Team/notes.md", "This is very good.\n");
    root.file("other/.keep", "");
    let result = run(
        &root.path().join("other"),
        &["check", ".."],
        &[("HAKKIRI_CONFIG", "../.ctl/hakkiri.yaml")],
    )?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "../docs/a.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file",
        ]
    );
    Ok(())
}

#[test]
fn a_file_named_two_ways_is_linted_once() -> TestResult {
    let suite = Suite::new("check-twice");
    let root = project(&suite, "twice", CONFIG);
    root.file("docs/a.md", "This is very good.\n");
    root.file("docs/sub/.keep", "");
    let result = run(
        &root.path().join("docs/sub"),
        &["check", "../a.md", "..", "../../docs/a.md"],
        &[("HAKKIRI_CONFIG", "../../.ctl/hakkiri.yaml")],
    )?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "../a.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file"
        ]
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_walk_from_a_folder_that_holds_the_root_leaves_the_siblings_alone() -> TestResult {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    let suite = Suite::new("check-prune");
    let root = nested(&suite, "prune");
    let locked = root.path().join("sibling/locked");
    std::fs::create_dir_all(&locked)?;
    std::fs::set_permissions(&locked, Permissions::from_mode(0o000))?;
    let result = run(&root.path().join("project"), &["check", ".."], &[]);
    std::fs::set_permissions(&locked, Permissions::from_mode(0o755))?;
    let result = result?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(result.lines().last(), Some(&"1 alert in 1 file"));
    Ok(())
}

#[test]
fn a_missing_path_is_one_failure() -> TestResult {
    let suite = Suite::new("check-missing-path");
    let root = project(&suite, "missing-path", CONFIG);
    root.file("a.md", "Fine.\n");
    let result = run(root.path(), &["check", "typo.md"], &[])?;
    assert_eq!(result.code, 2);
    assert!(result.out.contains("failed typo.md:"), "{}", result.out);
    assert!(!result.out.contains("project root"), "{}", result.out);
    Ok(())
}

#[test]
fn a_package_path_is_read_from_the_root() -> TestResult {
    let suite = Suite::new("check-root-package");
    let root = project(&suite, "root-package", DOCS_CONFIG);
    root.file("docs/a.md", "This is very good.\n");
    root.file("other/.keep", "");
    let result = run(
        &root.path().join("other"),
        &["check", "../docs"],
        &[("HAKKIRI_CONFIG", "../.ctl/hakkiri.yaml")],
    )?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines().last(), Some(&"1 alert in 1 file"));
    Ok(())
}

#[test]
fn a_level_hides_the_lower_alerts() -> TestResult {
    let suite = Suite::new("check-level");
    let root = project(
        &suite,
        "level",
        "level: warning\npackages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file("a.md", "This is very good, maybe.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file",
        ]
    );
    Ok(())
}

#[test]
fn two_globs_apply_in_order() -> TestResult {
    let suite = Suite::new("check-globs");
    let root = project(
        &suite,
        "globs",
        "packages:\n  - path: styles\nfiles:\n  '*.{md,rs}': {styles: [Team]}\n  '*.rs': {Team.Filler: off}\n",
    );
    root.file("a.md", "This is very good.\n");
    root.file("a.rs", "// This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 2 files",
        ]
    );
    Ok(())
}

#[test]
fn a_file_no_glob_matches_is_not_linted() -> TestResult {
    let suite = Suite::new("check-unmatched");
    let root = project(
        &suite,
        "unmatched",
        "packages:\n  - path: styles\nfiles:\n  'docs/*.md': {styles: [Team]}\n",
    );
    root.file("docs/a.md", "This is very good.\n");
    root.file("README.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(
        result.lines(),
        [
            "docs/a.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file",
        ]
    );
    Ok(())
}

#[test]
fn an_empty_styles_list_skips_the_files() -> TestResult {
    let suite = Suite::new("check-empty");
    let root = project(
        &suite,
        "empty",
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n  CHANGELOG.md: {styles: []}\n",
    );
    root.file("README.md", "This is very good.\n");
    root.file("CHANGELOG.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(
        result.lines(),
        [
            "README.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file",
        ]
    );
    Ok(())
}

#[test]
fn a_rule_name_sets_the_level() -> TestResult {
    let suite = Suite::new("check-rule-level");
    let root = project(
        &suite,
        "rule-level",
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team], Team.Filler: error}\n",
    );
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: error Team.Filler: Remove 'very'.",
            "1 alert in 1 file",
        ]
    );
    Ok(())
}

#[test]
fn a_rule_name_ends_at_the_first_dot_of_its_file_name() -> TestResult {
    let suite = Suite::new("check-first-dot");
    let root = suite.scenario("first-dot");
    root.file("styles/Team/Terms.custom.yml", FILLER);
    root.file(".ctl/hakkiri.yaml", CONFIG);
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Terms: Remove 'very'.",
            "1 alert in 1 file",
        ]
    );
    Ok(())
}

#[test]
fn a_rule_in_a_folder_under_a_style_is_named_by_every_folder() -> TestResult {
    let suite = Suite::new("check-nested");
    let root = suite.scenario("nested");
    root.file("styles/Team/Sub/Bar.yml", FILLER);
    root.file("styles/Team/Sub/Deep/Baz.yml", HARSH);
    root.file(
        "styles/Team/Sub/Bar.test.yml",
        "- name: cases\n  input: very\n  contains: Team.Sub.Bar\n",
    );
    root.file("styles/Team/_draft/Old.yml", "not: a rule\n");
    root.file("styles/Team/.hidden/Old.yml", "not: a rule\n");
    root.file(".ctl/hakkiri.yaml", CONFIG);
    root.file("a.md", "This is very terrible.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Sub.Bar: Remove 'very'.",
            "a.md:1:14: error Team.Sub.Deep.Baz: Never 'terrible'.",
            "2 alerts in 1 file",
        ]
    );
    Ok(())
}

#[test]
fn two_files_with_one_rule_name_are_refused() -> TestResult {
    let suite = Suite::new("check-clash");
    let root = suite.scenario("clash");
    root.file("styles/Team/Terms.yml", FILLER);
    root.file("styles/Team/Terms.custom.yml", HARSH);
    root.file(".ctl/hakkiri.yaml", CONFIG);
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}", result.out);
    for part in ["Team.Terms", "Terms.yml", "Terms.custom.yml"] {
        assert!(result.err.contains(part), "{part}: {}", result.err);
    }
    Ok(())
}

#[test]
fn a_rule_name_with_a_bracket_is_refused() -> TestResult {
    let suite = Suite::new("check-bracket");
    let root = suite.scenario("bracket");
    root.file("styles/Team/Weird[max].yml", FILLER);
    root.file(".ctl/hakkiri.yaml", CONFIG);
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}", result.out);
    assert!(result.err.contains("Team.Weird[max]"), "{}", result.err);
    Ok(())
}

/// A config that lists only `Other`, so no glob enables `Team`.
const UNUSED_TEAM: &str = "packages:\n  - path: styles\nfiles:\n  '*.md':\n    styles: [Other]\n";

#[test]
fn a_name_clash_in_a_style_no_glob_lists_is_not_refused() -> TestResult {
    let suite = Suite::new("check-clash-unused");
    let root = suite.scenario("clash-unused");
    root.file("styles/Team/Terms.yml", FILLER);
    root.file("styles/Team/Terms.custom.yml", HARSH);
    root.file("styles/Other/Filler.yml", FILLER);
    root.file(".ctl/hakkiri.yaml", UNUSED_TEAM);
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Other.Filler: Remove 'very'.",
            "1 alert in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_bracket_in_a_style_no_glob_lists_is_not_refused() -> TestResult {
    let suite = Suite::new("check-bracket-unused");
    let root = suite.scenario("bracket-unused");
    root.file("styles/Team/Weird[max].yml", FILLER);
    root.file("styles/Other/Filler.yml", FILLER);
    root.file(".ctl/hakkiri.yaml", UNUSED_TEAM);
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Other.Filler: Remove 'very'.",
            "1 alert in 1 file"
        ]
    );
    Ok(())
}

#[test]
fn a_clash_is_refused_when_any_glob_lists_the_style() -> TestResult {
    let suite = Suite::new("check-clash-later-glob");
    let root = suite.scenario("clash-later-glob");
    root.file("styles/Team/Terms.yml", FILLER);
    root.file("styles/Team/Terms.custom.yml", HARSH);
    root.file("styles/Other/Filler.yml", FILLER);
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Other]}\n  '*.txt': {styles: [Team]}\n",
    );
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}", result.out);
    assert!(result.err.contains("Team.Terms"), "{}", result.err);
    Ok(())
}

#[test]
fn a_misspelled_rule_name_is_refused() -> TestResult {
    let suite = Suite::new("check-misspelled");
    let root = project(
        &suite,
        "misspelled",
        "packages:\n  - path: styles\nfiles:\n  '*.md':\n    styles: [Team]\n    Team.Fillr: off\n",
    );
    root.file("a.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2);
    assert!(
        result
            .err
            .contains("line 6: Team.Fillr: names no rule in any style the config loads"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_package_entry_with_a_path_and_a_url_is_refused() -> TestResult {
    let suite = Suite::new("check-both");
    let root = project(
        &suite,
        "both",
        "packages:\n  - path: styles\n    url: https://example.test/styles.tar.gz\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2);
    assert!(
        result
            .err
            .contains("line 2: packages[0]: an entry holds a `path` or a `url`, never both"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn one_style_in_two_folders_is_refused() -> TestResult {
    let suite = Suite::new("check-twice");
    let root = suite.scenario("twice");
    root.file("a/Team/Filler.yml", FILLER);
    root.file("b/Team/Filler.yml", FILLER);
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: a\n  - path: b\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2);
    assert!(
        result.err.contains("`a/Team` and `b/Team`"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_run_over_many_files_prints_the_same_text_twice() -> TestResult {
    let suite = Suite::new("check-many");
    let root = project(&suite, "many", CONFIG);
    for index in 0..200 {
        root.file(
            format!("dir{}/file{index}.md", index % 20),
            "This is very good.\n",
        );
    }
    let first = run(root.path(), &["check"], &[])?;
    let second = run(root.path(), &["check"], &[])?;
    assert_eq!(first.code, 0, "{}", first.err);
    assert_eq!(first.out, second.out);
    assert_eq!(first.lines().last(), Some(&"200 alerts in 200 files"));
    let lines = first.lines();
    let alerts = lines.get(..200).ok_or("fewer than 200 alerts")?;
    let mut sorted = alerts.to_vec();
    sorted.sort_unstable();
    assert_eq!(alerts, sorted.as_slice());
    Ok(())
}

/// A project in a git repository: `.git` marks the root, and `.gitignore`
/// ignores `build/` and `draft.md`.
fn repository<'a>(suite: &'a Suite, name: &str, config: &str) -> Root<'a> {
    let root = project(suite, name, config);
    root.file(".git/HEAD", "ref: refs/heads/main\n");
    root.file(".gitignore", "build/\ndraft.md\n");
    root.file("a.md", "This is very good.\n");
    root.file("draft.md", "This is very good.\n");
    root.file("build/b.md", "This is very good.\n");
    root
}

/// The one line of a run that lints `a.md` alone.
const ONLY_A: [&str; 2] = [
    "a.md:1:9: warning Team.Filler: Remove 'very'.",
    "1 alert in 1 file",
];

/// The lines of a run that lints `a.md`, `build/b.md` and `draft.md`.
const ALL_THREE: [&str; 4] = [
    "a.md:1:9: warning Team.Filler: Remove 'very'.",
    "build/b.md:1:9: warning Team.Filler: Remove 'very'.",
    "draft.md:1:9: warning Team.Filler: Remove 'very'.",
    "3 alerts in 3 files",
];

#[test]
fn a_walk_skips_what_gitignore_ignores() -> TestResult {
    let suite = Suite::new("check-gitignore");
    let root = repository(&suite, "ignored", CONFIG);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.lines(), ONLY_A);
    Ok(())
}

#[test]
fn a_walk_skips_what_the_exclude_file_ignores() -> TestResult {
    let suite = Suite::new("check-exclude");
    let root = repository(&suite, "exclude", CONFIG);
    root.file(".gitignore", "");
    root.file(".git/info/exclude", "build/\ndraft.md\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.lines(), ONLY_A);
    Ok(())
}

#[test]
fn a_walk_skips_what_the_global_excludes_ignore() -> TestResult {
    let suite = Suite::new("check-global");
    let root = repository(&suite, "global", CONFIG);
    root.file(".gitignore", "");
    let home = suite.scenario("home");
    home.file(".config/git/ignore", "build/\ndraft.md\n");
    let home_path = home.path().to_string_lossy().into_owned();
    let config_path = home.path().join(".config").to_string_lossy().into_owned();
    let result = run(
        root.path(),
        &["check"],
        &[("HOME", &home_path), ("XDG_CONFIG_HOME", &config_path)],
    )?;
    assert_eq!(result.lines(), ONLY_A);
    Ok(())
}

#[test]
fn a_nested_gitignore_applies_below_its_folder() -> TestResult {
    let suite = Suite::new("check-nested-gitignore");
    let root = repository(&suite, "nested-ignore", CONFIG);
    root.file(".gitignore", "");
    root.file("docs/.gitignore", "skip.md\n");
    root.file("docs/skip.md", "This is very good.\n");
    root.file("docs/keep.md", "This is very good.\n");
    root.file("skip.md", "This is very good.\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "build/b.md:1:9: warning Team.Filler: Remove 'very'.",
            "docs/keep.md:1:9: warning Team.Filler: Remove 'very'.",
            "draft.md:1:9: warning Team.Filler: Remove 'very'.",
            "skip.md:1:9: warning Team.Filler: Remove 'very'.",
            "5 alerts in 5 files",
        ]
    );
    Ok(())
}

#[test]
fn a_named_file_is_not_filtered_by_gitignore() -> TestResult {
    let suite = Suite::new("check-named-ignored");
    let root = repository(&suite, "named", CONFIG);
    let result = run(root.path(), &["check", "draft.md", "build/b.md"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "build/b.md:1:9: warning Team.Filler: Remove 'very'.",
            "draft.md:1:9: warning Team.Filler: Remove 'very'.",
            "2 alerts in 2 files",
        ]
    );
    Ok(())
}

#[test]
fn gitignore_false_reads_the_ignored_files() -> TestResult {
    let suite = Suite::new("check-gitignore-off");
    let root = repository(
        &suite,
        "off",
        "gitignore: false\npackages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    let from_config = run(root.path(), &["check"], &[])?;
    assert_eq!(from_config.lines(), ALL_THREE);
    let on = repository(&suite, "off-flag", CONFIG);
    let from_flag = run(on.path(), &["check", "--no-gitignore"], &[])?;
    assert_eq!(from_flag.lines(), ALL_THREE);
    let from_variable = run(on.path(), &["check"], &[("HAKKIRI_GITIGNORE", "false")])?;
    assert_eq!(from_variable.lines(), ALL_THREE);
    let from_value = run(on.path(), &["check", "--gitignore=false"], &[])?;
    assert_eq!(from_value.lines(), ALL_THREE);
    Ok(())
}

#[test]
fn the_flag_then_the_variable_then_the_config_decide_gitignore() -> TestResult {
    let suite = Suite::new("check-gitignore-order");
    let off = "gitignore: false\npackages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n";
    let root = repository(&suite, "order", off);
    let variable = run(root.path(), &["check"], &[("HAKKIRI_GITIGNORE", "true")])?;
    assert_eq!(variable.lines(), ONLY_A, "the variable beats the config");
    let flag = run(
        root.path(),
        &["check", "--gitignore=true"],
        &[("HAKKIRI_GITIGNORE", "false")],
    )?;
    assert_eq!(flag.lines(), ONLY_A, "the flag beats the variable");
    let negated = run(
        root.path(),
        &["check", "--no-gitignore"],
        &[("HAKKIRI_GITIGNORE", "true")],
    )?;
    assert_eq!(
        negated.lines(),
        ALL_THREE,
        "--no-gitignore beats the variable"
    );
    Ok(())
}

#[test]
fn a_config_value_that_is_not_a_boolean_is_refused() -> TestResult {
    let suite = Suite::new("check-gitignore-bad");
    let root = repository(&suite, "bad", "gitignore: maybe\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("failed .ctl/hakkiri.yaml: line 1: invalid boolean"),
        "{}",
        result.err
    );
    Ok(())
}

/// A config whose one glob is `docs/*.md`, so `README.md` has no glob.
const DOCS_ONLY: &str = "packages:\n  - path: styles\nfiles:\n  'docs/*.md': {styles: [Team]}\n";

const README_WARNING: &str = "warning README.md: no glob selects it, so it is not linted\n";

fn unmatched<'a>(suite: &'a Suite, name: &str, config: &str) -> Root<'a> {
    let root = project(suite, name, config);
    root.file("README.md", "This is very good.\n");
    root.file("docs/a.md", "This is very good.\n");
    root
}

#[test]
fn a_named_file_no_glob_selects_draws_a_warning() -> TestResult {
    let suite = Suite::new("check-on-unmatched");
    let root = unmatched(&suite, "warn", DOCS_ONLY);
    let result = run(root.path(), &["check", "README.md", "docs"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "docs/a.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file",
        ]
    );
    assert_eq!(result.err, README_WARNING);
    Ok(())
}

#[test]
fn a_file_found_by_a_walk_that_no_glob_selects_draws_no_warning() -> TestResult {
    let suite = Suite::new("check-walk-unmatched");
    let root = unmatched(&suite, "walked", DOCS_ONLY);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.err, "");
    Ok(())
}

#[test]
fn a_file_named_twice_draws_one_warning() -> TestResult {
    let suite = Suite::new("check-on-unmatched-twice");
    let root = unmatched(&suite, "twice", DOCS_ONLY);
    let result = run(root.path(), &["check", "README.md", "./README.md"], &[])?;
    assert_eq!(result.err, README_WARNING);
    Ok(())
}

#[test]
fn on_unmatched_error_fails_the_run() -> TestResult {
    let suite = Suite::new("check-on-unmatched-error");
    let root = unmatched(&suite, "error", DOCS_ONLY);
    let result = run(
        root.path(),
        &["check", "--on-unmatched", "error", "README.md"],
        &[],
    )?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .out
            .contains("failed README.md: no glob selects it, so it is not linted"),
        "{}",
        result.out
    );
    Ok(())
}

#[test]
fn on_unmatched_ignore_says_nothing() -> TestResult {
    let suite = Suite::new("check-on-unmatched-ignore");
    let root = unmatched(&suite, "ignore", DOCS_ONLY);
    let result = run(
        root.path(),
        &["check", "--on-unmatched", "ignore", "README.md"],
        &[],
    )?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(result.err, "");
    Ok(())
}

#[test]
fn the_flag_then_the_variable_then_the_config_decide_on_unmatched() -> TestResult {
    let suite = Suite::new("check-on-unmatched-order");
    let config = format!("on_unmatched: error\n{DOCS_ONLY}");
    let root = unmatched(&suite, "order", &config);
    let from_config = run(root.path(), &["check", "README.md"], &[])?;
    assert_eq!(from_config.code, 2, "{}", from_config.err);
    let variable = run(
        root.path(),
        &["check", "README.md"],
        &[("HAKKIRI_ON_UNMATCHED", "ignore")],
    )?;
    assert_eq!(variable.code, 0, "the variable beats the config");
    assert_eq!(variable.err, "");
    let flag = run(
        root.path(),
        &["check", "--on-unmatched", "warn", "README.md"],
        &[("HAKKIRI_ON_UNMATCHED", "ignore")],
    )?;
    assert_eq!(flag.code, 0, "{}", flag.err);
    assert_eq!(flag.err, README_WARNING, "the flag beats the variable");
    Ok(())
}

#[test]
fn a_file_a_glob_selects_with_no_rule_to_run_draws_no_warning() -> TestResult {
    let suite = Suite::new("check-on-unmatched-empty");
    let root = project(
        &suite,
        "empty",
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n  CHANGELOG.md: {styles: []}\n",
    );
    root.file("CHANGELOG.md", "This is very good.\n");
    let result = run(root.path(), &["check", "CHANGELOG.md"], &[])?;
    assert_eq!(result.err, "");
    Ok(())
}

#[test]
fn an_unknown_on_unmatched_word_is_refused() -> TestResult {
    let suite = Suite::new("check-on-unmatched-bad");
    let root = project(&suite, "bad", "on_unmatched: loud\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.err.contains("loud"), "{}", result.err);
    let flag = run(root.path(), &["check", "--on-unmatched", "loud"], &[])?;
    assert_eq!(flag.code, 2, "{}{}", flag.out, flag.err);
    Ok(())
}

#[test]
fn the_flag_then_the_variable_then_the_config_decide_the_fail_level() -> TestResult {
    let suite = Suite::new("check-fail-level-order");
    let config = format!("fail_level: warning\n{CONFIG}");
    let root = project(&suite, "order", &config);
    root.file("a.md", "This is very good.\n");
    let from_config = run(root.path(), &["check"], &[])?;
    assert_eq!(from_config.code, 1, "{}", from_config.err);
    let variable = run(root.path(), &["check"], &[("HAKKIRI_FAIL_LEVEL", "error")])?;
    assert_eq!(variable.code, 0, "the variable beats the config");
    let flag = run(
        root.path(),
        &["check", "--fail-level", "warning"],
        &[("HAKKIRI_FAIL_LEVEL", "error")],
    )?;
    assert_eq!(flag.code, 1, "the flag beats the variable");
    Ok(())
}

#[test]
fn an_unknown_fail_level_in_the_config_is_refused() -> TestResult {
    let suite = Suite::new("check-fail-level-bad");
    let root = project(&suite, "bad", "fail_level: loud\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(result.err.contains("loud"), "{}", result.err);
    Ok(())
}

/// A linked git worktree of the project: `.git` is a file that points into the
/// `worktrees` folder of a main repository, as `git worktree add` writes it.
/// The worktree's `.gitignore` ignores `build/`, and the main repository's
/// `info/exclude` holds `exclude`. The main repository comes back with the
/// worktree, because its directory is deleted when it drops.
fn worktree<'a>(suite: &'a Suite, name: &str, exclude: &str) -> (Root<'a>, Root<'a>) {
    let main = suite.scenario(&format!("{name}-main"));
    main.file(".git/HEAD", "ref: refs/heads/main\n");
    main.file(".git/info/exclude", exclude);
    main.file(".git/worktrees/wt/gitdir", "wt/.git\n");
    main.file(".git/worktrees/wt/commondir", "../..\n");
    let gitdir = main.path().join(".git/worktrees/wt");
    let root = project(suite, name, CONFIG);
    root.file(".git", format!("gitdir: {}\n", gitdir.display()));
    root.file(".gitignore", "build/\n");
    root.file("a.md", "This is very good.\n");
    root.file("draft.md", "This is very good.\n");
    root.file("build/b.md", "This is very good.\n");
    (root, main)
}

#[test]
fn a_walk_in_a_git_worktree_skips_what_gitignore_ignores() -> TestResult {
    let suite = Suite::new("check-worktree");
    let (root, _main) = worktree(&suite, "gitignore", "");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "draft.md:1:9: warning Team.Filler: Remove 'very'.",
            "2 alerts in 2 files",
        ]
    );
    Ok(())
}

#[test]
fn a_walk_in_a_git_worktree_skips_what_the_main_exclude_file_ignores() -> TestResult {
    let suite = Suite::new("check-worktree-exclude");
    let (root, _main) = worktree(&suite, "exclude", "draft.md\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 0, "{}", result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:9: warning Team.Filler: Remove 'very'.",
            "1 alert in 1 file",
        ]
    );
    Ok(())
}
