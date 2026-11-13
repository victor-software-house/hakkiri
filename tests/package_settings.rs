//! A package's settings file: the scenarios of the package-settings change.
//!
//! A package folder that holds `.ctl/hakkiri.yaml` ships settings in the format
//! of the project config. Each test writes a project in a temporary directory
//! and runs the binary there. The tests of packages a package lists start a
//! small HTTP server on a local port and serve zips built in the test.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

mod common;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Write as _;
use std::io::{Cursor, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{io, iter, thread};

use common::run;
use ctl_core::fixture::{Root, Suite};
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;

type TestResult = Result<(), Box<dyn Error>>;

/// A rule that flags `very`.
const FILLER: &str = "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\nscope: text\nignorecase: true\ntokens:\n  - very\n";

/// A rule that flags `good`.
const PLAIN: &str = "extends: existence\nmessage: \"Drop '%s'.\"\nlevel: warning\nscope: text\nignorecase: true\ntokens:\n  - good\n";

/// The text every test lints: `very` starts at column 9 and `good` at 14.
const TEXT: &str = "This is very good.\n";

/// The alert of `very` in `file`, with the level `level`.
fn very(file: &str, level: &str) -> String {
    format!("{file}:1:9: {level} Team.Filler: Remove 'very'.")
}

/// A project of one text file, `a.md`, whose config is `config`, with the style
/// `Team` in the package `pkg` and the style `Other` in `own`, each a `styles`
/// folder beside the `.ctl` folder of its settings.
fn project<'a>(suite: &'a Suite, name: &str, config: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("pkg/styles/Team/Filler.yml", FILLER);
    root.file("own/styles/Other/Plain.yml", PLAIN);
    root.file(".ctl/hakkiri.yaml", config);
    root.file("a.md", TEXT);
    root
}

/// `entries` as lines of a list under `packages`, each indented two spaces.
fn indented<S: AsRef<str>>(entries: &[S]) -> String {
    let mut out = String::new();
    for entry in entries {
        let _ = writeln!(out, "  {}", entry.as_ref());
    }
    out
}

/// The output lines of a `check` in `root`, with the exit status.
fn check(root: &Root<'_>) -> Result<(i32, Vec<String>, String), Box<dyn Error>> {
    let result = run(root.path(), &["check"], &[])?;
    let lines = result
        .lines()
        .iter()
        .map(|line| (*line).to_owned())
        .collect();
    Ok((result.code, lines, result.err))
}

#[test]
fn a_package_turns_a_style_on() -> TestResult {
    let suite = Suite::new("package-settings-on");
    let root = project(&suite, "on", "packages:\n  - path: pkg/styles\n");
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "files:\n  '*.md': {styles: [Team]}\n",
    );
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert_eq!(lines, [very("a.md", "warning"), "1 alert in 1 file".into()]);
    Ok(())
}

#[test]
fn a_package_with_no_settings_file_lints_no_file() -> TestResult {
    let suite = Suite::new("package-settings-none");
    let root = project(&suite, "none", "packages:\n  - path: pkg/styles\n");
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert!(
        lines.iter().all(|line| !line.contains("Team.Filler")),
        "{lines:?}"
    );
    Ok(())
}

#[test]
fn the_project_turns_off_a_rule_a_package_runs() -> TestResult {
    let suite = Suite::new("package-settings-off");
    let root = project(
        &suite,
        "off",
        "packages:\n  - path: pkg/styles\nfiles:\n  'a.md': {Team.Filler: off}\n",
    );
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "files:\n  '*.md': {styles: [Team], Team.Filler: warning}\n",
    );
    root.file("b.md", TEXT);
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert_eq!(lines, [very("b.md", "warning"), "1 alert in 1 file".into()]);
    Ok(())
}

