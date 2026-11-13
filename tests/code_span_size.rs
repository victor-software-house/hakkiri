//! `code_span_size`: how many words a code span counts as in a word count and
//! in the metrics built on words.
//!
//! Each test writes a project in a temporary directory and runs the binary
//! there. The rule measures the words of each sentence, so a code span moves
//! its count: 7 words at 1, 3 at 0, which is what Vale counts, and 11 at 2.

#![expect(
    clippy::panic_in_result_fn,
    reason = "A test returns Result for `?` and fails through an assertion."
)]

mod common;

use std::error::Error;

use common::run;
use ctl_core::fixture::{Root, Suite};

type TestResult = Result<(), Box<dyn Error>>;

/// A rule that prints the words of each sentence.
const WORDS: &str = "extends: metric\nmessage: \"%s words\"\nlevel: warning\nscope: sentence\nformula: words\ncondition: '> 0'\n";

/// The text of every file: a sentence with four code spans.
const TEXT: &str = "Run `a`, `b`, `c` and `one two three` today.\n";

/// The style `Team` in `pkg/styles`, a package with settings `settings`, and
/// `a.md` and `b.md` under the config `config`.
fn project<'a>(suite: &'a Suite, name: &str, config: &str, settings: &str) -> Root<'a> {
    let root = suite.scenario(name);
    root.file("pkg/styles/Team/Words.yml", WORDS);
    root.file("pkg/.ctl/hakkiri.yaml", settings);
    root.file(".ctl/hakkiri.yaml", config);
    root.file("a.md", TEXT);
    root.file("b.md", TEXT);
    root
}

/// The config that lists the package and runs `Team` on every Markdown file,
/// with `extra` after it.
fn config(extra: &str) -> String {
    format!("packages:\n  - path: pkg/styles\nfiles:\n  '*.md': {{styles: [Team]}}\n{extra}")
}

/// The alert lines of a run in `root` with `args` and `envs`.
fn lines(
    root: &Root<'_>,
    args: &[&str],
    envs: &[(&str, &str)],
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut all = vec!["check"];
    all.extend(args);
    let result = run(root.path(), &all, envs)?;
    assert_eq!(result.code, 0, "{}", result.err);
    Ok(result
        .lines()
        .iter()
        .map(|line| (*line).to_owned())
        .collect())
}

/// The line of the alert of `file` with `words` words.
fn counted(file: &str, words: &str) -> String {
    format!("{file}:1:1: warning Team.Words: {words} words")
}

#[test]
fn a_code_span_is_one_word_by_default() -> TestResult {
    let suite = Suite::new("code-span-size-default");
    let root = project(&suite, "default", &config(""), "{}\n");
    assert_eq!(
        lines(&root, &[], &[])?,
        [
            counted("a.md", "7.00"),
            counted("b.md", "7.00"),
            "2 alerts in 2 files".into()
        ]
    );
    Ok(())
}

#[test]
fn the_project_sets_the_size() -> TestResult {
    let suite = Suite::new("code-span-size-project");
    let root = project(&suite, "zero", &config("code_span_size: 0\n"), "{}\n");
    assert_eq!(
        lines(&root, &[], &[])?,
        [
            counted("a.md", "3.00"),
            counted("b.md", "3.00"),
            "2 alerts in 2 files".into()
        ]
    );
    Ok(())
}

#[test]
fn a_package_sets_the_size_and_the_project_overrides_it() -> TestResult {
    let suite = Suite::new("code-span-size-package");
    let from_package = project(&suite, "package", &config(""), "code_span_size: 0\n");
    assert_eq!(
        lines(&from_package, &[], &[])?,
        [
            counted("a.md", "3.00"),
            counted("b.md", "3.00"),
            "2 alerts in 2 files".into()
        ]
    );
    let overridden = project(
        &suite,
        "project",
        &config("code_span_size: 2\n"),
        "code_span_size: 0\n",
    );
    assert_eq!(
        lines(&overridden, &[], &[])?,
        [
            counted("a.md", "11.00"),
            counted("b.md", "11.00"),
            "2 alerts in 2 files".into()
        ]
    );
    Ok(())
}

