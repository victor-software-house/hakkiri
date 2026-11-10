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
use std::path::Path;
use std::process::Command;

use ctl_core::fixture::{Root, Suite};
use serde_json::Value;

type TestResult = Result<(), Box<dyn Error>>;

/// The variables a caller's shell may set that change what hakkiri prints.
/// Agent shells export `CLICOLOR_FORCE=1` and `FORCE_COLOR=3`.
const CALLER_ENV: [&str; 5] = [
    "CLICOLOR",
    "CLICOLOR_FORCE",
    "FORCE_COLOR",
    "NO_COLOR",
    "COLORTERM",
];

/// The hakkiri binary with none of [`CALLER_ENV`] and no `HAKKIRI_*` variable,
/// so a test sees the same output in an agent shell, a terminal and CI. A test
/// about colour or a variable sets its own back on top.
fn binary() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_hakkiri"));
    for name in CALLER_ENV {
        command.env_remove(name);
    }
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("HAKKIRI_") {
            command.env_remove(name);
        }
    }
    command
}

/// What a run gave: its exit status and its two streams.
#[derive(Debug)]
struct Run {
    code: i32,
    out: String,
    err: String,
}

impl Run {
    /// The lines of standard output.
    fn lines(&self) -> Vec<&str> {
        self.out.lines().collect()
    }
}

/// Runs hakkiri in `dir` with `args` and the variables `envs`.
fn run(dir: &Path, args: &[&str], envs: &[(&str, &str)]) -> Result<Run, Box<dyn Error>> {
    let output = binary()
        .current_dir(dir)
        .args(args)
        .envs(envs.iter().copied())
        .output()?;
    Ok(Run {
        code: output.status.code().ok_or("killed by a signal")?,
        out: String::from_utf8(output.stdout)?,
        err: String::from_utf8(output.stderr)?,
    })
}

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
    root.file("docs/hakkiri.yaml", CONFIG);
    root.file("a.md", "This is very good.\n");
    let result = run(
        root.path(),
        &["check"],
        &[("HAKKIRI_CONFIG", "docs/hakkiri.yaml")],
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