#[test]
fn a_rule_a_package_ships_off_stays_off_until_the_project_turns_it_on() -> TestResult {
    let suite = Suite::new("package-settings-opt-in");
    let settings = "files:\n  '*.md': {Team.Filler: off}\n";
    let root = project(
        &suite,
        "off",
        "packages:\n  - path: pkg/styles\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file("pkg/.ctl/hakkiri.yaml", settings);
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert!(
        lines.iter().all(|line| !line.contains("Team.Filler")),
        "{lines:?}"
    );

    let root = project(
        &suite,
        "on",
        "packages:\n  - path: pkg/styles\nfiles:\n  '*.md': {styles: [Team], Team.Filler: on}\n",
    );
    root.file("pkg/.ctl/hakkiri.yaml", settings);
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert_eq!(lines, [very("a.md", "warning"), "1 alert in 1 file".into()]);
    Ok(())
}

#[test]
fn the_projects_level_wins_and_the_packages_applies_when_the_project_has_none() -> TestResult {
    let suite = Suite::new("package-settings-level");
    let settings = "level: error\nfiles:\n  '*.md': {styles: [Team]}\n";

    let root = project(&suite, "package", "packages:\n  - path: pkg/styles\n");
    root.file("pkg/.ctl/hakkiri.yaml", settings);
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert!(
        lines.iter().all(|line| !line.contains("Team.Filler")),
        "{lines:?}"
    );

    let root = project(
        &suite,
        "project",
        "level: suggestion\npackages:\n  - path: pkg/styles\n",
    );
    root.file("pkg/.ctl/hakkiri.yaml", settings);
    let (_, lines, err) = check(&root)?;
    assert_eq!(
        lines,
        [very("a.md", "warning"), "1 alert in 1 file".into()],
        "{err}"
    );
    Ok(())
}

#[test]
fn a_later_package_wins_over_an_earlier_one() -> TestResult {
    let suite = Suite::new("package-settings-order");
    for (name, order, level) in [
        ("first-a", "pkg, a, b", "suggestion"),
        ("first-b", "pkg, b, a", "error"),
    ] {
        let packages = indented(
            &order
                .split(", ")
                .map(|folder| format!("- path: {folder}/styles"))
                .collect::<Vec<_>>(),
        );
        let root = project(&suite, name, &format!("packages:\n{packages}"));
        root.file("a/styles/.keep", "");
        root.file("b/styles/.keep", "");
        root.file(
            "a/.ctl/hakkiri.yaml",
            "files:\n  '*.md': {Team.Filler: error}\n",
        );
        root.file(
            "b/.ctl/hakkiri.yaml",
            "files:\n  '*.md': {Team.Filler: suggestion}\n",
        );
        let (code, lines, err) = check(&root)?;
        assert_eq!(code, i32::from(level == "error"), "{name}: {err}");
        assert_eq!(
            lines.first().map(String::as_str),
            Some(very("a.md", level).as_str()),
            "{name}: {lines:?}"
        );
    }
    Ok(())
}

#[test]
fn styles_add_up_under_the_same_glob_and_replace_under_another() -> TestResult {
    let suite = Suite::new("package-settings-styles");
    let settings = "files:\n  '*.md': {styles: [Team]}\n";
    let both = [
        very("a.md", "warning"),
        "a.md:1:14: warning Other.Plain: Drop 'good'.".to_owned(),
        "2 alerts in 1 file".to_owned(),
    ];

    let root = project(
        &suite,
        "same",
        "packages:\n  - path: pkg/styles\n  - path: own/styles\nfiles:\n  '*.md': {styles: [Other]}\n",
    );
    root.file("pkg/.ctl/hakkiri.yaml", settings);
    let (_, lines, err) = check(&root)?;
    assert_eq!(lines, both, "{err}");

    let root = project(
        &suite,
        "other",
        "packages:\n  - path: pkg/styles\n  - path: own/styles\nfiles:\n  'docs/*.md': {styles: [Other]}\n",
    );
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "files:\n  '*.md': {styles: [Team], Team.Filler: error}\n",
    );
    root.file("docs/a.md", TEXT);
    let (_, lines, err) = check(&root)?;
    assert_eq!(
        lines,
        [
            very("a.md", "error").as_str(),
            very("docs/a.md", "error").as_str(),
            "docs/a.md:1:14: warning Other.Plain: Drop 'good'.",
            "3 alerts in 2 files"
        ],
        "{err}"
    );

    let root = project(
        &suite,
        "empty",
        "packages:\n  - path: pkg/styles\n  - path: own/styles\nfiles:\n  '*.md': {styles: []}\n",
    );
    root.file("pkg/.ctl/hakkiri.yaml", settings);
    let (_, lines, err) = check(&root)?;
    assert!(
        lines.iter().all(|line| !line.contains("Team.")),
        "{lines:?}{err}"
    );
    Ok(())
}