#[test]
fn a_glob_sets_the_size_for_its_files() -> TestResult {
    let suite = Suite::new("code-span-size-glob");
    let root = project(
        &suite,
        "glob",
        &config("  'a.md': {code_span_size: 0}\ncode_span_size: 2\n"),
        "{}\n",
    );
    assert_eq!(
        lines(&root, &[], &[])?,
        [
            counted("a.md", "3.00"),
            counted("b.md", "11.00"),
            "2 alerts in 2 files".into()
        ]
    );
    Ok(())
}

#[test]
fn the_variable_beats_the_config_and_the_flag_beats_the_variable() -> TestResult {
    let suite = Suite::new("code-span-size-env");
    let root = project(
        &suite,
        "env",
        &config("  'a.md': {code_span_size: 2}\ncode_span_size: 2\n"),
        "{}\n",
    );
    assert_eq!(
        lines(&root, &[], &[("HAKKIRI_CODE_SPAN_SIZE", "0")])?,
        [
            counted("a.md", "3.00"),
            counted("b.md", "3.00"),
            "2 alerts in 2 files".into()
        ]
    );
    assert_eq!(
        lines(
            &root,
            &["--code-span-size", "1"],
            &[("HAKKIRI_CODE_SPAN_SIZE", "0")]
        )?,
        [
            counted("a.md", "7.00"),
            counted("b.md", "7.00"),
            "2 alerts in 2 files".into()
        ]
    );
    Ok(())
}

#[test]
fn a_size_outside_zero_to_255_is_refused_with_its_line() -> TestResult {
    let suite = Suite::new("code-span-size-refused");
    for (name, extra, line, said) in [
        (
            "top",
            "code_span_size: 300\n",
            5,
            "`300` is too many words: use 0 to 255",
        ),
        (
            "glob",
            "  'a.md': {code_span_size: 300}\n",
            5,
            "`300` is too many words: use 0 to 255",
        ),
        (
            "word",
            "  'a.md': {code_span_size: many}\n",
            5,
            "is the number of words",
        ),
    ] {
        let root = project(&suite, name, &config(extra), "{}\n");
        let result = run(root.path(), &["check"], &[])?;
        assert_eq!(result.code, 2, "{name}: {}", result.out);
        let text = format!("{}{}", result.out, result.err);
        assert!(
            text.contains(&format!(".ctl/hakkiri.yaml: line {line}")),
            "{name}: {text}"
        );
        assert!(text.contains(said), "{name}: {text}");
    }
    Ok(())
}

#[test]
fn a_package_glob_beats_the_projects_top_level_unless_the_project_forces_it() -> TestResult {
    let suite = Suite::new("code-span-size-cascade");
    let package = "files:\n  '*.md': {code_span_size: 0}\n";
    let ranked = project(&suite, "ranked", &config("code_span_size: 2\n"), package);
    assert_eq!(
        lines(&ranked, &[], &[])?,
        [
            counted("a.md", "3.00"),
            counted("b.md", "3.00"),
            "2 alerts in 2 files".into()
        ]
    );
    let forced = project(
        &suite,
        "forced",
        &config("code_span_size: !override 2\n"),
        package,
    );
    assert_eq!(
        lines(&forced, &[], &[])?,
        [
            counted("a.md", "11.00"),
            counted("b.md", "11.00"),
            "2 alerts in 2 files".into()
        ]
    );
    Ok(())
}

#[test]
fn the_settings_of_different_keys_merge_across_globs() -> TestResult {
    let suite = Suite::new("code-span-size-merge");
    let root = project(
        &suite,
        "merge",
        &config("  'docs/*.md': {code_span_size: 0}\n"),
        "files:\n  '*.md': {Team.Words: error}\n",
    );
    root.file("docs/a.md", TEXT);
    let result = run(root.path(), &["check"], &[])?;
    assert_eq!(result.code, 1, "{}{}", result.out, result.err);
    assert_eq!(
        result.lines(),
        [
            "a.md:1:1: error Team.Words: 7.00 words",
            "b.md:1:1: error Team.Words: 7.00 words",
            "docs/a.md:1:1: error Team.Words: 3.00 words",
            "3 alerts in 3 files"
        ]
    );
    Ok(())
}