#[test]
fn at_equal_specificity_the_project_beats_a_package() -> TestResult {
    let suite = Suite::new("package-settings-tie");
    let root = project(
        &suite,
        "tie",
        "packages:\n  - path: pkg/styles\nfiles:\n  '*.md': {Team.Filler: warning}\n",
    );
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "files:\n  '*.md': {Team.Filler: error}\n",
    );
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert_eq!(lines, [very("a.md", "warning"), "1 alert in 1 file".into()]);
    Ok(())
}

#[test]
fn a_forced_level_beats_a_more_specific_off() -> TestResult {
    let suite = Suite::new("package-settings-forced-level");
    let root = project(
        &suite,
        "forced",
        "packages:\n  - path: pkg/styles\nfiles:\n  '*.md': {Team.Filler: !override error}\n",
    );
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "files:\n  'docs/**/*.md': {Team.Filler: off}\n",
    );
    root.file("docs/a.md", TEXT);
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 1, "{err}");
    assert_eq!(
        lines,
        [
            very("a.md", "error"),
            very("docs/a.md", "error"),
            "2 alerts in 2 files".into()
        ]
    );
    Ok(())
}

#[test]
fn a_package_that_forces_a_value_is_an_error_naming_its_line() -> TestResult {
    let suite = Suite::new("package-settings-package-forces");
    let root = project(&suite, "forces", "packages:\n  - path: pkg/styles\n");
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "code_span_size: !override 0\nfiles:\n  '*.md':\n    Team.Filler: !override off\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    for expected in [
        "failed pkg/.ctl/hakkiri.yaml: line 1: code_span_size: `!override` is not allowed in a package's settings",
        "failed pkg/.ctl/hakkiri.yaml: line 4: Team.Filler: `!override` is not allowed in a package's settings",
    ] {
        assert!(result.err.contains(expected), "{expected}: {}", result.err);
    }
    Ok(())
}

#[test]
fn a_tag_other_than_override_is_an_error_naming_its_line() -> TestResult {
    let suite = Suite::new("package-settings-other-tag");
    let root = project(
        &suite,
        "tag",
        "packages:\n  - path: pkg/styles\ncode_span_size: !important 2\nfiles:\n  '*.md':\n    Team.Filler: !important off\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    for expected in [
        "failed .ctl/hakkiri.yaml: line 3: code_span_size: `!important` is not a tag hakkiri reads",
        "failed .ctl/hakkiri.yaml: line 6: Team.Filler: `!important` is not a tag hakkiri reads",
    ] {
        assert!(result.err.contains(expected), "{expected}: {}", result.err);
    }
    Ok(())
}

/// A project that lists the packages `pkg` and `own`, whose settings hide
/// `very` in every Markdown file, and the files `a.md` and `docs/a.md`.
fn ignoring<'a>(suite: &'a Suite, name: &str, files: &str) -> Root<'a> {
    let root = project(
        suite,
        name,
        &format!("packages:\n  - path: pkg/styles\n  - path: own/styles\nfiles:\n{files}"),
    );
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "files:\n  '*.md':\n    styles: [Team, Other]\n    token_ignores: ['(very)']\n",
    );
    root.file("docs/a.md", TEXT);
    root
}

#[test]
fn the_ignores_of_every_matching_glob_add_up() -> TestResult {
    let suite = Suite::new("package-settings-ignores-add-up");
    let root = ignoring(
        &suite,
        "add-up",
        "  'docs/*.md':\n    token_ignores: ['(good)']\n",
    );
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        lines,
        [
            "a.md:1:14: warning Other.Plain: Drop 'good'.",
            "1 alert in 2 files"
        ]
    );
    Ok(())
}

#[test]
fn a_forced_list_replaces_the_ignores_without_the_tag() -> TestResult {
    let suite = Suite::new("package-settings-ignores-forced");
    let root = ignoring(
        &suite,
        "forced",
        "  'docs/*.md':\n    token_ignores: !override ['(good)']\n",
    );
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        lines,
        [
            "a.md:1:14: warning Other.Plain: Drop 'good'.",
            very("docs/a.md", "warning").as_str(),
            "2 alerts in 2 files"
        ]
    );
    Ok(())
}

/// A rule that flags `GitHub` and `Kubernetes`, and the text that holds both.
const TERMS: &str = "extends: existence\nmessage: \"Remove '%s'.\"\nlevel: warning\nscope: text\ntokens:\n  - GitHub\n  - Kubernetes\n";

#[test]
fn the_vocabularies_of_a_package_and_the_project_add_up() -> TestResult {
    let suite = Suite::new("package-settings-vocab");
    let package = |root: &Root<'_>| {
        root.file("pkg/styles/Team/Filler.yml", TERMS);
        root.file("pkg/styles/config/vocabularies/Docs/accept.txt", "GitHub\n");
        root.file(
            "pkg/.ctl/hakkiri.yaml",
            "vocab: [Docs]\nfiles:\n  '*.md': {styles: [Team]}\n",
        );
        root.file(
            "own/styles/config/vocabularies/Own/accept.txt",
            "Kubernetes\n",
        );
        root.file("a.md", "Use GitHub and Kubernetes here.\n");
    };

    let root = project(
        &suite,
        "package",
        "packages:\n  - path: pkg/styles\n  - path: own/styles\n",
    );
    package(&root);
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        lines,
        [
            "a.md:1:16: warning Team.Filler: Remove 'Kubernetes'.",
            "1 alert in 1 file"
        ]
    );

    let root = project(
        &suite,
        "both",
        "vocab: [Own]\npackages:\n  - path: pkg/styles\n  - path: own/styles\n",
    );
    package(&root);
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert!(
        lines.iter().all(|line| !line.contains("Team.Filler")),
        "{lines:?}"
    );
    Ok(())
}

#[test]
fn an_empty_vocab_list_drops_the_names_of_the_packages() -> TestResult {
    let suite = Suite::new("package-settings-vocab-clear");
    for (name, config, flagged) in [
        ("missing", "packages:\n  - path: pkg/styles\n", false),
        (
            "empty",
            "vocab: []\npackages:\n  - path: pkg/styles\n",
            true,
        ),
    ] {
        let root = project(&suite, name, config);
        root.file("pkg/styles/Team/Filler.yml", TERMS);
        root.file("pkg/styles/config/vocabularies/Docs/accept.txt", "GitHub\n");
        root.file(
            "pkg/.ctl/hakkiri.yaml",
            "vocab: [Docs]\nfiles:\n  '*.md': {styles: [Team]}\n",
        );
        root.file("a.md", "Use GitHub here.\n");
        let (code, lines, err) = check(&root)?;
        assert_eq!(code, 0, "{name}: {err}");
        assert_eq!(
            lines.iter().any(|line| line.contains("Remove 'GitHub'")),
            flagged,
            "{name}: {lines:?}"
        );
    }
    Ok(())
}

#[test]
fn a_later_package_can_clear_the_vocabularies_of_an_earlier_one() -> TestResult {
    let suite = Suite::new("package-settings-vocab-later");
    let root = project(
        &suite,
        "package",
        "packages:\n  - path: pkg/styles\n  - path: later/styles\n",
    );
    root.file("pkg/styles/Team/Filler.yml", TERMS);
    root.file("pkg/styles/config/vocabularies/Docs/accept.txt", "GitHub\n");
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "vocab: [Docs]\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file("later/styles/.keep", "");
    root.file("later/.ctl/hakkiri.yaml", "vocab: []\n");
    root.file("a.md", "Use GitHub here.\n");
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert!(
        lines.iter().any(|line| line.contains("Remove 'GitHub'")),
        "{lines:?}"
    );
    Ok(())
}

#[test]
fn an_empty_vocab_in_a_glob_clears_only_the_lists_of_the_same_glob_text() -> TestResult {
    let suite = Suite::new("package-settings-vocab-glob");
    // The package's `*.md` glob names `Docs`, which accepts `GitHub`, and its
    // `a.md` glob names `Own`, which accepts `Kubernetes`. Both globs match
    // `a.md`, and only the first matches `b.md`.
    let write = |root: &Root<'_>| {
        root.file("pkg/styles/Team/Filler.yml", TERMS);
        root.file("pkg/styles/config/vocabularies/Docs/accept.txt", "GitHub\n");
        root.file(
            "pkg/styles/config/vocabularies/Own/accept.txt",
            "Kubernetes\n",
        );
        root.file(
            "pkg/.ctl/hakkiri.yaml",
            "files:\n  '*.md':\n    styles: [Team]\n    vocab: [Docs]\n  'a.md':\n    vocab: [Own]\n",
        );
        root.file("a.md", "Use GitHub and Kubernetes here.\n");
        root.file("b.md", "Use GitHub and Kubernetes here.\n");
    };
    let flagged = |lines: &[String], file: &str, word: &str| {
        lines
            .iter()
            .any(|line| line.starts_with(file) && line.contains(&format!("Remove '{word}'")))
    };
    // The glob the project writes, and whether `GitHub` is flagged in `a.md`
    // and in `b.md`, and `Kubernetes` in `a.md`. `Kubernetes` is flagged in
    // `b.md` whatever the project writes.
    for (name, glob, a_github, b_github, a_kubernetes) in [
        ("same", "'*.md': {vocab: []}", true, true, false),
        ("same-a", "'a.md': {vocab: []}", false, false, true),
        ("other", "'[ab].md': {vocab: []}", false, false, false),
        ("missing", "'*.md': {styles: [Team]}", false, false, false),
    ] {
        let root = project(
            &suite,
            name,
            &format!("packages:\n  - path: pkg/styles\nfiles:\n  {glob}\n"),
        );
        write(&root);
        let (code, lines, err) = check(&root)?;
        assert_eq!(code, 0, "{name}: {err}");
        assert_eq!(
            flagged(&lines, "a.md", "GitHub"),
            a_github,
            "{name}: {lines:?}"
        );
        assert_eq!(
            flagged(&lines, "b.md", "GitHub"),
            b_github,
            "{name}: {lines:?}"
        );
        assert_eq!(
            flagged(&lines, "a.md", "Kubernetes"),
            a_kubernetes,
            "{name}: {lines:?}"
        );
        assert!(flagged(&lines, "b.md", "Kubernetes"), "{name}: {lines:?}");
    }
    Ok(())
}

#[test]
fn a_vocabulary_a_package_names_that_no_package_holds_is_an_error() -> TestResult {
    let suite = Suite::new("package-settings-vocab-unknown");
    let root = project(&suite, "unknown", "packages:\n  - path: pkg/styles\n");
    root.file("pkg/.ctl/hakkiri.yaml", "vocab: [Nope]\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result.err.contains(
            "failed pkg/.ctl/hakkiri.yaml: line 1: vocab[0]: `Nope` is not a vocabulary in any package"
        ),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_key_hakkiri_does_not_define_in_a_package_file_names_the_line_and_the_package() -> TestResult {
    let suite = Suite::new("package-settings-unknown");
    let root = project(&suite, "unknown", "packages:\n  - path: pkg/styles\n");
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "level: warning\nMinAlertLevel: warning\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("failed pkg/.ctl/hakkiri.yaml: line 2: unknown field `MinAlertLevel`"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_rule_setting_in_a_package_file_that_names_no_rule_is_an_error() -> TestResult {
    let suite = Suite::new("package-settings-no-rule");
    let root = project(&suite, "norule", "packages:\n  - path: pkg/styles\n");
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "files:\n  '*.md':\n    styles: [Team]\n    Team.Nope: warning\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("failed pkg/.ctl/hakkiri.yaml: line 4: Team.Nope: names no rule"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_package_that_lists_a_path_is_an_error() -> TestResult {
    let suite = Suite::new("package-settings-nested-path");
    let root = project(&suite, "nested", "packages:\n  - path: pkg/styles\n");
    root.file("pkg/.ctl/hakkiri.yaml", "packages:\n  - path: ../own\n");
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result.err.contains(
            "failed pkg/.ctl/hakkiri.yaml: line 2: path: is not allowed in a package's settings"
        ),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn the_project_config_is_not_a_package_top_when_the_styles_sit_in_the_root() -> TestResult {
    let suite = Suite::new("package-settings-self");
    let root = suite.scenario("self");
    root.file("styles/Team/Filler.yml", FILLER);
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file("a.md", TEXT);
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert_eq!(lines, [very("a.md", "warning"), "1 alert in 1 file".into()]);
    Ok(())
}

#[test]
fn another_config_in_the_root_is_not_a_package_top_either() -> TestResult {
    let suite = Suite::new("package-settings-other-config");
    let root = suite.scenario("other");
    root.file("styles/Team/Filler.yml", FILLER);
    root.file(
        "cfg.yaml",
        "packages:\n  - path: styles\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file(".ctl/hakkiri.yaml", "NotAKey: 1\n");
    root.file("a.md", TEXT);
    let result = run(root.path(), &["check", "--config", "cfg.yaml"], &[])?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [very("a.md", "warning").as_str(), "1 alert in 1 file"]
    );
    Ok(())
}

#[test]
fn a_styles_folder_with_another_name_has_no_top() -> TestResult {
    let suite = Suite::new("package-settings-name");
    let root = suite.scenario("name");
    root.file("pkg/rules/Team/Filler.yml", FILLER);
    root.file("pkg/.ctl/hakkiri.yaml", "NotAKey: 1\n");
    root.file(
        ".ctl/hakkiri.yaml",
        "packages:\n  - path: pkg/rules\nfiles:\n  '*.md': {styles: [Team]}\n",
    );
    root.file("a.md", TEXT);
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert_eq!(lines, [very("a.md", "warning"), "1 alert in 1 file".into()]);
    Ok(())
}

/// What the server holds, by path.
type Routes = Arc<Mutex<HashMap<String, Vec<u8>>>>;

/// A server on a local port, whose routes can be added after it starts: a zip
/// names the URL of the next one, so the port comes first.
struct Server {
    address: SocketAddr,
    routes: Routes,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Server {
    fn start() -> Result<Self, Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let routes: Routes = Arc::default();
        let stop = Arc::new(AtomicBool::new(false));
        let held = Arc::clone(&routes);
        let done = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            for stream in listener.incoming() {
                if done.load(Ordering::SeqCst) {
                    break;
                }
                if let Ok(stream) = stream {
                    let _ = answer(stream, &held);
                }
            }
        });
        Ok(Self {
            address,
            routes,
            stop,
            thread: Some(thread),
        })
    }

    /// The URL of `path` on this server.
    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.address)
    }

    /// Serves `body` at `path`.
    fn add(&self, path: &str, body: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self.routes
            .lock()
            .map_err(|error| format!("the routes are poisoned: {error}"))?
            .insert(path.to_owned(), body);
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.address);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Reads one request from `stream` and writes the body of its route, or 404.
fn answer(mut stream: TcpStream, routes: &Routes) -> io::Result<()> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Ok(());
        }
        request.extend_from_slice(&buffer[..read]);
    }
    let text = String::from_utf8_lossy(&request);
    let path = text.split_whitespace().nth(1).unwrap_or("/");
    let body = routes.lock().ok().and_then(|held| held.get(path).cloned());
    match body {
        Some(body) => {
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )?;
            stream.write_all(&body)?;
        }
        None => write!(
            stream,
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )?,
    }
    stream.flush()
}

/// A zip with the top folder `Pkg`, a `styles` folder, and each of `files`
/// under `Pkg`.
fn package_zip(files: &[(&str, &str)]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, text) in iter::once(&("styles/.keep", "")).chain(files) {
        writer.start_file(format!("Pkg/{name}"), SimpleFileOptions::default())?;
        writer.write_all(text.as_bytes())?;
    }
    Ok(writer.finish()?.into_inner())
}

/// The lowercase hex SHA-256 of `bytes`.
fn sha256(bytes: &[u8]) -> String {
    let mut out = String::new();
    for byte in Sha256::digest(bytes) {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Serves the zip `files` at `path`, and returns the `packages` entry for it.
fn serve(server: &Server, path: &str, files: &[(&str, &str)]) -> Result<String, Box<dyn Error>> {
    let zip = package_zip(files)?;
    let entry = format!("- url: {}\n    sha256: {}", server.url(path), sha256(&zip));
    server.add(path, zip)?;
    Ok(entry)
}

/// Runs `check` in `root` with its cache in a folder of `suite`.
fn check_cached(suite: &Suite, root: &Root<'_>, name: &str) -> Result<common::Run, Box<dyn Error>> {
    let cache = suite.scenario(name);
    let cache = cache
        .path()
        .to_str()
        .ok_or("the cache path is not UTF-8")?
        .to_owned();
    let envs = [
        ("HAKKIRI_CACHE_DIR", cache.as_str()),
        ("NO_PROXY", "127.0.0.1,localhost"),
        ("no_proxy", "127.0.0.1,localhost"),
    ];
    run(root.path(), &["check"], &envs)
}

/// A project whose config lists `entry` and reads `a.md`, with no `files`.
fn listing<'a>(suite: &'a Suite, name: &str, entries: &[&str]) -> Root<'a> {
    let root = suite.scenario(name);
    root.file(
        ".ctl/hakkiri.yaml",
        format!("packages:\n{}", indented(entries)),
    );
    root.file("a.md", TEXT);
    root
}

#[test]
fn a_package_a_package_lists_is_fetched_and_its_styles_run() -> TestResult {
    let suite = Suite::new("package-settings-fetched");
    let server = Server::start()?;
    let base = serve(&server, "/base.zip", &[("styles/Team/Filler.yml", FILLER)])?;
    let settings = format!("packages:\n  {base}\nfiles:\n  '*.md': {{styles: [Team]}}\n");
    let top = serve(&server, "/top.zip", &[(".ctl/hakkiri.yaml", &settings)])?;
    let root = listing(&suite, "project", &[&top]);
    let result = check_cached(&suite, &root, "cache")?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [very("a.md", "warning").as_str(), "1 alert in 1 file"]
    );
    Ok(())
}

#[test]
fn the_settings_of_a_package_override_those_of_the_packages_it_lists() -> TestResult {
    let suite = Suite::new("package-settings-nested-order");
    let server = Server::start()?;
    let base = serve(
        &server,
        "/base.zip",
        &[
            ("styles/Team/Filler.yml", FILLER),
            (
                ".ctl/hakkiri.yaml",
                "files:\n  '*.md': {styles: [Team], Team.Filler: error}\n",
            ),
        ],
    )?;
    let settings = format!("packages:\n  {base}\nfiles:\n  '*.md': {{Team.Filler: suggestion}}\n");
    let top = serve(&server, "/top.zip", &[(".ctl/hakkiri.yaml", &settings)])?;
    let root = listing(&suite, "project", &[&top]);
    let result = check_cached(&suite, &root, "cache")?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [very("a.md", "suggestion").as_str(), "1 alert in 1 file"]
    );
    Ok(())
}

#[test]
fn a_package_named_twice_loads_once() -> TestResult {
    let suite = Suite::new("package-settings-twice");
    let server = Server::start()?;
    let base = serve(
        &server,
        "/base.zip",
        &[
            ("styles/Team/Filler.yml", FILLER),
            (".ctl/hakkiri.yaml", "files:\n  '*.md': {styles: [Team]}\n"),
        ],
    )?;
    let settings = format!("packages:\n  {base}\n");
    let top = serve(&server, "/top.zip", &[(".ctl/hakkiri.yaml", &settings)])?;
    let root = listing(&suite, "project", &[&base, &top]);
    let result = check_cached(&suite, &root, "cache")?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [very("a.md", "warning").as_str(), "1 alert in 1 file"]
    );
    Ok(())
}

/// A chain of `length` packages, each listing the next, and the project that
/// lists the first. The last holds a style, so a chain that loads draws an
/// alert.
fn chain<'a>(
    suite: &'a Suite,
    server: &Server,
    name: &str,
    length: usize,
) -> Result<Root<'a>, Box<dyn Error>> {
    let mut next = serve(
        server,
        &format!("/p{length}.zip"),
        &[
            ("styles/Team/Filler.yml", FILLER),
            (".ctl/hakkiri.yaml", "files:\n  '*.md': {styles: [Team]}\n"),
        ],
    )?;
    for index in (1..length).rev() {
        let settings = format!("packages:\n  {next}\n");
        next = serve(
            server,
            &format!("/p{index}.zip"),
            &[(".ctl/hakkiri.yaml", &settings)],
        )?;
    }
    Ok(listing(suite, name, &[&next]))
}

#[test]
fn a_chain_of_ten_packages_loads() -> TestResult {
    let suite = Suite::new("package-settings-ten");
    let server = Server::start()?;
    let root = chain(&suite, &server, "project", 10)?;
    let result = check_cached(&suite, &root, "cache")?;
    assert_eq!(result.code, 0, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [very("a.md", "warning").as_str(), "1 alert in 1 file"]
    );
    Ok(())
}

#[test]
fn a_chain_of_eleven_packages_is_an_error() -> TestResult {
    let suite = Suite::new("package-settings-eleven");
    let server = Server::start()?;
    let root = chain(&suite, &server, "project", 11)?;
    let result = check_cached(&suite, &root, "cache")?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    assert!(
        result
            .err
            .contains("is over the limit of 10 packages in a chain"),
        "{}",
        result.err
    );
    Ok(())
}

#[test]
fn a_package_that_sets_a_run_option_is_an_error() -> TestResult {
    let suite = Suite::new("package-settings-run-options");
    let root = project(&suite, "run-options", "packages:\n  - path: pkg/styles\n");
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "files: {}\ngitignore: false\nfail_level: warning\non_unmatched: error\non_unknown: warn\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    for (line, key) in [
        (2, "gitignore"),
        (3, "fail_level"),
        (4, "on_unmatched"),
        (5, "on_unknown"),
    ] {
        let want = format!(
            "failed pkg/.ctl/hakkiri.yaml: line {line}: {key}: belongs to the project's config"
        );
        assert!(result.err.contains(&want), "{}", result.err);
    }
    Ok(())
}

#[test]
fn a_package_maps_extensions_and_the_project_overrides_it() -> TestResult {
    let suite = Suite::new("package-settings-formats");
    let root = project(&suite, "formats", "packages:\n  - path: pkg/styles\n");
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "formats:\n  txt: md\nfiles:\n  '*': {styles: [Team]}\n",
    );
    root.file("a.txt", "```\nvery\n```\n");
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert!(
        lines.iter().all(|line| !line.contains("a.txt")),
        "the package reads a.txt as md: {lines:?}"
    );
    root.file(
        ".ctl/hakkiri.yaml",
        "formats:\n  txt: text\npackages:\n  - path: pkg/styles\n",
    );
    let (code, lines, err) = check(&root)?;
    assert_eq!(code, 0, "{err}");
    assert!(
        lines.contains(&very("a.txt", "warning").replace(":1:9:", ":2:1:")),
        "the project reads a.txt as text: {lines:?}"
    );
    Ok(())
}

#[test]
fn a_refused_key_of_a_package_file_names_its_own_line_whatever_its_value_spans() -> TestResult {
    let suite = Suite::new("package-settings-refused-key-lines");
    let root = project(&suite, "refused", "packages:\n  - path: pkg/styles\n");
    root.file(
        "pkg/.ctl/hakkiri.yaml",
        "level: warning\nfail_level:\n  error\npackages:\n  - path:\n      ../own\n",
    );
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 2, "{}{}", result.out, result.err);
    for expected in [
        "line 2: fail_level: belongs to the project's config",
        "line 5: path: is not allowed in a package's settings",
    ] {
        assert!(result.err.contains(expected), "{expected}: {}", result.err);
    }
    Ok(())
}
